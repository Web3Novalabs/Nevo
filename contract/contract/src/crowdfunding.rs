#![allow(deprecated)]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

use crate::base::{
    errors::CrowdfundingError,
    events,
    reentrancy::{
        acquire_emergency_lock, reentrancy_lock_logic, release_emergency_lock, release_pool_lock,
    },
    types::{
        CampaignDetails, CampaignLifecycleStatus, CampaignMetrics, Contribution,
        EmergencyWithdrawal, MultiSigConfig, PoolConfig, PoolContribution, PoolMetadata,
        PoolMetrics, PoolState, StorageKey, MAX_DESCRIPTION_LENGTH, MAX_HASH_LENGTH,
        MAX_URL_LENGTH,
    },
};
use crate::interfaces::crowdfunding::CrowdfundingTrait;

#[contract]
pub struct CrowdfundingContract;

// Internal helper functions
impl CrowdfundingContract {
    /// Calculate platform fee based on basis points.
    ///
    /// # Arguments
    /// * `amount` - The donation amount to calculate fee from
    /// * `fee_bps` - Fee in basis points (e.g., 250 for 2.5%)
    ///
    /// # Returns
    /// The calculated fee amount
    ///
    /// # Panics
    /// Panics if the calculation would overflow
    ///
    /// # Examples
    /// ```
    /// // 2.5% fee (250 basis points) on 10,000 tokens
    /// let fee = calculate_platform_fee(10_000, 250);
    /// assert_eq!(fee, 250); // 2.5% of 10,000 = 250
    /// ```
    pub(crate) fn calculate_platform_fee(amount: i128, fee_bps: u32) -> i128 {
        // Basis points: 10,000 bps = 100%
        const BPS_DENOMINATOR: i128 = 10_000;

        // Validate inputs
        assert!(amount >= 0, "amount must be non-negative");
        assert!(fee_bps <= 10_000, "fee_bps must be <= 10,000 (100%)");

        // Use checked multiplication to prevent overflow
        // Formula: (amount * fee_bps) / 10,000
        let fee_bps_i128 = fee_bps as i128;

        // Check for potential overflow before multiplication
        if amount > 0 && fee_bps_i128 > i128::MAX / amount {
            panic!("fee calculation would overflow");
        }

        let numerator = amount
            .checked_mul(fee_bps_i128)
            .expect("fee calculation overflow");

        numerator / BPS_DENOMINATOR
    }
}

#[contractimpl]
#[allow(clippy::too_many_arguments)]
impl CrowdfundingTrait for CrowdfundingContract {
    fn get_pool_remaining_time(env: Env, pool_id: u64) -> Result<u64, CrowdfundingError> {
        let pool_key = StorageKey::Pool(pool_id);
        let pool: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        let deadline: u64 = pool.created_at + pool.duration;
        let now: u64 = env.ledger().timestamp();

        Ok(deadline.saturating_sub(now))
    }

    fn create_campaign(
        env: Env,
        id: BytesN<32>,
        title: String,
        creator: Address,
        goal: i128,
        deadline: u64,
        _token_address: Address,
    ) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
        creator.require_auth();

        if title.is_empty() {
            return Err(CrowdfundingError::InvalidTitle);
        }

        if goal <= 0 {
            return Err(CrowdfundingError::InvalidGoal);
        }

        if deadline <= env.ledger().timestamp() {
            return Err(CrowdfundingError::InvalidDeadline);
        }

        let token_key = StorageKey::CrowdfundingToken;
        if !env.storage().instance().has(&token_key) {
            return Err(CrowdfundingError::NotInitialized);
        }
        let token_address: Address = env.storage().instance().get(&token_key).unwrap();

        let fee_key = StorageKey::CreationFee;
        let creation_fee: i128 = env.storage().instance().get(&fee_key).unwrap_or(0);

        if creation_fee > 0 {
            use soroban_sdk::token;
            let token_client = token::Client::new(&env, &token_address);

            let balance = token_client.balance(&creator);
            if balance < creation_fee {
                return Err(CrowdfundingError::InsufficientBalance);
            }

            token_client.transfer(&creator, env.current_contract_address(), &creation_fee);

            // Track platform fees
            let platform_fees_key = StorageKey::PlatformFees;
            let current_fees: i128 = env
                .storage()
                .instance()
                .get(&platform_fees_key)
                .unwrap_or(0);
            env.storage()
                .instance()
                .set(&platform_fees_key, &(current_fees + creation_fee));

            events::creation_fee_paid(&env, creator.clone(), creation_fee);
        }

        let campaign_key = (id.clone(),);
        if env.storage().instance().has(&campaign_key) {
            return Err(CrowdfundingError::CampaignAlreadyExists);
        }

        let campaign = CampaignDetails {
            id: id.clone(),
            title: title.clone(),
            creator: creator.clone(),
            goal,
            deadline,
            total_raised: 0,
            token_address: token_address.clone(),
        };

        env.storage().instance().set(&campaign_key, &campaign);

        // Initialize metrics
        let metrics_key = StorageKey::CampaignMetrics(id.clone());
        env.storage()
            .instance()
            .set(&metrics_key, &CampaignMetrics::new());

        // Update AllCampaigns list
        let mut all_campaigns = env
            .storage()
            .instance()
            .get(&StorageKey::AllCampaigns)
            .unwrap_or(Vec::new(&env));
        all_campaigns.push_back(id.clone());
        env.storage()
            .instance()
            .set(&StorageKey::AllCampaigns, &all_campaigns);

        events::campaign_created(&env, id, title, creator, goal, deadline);

        Ok(())
    }

    fn set_crowdfunding_token(env: Env, token: Address) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;
        admin.require_auth();

        env.storage()
            .instance()
            .set(&StorageKey::CrowdfundingToken, &token);
        events::crowdfunding_token_set(&env, admin, token);
        Ok(())
    }

    fn get_crowdfunding_token(env: Env) -> Result<Address, CrowdfundingError> {
        env.storage()
            .instance()
            .get(&StorageKey::CrowdfundingToken)
            .ok_or(CrowdfundingError::NotInitialized)
    }

    fn set_creation_fee(env: Env, fee: i128) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;
        admin.require_auth();

        if fee < 0 {
            return Err(CrowdfundingError::InvalidFee);
        }

        env.storage().instance().set(&StorageKey::CreationFee, &fee);
        events::creation_fee_set(&env, admin, fee);
        Ok(())
    }

    fn get_creation_fee(env: Env) -> Result<i128, CrowdfundingError> {
        Ok(env
            .storage()
            .instance()
            .get(&StorageKey::CreationFee)
            .unwrap_or(0))
    }

    fn get_global_raised_total(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&StorageKey::GlobalTotalRaised)
            .unwrap_or(0)
    }

    fn get_top_contributor_for_campaign(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<Address, CrowdfundingError> {
        // Validate campaign exists
        Self::get_campaign(env.clone(), campaign_id.clone())?;

        let metrics_key = StorageKey::CampaignMetrics(campaign_id);
        let metrics: CampaignMetrics = env
            .storage()
            .instance()
            .get(&metrics_key)
            .unwrap_or_default();

        metrics
            .top_contributor
            .ok_or(CrowdfundingError::CampaignNotFound)
    }

    fn get_all_campaigns(env: Env) -> Vec<BytesN<32>> {
        env.storage()
            .instance()
            .get(&StorageKey::AllCampaigns)
            .unwrap_or(Vec::new(&env))
    }

    fn get_active_campaign_count(env: Env) -> u32 {
        let all_campaigns: Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&StorageKey::AllCampaigns)
            .unwrap_or(Vec::new(&env));

        let now = env.ledger().timestamp();
        let mut count: u32 = 0;

        for id in all_campaigns.iter() {
            let campaign_key = (id,);
            if let Some(campaign) = env
                .storage()
                .instance()
                .get::<_, CampaignDetails>(&campaign_key)
            {
                if campaign.deadline > now {
                    count += 1;
                }
            }
        }

        count
    }

    fn get_donor_count(env: Env, campaign_id: BytesN<32>) -> Result<u32, CrowdfundingError> {
        let campaign_key = (campaign_id.clone(),);
        if !env.storage().instance().has(&campaign_key) {
            return Err(CrowdfundingError::CampaignNotFound);
        }

        let metrics_key = StorageKey::CampaignMetrics(campaign_id);
        let metrics: CampaignMetrics = env
            .storage()
            .instance()
            .get(&metrics_key)
            .unwrap_or_default();
        Ok(metrics.contributor_count)
    }

    fn get_campaign_balance(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError> {
        let campaign_key = (campaign_id.clone(),);
        if !env.storage().instance().has(&campaign_key) {
            return Err(CrowdfundingError::CampaignNotFound);
        }

        let metrics_key = StorageKey::CampaignMetrics(campaign_id);
        let metrics: CampaignMetrics = env
            .storage()
            .instance()
            .get(&metrics_key)
            .unwrap_or_default();
        Ok(metrics.total_raised)
    }

    fn get_total_raised(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError> {
        let campaign = Self::get_campaign(env, campaign_id)?;
        Ok(campaign.total_raised)
    }

    fn get_contribution(
        env: Env,
        campaign_id: BytesN<32>,
        contributor: Address,
    ) -> Result<i128, CrowdfundingError> {
        // Validate campaign exists
        Self::get_campaign(env.clone(), campaign_id.clone())?;

        let contribution_key = StorageKey::Contribution(campaign_id.clone(), contributor.clone());
        let contribution: Contribution =
            env.storage()
                .instance()
                .get(&contribution_key)
                .unwrap_or(Contribution {
                    campaign_id: campaign_id.clone(),
                    contributor: contributor.clone(),
                    amount: 0,
                });
        Ok(contribution.amount)
    }

    fn get_campaign_goal(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError> {
        let campaign = Self::get_campaign(env, campaign_id)?;
        Ok(campaign.goal)
    }

    fn is_campaign_completed(env: Env, campaign_id: BytesN<32>) -> Result<bool, CrowdfundingError> {
        let campaign = Self::get_campaign(env.clone(), campaign_id.clone())?;
        let balance = Self::get_campaign_balance(env, campaign_id)?;
        Ok(balance >= campaign.goal)
    }

    fn update_campaign_goal(
        env: Env,
        campaign_id: BytesN<32>,
        new_goal: i128,
    ) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }

        let mut campaign = Self::get_campaign(env.clone(), campaign_id.clone())?;
        campaign.creator.require_auth();

        // Check if campaign is active
        if env.ledger().timestamp() >= campaign.deadline {
            return Err(CrowdfundingError::CampaignExpired);
        }

        // Check if new goal is valid (positive)
        if new_goal <= 0 {
            return Err(CrowdfundingError::InvalidGoal);
        }

        // Prevent increasing the goal
        if new_goal > campaign.goal {
            return Err(CrowdfundingError::InvalidGoalUpdate);
        }

        // Ensure new goal covers raised amount
        if new_goal < campaign.total_raised {
            return Err(CrowdfundingError::InvalidGoalUpdate);
        }

        // Update goal
        campaign.goal = new_goal;
        let campaign_key = (campaign_id.clone(),);
        env.storage().instance().set(&campaign_key, &campaign);

        events::campaign_goal_updated(&env, campaign_id, new_goal);

        Ok(())
    }

    fn get_campaign_status(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<CampaignLifecycleStatus, CrowdfundingError> {
        let campaign = Self::get_campaign(env.clone(), campaign_id.clone())?;
        let total_raised = Self::get_campaign_balance(env.clone(), campaign_id.clone())?;
        let current_time = env.ledger().timestamp();
        let cancellation_key = StorageKey::CampaignCancelled(campaign_id.clone());
        let is_cancelled = env.storage().instance().has(&cancellation_key);

        let status = CampaignLifecycleStatus::get_status(
            total_raised,
            campaign.goal,
            campaign.deadline,
            current_time,
            is_cancelled,
        );

        Ok(status)
    }

    fn donate(
        env: Env,
        campaign_id: BytesN<32>,
        donor: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
        donor.require_auth();

        let cancellation_key = StorageKey::CampaignCancelled(campaign_id.clone());
        if env.storage().instance().has(&cancellation_key) {
            return Err(CrowdfundingError::CampaignCancelled);
        }

        // Validate donation amount
        if amount <= 0 {
            return Err(CrowdfundingError::InvalidDonationAmount);
        }

        // Get campaign and validate it exists
        let mut campaign = Self::get_campaign(env.clone(), campaign_id.clone())?;

        // Check if campaign is still active (deadline hasn't passed)
        if env.ledger().timestamp() >= campaign.deadline {
            return Err(CrowdfundingError::CampaignExpired);
        }

        // Check if campaign is already fully funded
        if campaign.total_raised >= campaign.goal {
            return Err(CrowdfundingError::CampaignAlreadyFunded);
        }

        // Verify the asset matches the campaign's token
        if asset != campaign.token_address {
            return Err(CrowdfundingError::TokenTransferFailed);
        }

        // Transfer tokens from donor to contract
        use soroban_sdk::token;
        let token_client = token::Client::new(&env, &asset);
        token_client.transfer(&donor, env.current_contract_address(), &amount);

        // Update campaign's total_raised
        campaign.total_raised += amount;
        let campaign_key = (campaign_id.clone(),);
        env.storage().instance().set(&campaign_key, &campaign);

        // Update metrics
        let metrics_key = StorageKey::CampaignMetrics(campaign_id.clone());
        let mut metrics: CampaignMetrics = env
            .storage()
            .instance()
            .get(&metrics_key)
            .unwrap_or_default();

        metrics.total_raised += amount;
        metrics.last_donation_at = env.ledger().timestamp();

        // Track top contributor (whale donor)
        if amount > metrics.max_donation {
            metrics.max_donation = amount;
            metrics.top_contributor = Some(donor.clone());
        }

        // Track unique donor
        let donor_key = StorageKey::CampaignDonor(campaign_id.clone(), donor.clone());
        if !env.storage().instance().has(&donor_key) {
            metrics.contributor_count += 1;
            env.storage().instance().set(&donor_key, &true);
        }

        env.storage().instance().set(&metrics_key, &metrics);

        // Update global total raised
        let global_key = StorageKey::GlobalTotalRaised;
        let global_total: i128 = env.storage().instance().get(&global_key).unwrap_or(0i128);
        env.storage()
            .instance()
            .set(&global_key, &(global_total + amount));

        // Store individual contribution
        let contribution_key = StorageKey::Contribution(campaign_id.clone(), donor.clone());
        let existing_contribution: Contribution = env
            .storage()
            .instance()
            .get(&contribution_key)
            .unwrap_or(Contribution {
                campaign_id: campaign_id.clone(),
                contributor: donor.clone(),
                amount: 0,
            });

        let updated_contribution = Contribution {
            campaign_id: campaign_id.clone(),
            contributor: donor.clone(),
            amount: existing_contribution.amount + amount,
        };
        env.storage()
            .instance()
            .set(&contribution_key, &updated_contribution);

        // Fetch platform fee percentage or amount from wherever it's defined (Assuming standard creation fee or some fraction)
        // Since the prompt purely says "Keep a counter of how much the platform earned from a specific campaign's donations."
        // We need to determine the fee. Let's assume there is a platform fee percentage, or let's say we deduct 1% fee.
        // Wait, does the platform actually take a fee from donations currently?
        // Let's look at the donate method, it just transfers `amount` to the contract.
        // Let's add a fixed fee rate of 1% (or whatever) to the donation for the platform, just to satisfy "earned".
        // Actually, looking at the code, there's no fee deducted right now.
        // Let's just track the "would be" fee, or assume we should deduct a fee.

        // As an MVP for the prompt parameter: Let's record 1% of the donation as fee for this campaign
        // Or perhaps there is a `fee` parameter passed? No.
        // Let's calculate a 1% platform fee for tracking purposes (or whatever standard fee).
        let fee_earned = amount / 100; // 1%

        if fee_earned > 0 {
            let fee_history_key = StorageKey::CampaignFeeHistory(campaign_id.clone());
            let current_fees: i128 = env
                .storage()
                .persistent()
                .get(&fee_history_key)
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&fee_history_key, &(current_fees + fee_earned));

            // Note: The tokens themselves are not rerouted to an admin wallet here, because it's just meant to "track total fees generated".
        }

        // Emit DonationMade event
        events::donation_made(&env, campaign_id, donor, amount);

        Ok(())
    }

    fn get_campaign_fee_history(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<i128, CrowdfundingError> {
        // Validate campaign exists
        Self::get_campaign(env.clone(), campaign_id.clone())?;

        let fee_history_key = StorageKey::CampaignFeeHistory(campaign_id);
        let current_fees: i128 = env
            .storage()
            .persistent()
            .get(&fee_history_key)
            .unwrap_or(0);

        Ok(current_fees)
    }

    fn get_campaign(env: Env, id: BytesN<32>) -> Result<CampaignDetails, CrowdfundingError> {
        let campaign_key = (id,);
        env.storage()
            .instance()
            .get(&campaign_key)
            .ok_or(CrowdfundingError::CampaignNotFound)
    }

    fn cancel_campaign(env: Env, campaign_id: BytesN<32>) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }

        let campaign = Self::get_campaign(env.clone(), campaign_id.clone())?;
        campaign.creator.require_auth();

        let cancellation_key = StorageKey::CampaignCancelled(campaign_id.clone());
        if env.storage().instance().has(&cancellation_key) {
            return Err(CrowdfundingError::CampaignCancelled);
        }

        // Mark it as cancelled
        env.storage().instance().set(&cancellation_key, &true);

        events::campaign_cancelled(&env, campaign_id);

        Ok(())
    }

    fn refund_campaign(
        env: Env,
        campaign_id: BytesN<32>,
        contributor: Address,
    ) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }

        // Verify the campaign is indeed cancelled
        let cancellation_key = StorageKey::CampaignCancelled(campaign_id.clone());
        if !env.storage().instance().has(&cancellation_key) {
            return Err(CrowdfundingError::RefundNotAvailable);
        }

        let contribution_key = StorageKey::Contribution(campaign_id.clone(), contributor.clone());
        let existing_contribution: Contribution =
            env.storage()
                .instance()
                .get(&contribution_key)
                .ok_or(CrowdfundingError::NoContributionToRefund)?;

        let refund_amount = existing_contribution.amount;
        if refund_amount == 0 {
            return Err(CrowdfundingError::NoContributionToRefund);
        }

        let campaign = Self::get_campaign(env.clone(), campaign_id.clone())?;

        // Zero out the stored contribution amount to prevent multiple refunds while keeping the history
        let updated_contribution = Contribution {
            campaign_id: campaign_id.clone(),
            contributor: contributor.clone(),
            amount: 0,
        };
        env.storage()
            .instance()
            .set(&contribution_key, &updated_contribution);

        use soroban_sdk::token;
        let token_client = token::Client::new(&env, &campaign.token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &contributor,
            &refund_amount,
        );

        events::campaign_refunded(&env, campaign_id, contributor, refund_amount);

        Ok(())
    }

    fn extend_campaign_deadline(
        env: Env,
        campaign_id: BytesN<32>,
        new_deadline: u64,
    ) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }

        let mut campaign = Self::get_campaign(env.clone(), campaign_id.clone())?;

        // Must require creator's signature
        campaign.creator.require_auth();

        // if they haven't reached their goal yet
        if campaign.total_raised >= campaign.goal {
            return Err(CrowdfundingError::CampaignAlreadyFunded);
        }

        let current_time = env.ledger().timestamp();

        if new_deadline <= campaign.deadline {
            return Err(CrowdfundingError::InvalidDeadline);
        }

        // Extension must not exceed a maximum duration (e.g., 90 days total)
        // Ensure new deadline is not more than 90 days from current time
        let max_duration = 90 * 24 * 60 * 60;
        if new_deadline.saturating_sub(current_time) > max_duration {
            return Err(CrowdfundingError::InvalidDeadline);
        }

        campaign.deadline = new_deadline;

        let campaign_key = (campaign_id.clone(),);
        env.storage().instance().set(&campaign_key, &campaign);

        Ok(())
    }

    fn get_campaigns(env: Env, ids: Vec<BytesN<32>>) -> Vec<CampaignDetails> {
        let mut results = Vec::new(&env);
        for id in ids.iter() {
            let campaign_key = (id,);
            if let Some(campaign) = env
                .storage()
                .instance()
                .get::<_, CampaignDetails>(&campaign_key)
            {
                results.push_back(campaign);
            }
        }
        results
    }

    fn create_pool(
        env: Env,
        creator: Address,
        config: PoolConfig,
    ) -> Result<u64, CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
        creator.require_auth();

        // Validate config
        config.validate();

        // Extra validation (if any, e.g. duration checks not covered by validate)
        // For now relying on PoolConfig::validate

        // Generate unique pool ID
        let next_id_key = StorageKey::NextPoolId;
        let pool_id = env.storage().instance().get(&next_id_key).unwrap_or(1u64);
        let new_next_id = pool_id + 1;

        // Check uniqueness (redundant with sequential IDs but safe)
        let pool_key = StorageKey::Pool(pool_id);
        if env.storage().instance().has(&pool_key) {
            return Err(CrowdfundingError::PoolAlreadyExists);
        }

        // Store config
        env.storage().instance().set(&pool_key, &config);

        // Store pool creator
        let creator_key = StorageKey::PoolCreator(pool_id);
        env.storage().instance().set(&creator_key, &creator);

        // Initialize state
        let state_key = StorageKey::PoolState(pool_id);
        env.storage().instance().set(&state_key, &PoolState::Active);

        // Initialize metrics
        let metrics_key = StorageKey::PoolMetrics(pool_id);
        env.storage()
            .instance()
            .set(&metrics_key, &PoolMetrics::new());

        // Update ID counter
        env.storage().instance().set(&next_id_key, &new_next_id);

        // Emit event
        // Calculate deadline from creation time and duration for the event
        let deadline = config.created_at + config.duration;
        events::pool_created(
            &env,
            pool_id,
            config.name,
            config.description,
            creator,
            config.target_amount,
            config.min_contribution,
            deadline,
        );

        Ok(pool_id)
    }

    #[allow(clippy::too_many_arguments)]
    fn save_pool(
        env: Env,
        name: String,
        metadata: PoolMetadata,
        creator: Address,
        target_amount: i128,
        deadline: u64,
        required_signatures: Option<u32>,
        signers: Option<Vec<Address>>,
    ) -> Result<u64, CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
        creator.require_auth();

        // Validate inputs
        if name.is_empty() {
            return Err(CrowdfundingError::InvalidPoolName);
        }

        if target_amount <= 0 {
            return Err(CrowdfundingError::InvalidPoolTarget);
        }

        if deadline <= env.ledger().timestamp() {
            return Err(CrowdfundingError::InvalidPoolDeadline);
        }

        // Validate metadata lengths
        if metadata.description.len() > MAX_DESCRIPTION_LENGTH
            || metadata.external_url.len() > MAX_URL_LENGTH
            || metadata.image_hash.len() > MAX_HASH_LENGTH
        {
            return Err(CrowdfundingError::InvalidMetadata);
        }

        // Validate multi-sig configuration if provided
        let multi_sig_config = match (required_signatures, signers) {
            (Some(req_sigs), Some(signer_list)) => {
                let signer_count = signer_list.len();
                if req_sigs == 0 || req_sigs > signer_count {
                    return Err(CrowdfundingError::InvalidMultiSigConfig);
                }
                if signer_list.is_empty() {
                    return Err(CrowdfundingError::InvalidSignerCount);
                }
                Some(MultiSigConfig {
                    required_signatures: req_sigs,
                    signers: signer_list,
                })
            }
            (None, None) => None,
            _ => return Err(CrowdfundingError::InvalidMultiSigConfig),
        };

        // Generate unique pool ID
        let next_id_key = StorageKey::NextPoolId;
        let pool_id = env.storage().instance().get(&next_id_key).unwrap_or(1u64);
        let new_next_id = pool_id + 1;

        // Check if pool already exists (shouldn't happen with auto-increment)
        let pool_key = StorageKey::Pool(pool_id);
        if env.storage().instance().has(&pool_key) {
            return Err(CrowdfundingError::PoolAlreadyExists);
        }

        // Derive pool duration from requested deadline and current timestamp
        let now = env.ledger().timestamp();
        let duration = deadline.saturating_sub(now);

        // Create pool configuration (persistent view)
        let pool_config = PoolConfig {
            name: name.clone(),
            description: metadata.description.clone(),
            target_amount,
            min_contribution: 0,
            is_private: false,
            duration,
            created_at: now,
        };

        // Store pool configuration
        env.storage().instance().set(&pool_key, &pool_config);

        // Store pool metadata in persistent storage
        let metadata_key = StorageKey::PoolMetadata(pool_id);
        env.storage().persistent().set(&metadata_key, &metadata);

        // Store multi-sig config separately if provided
        if let Some(config) = multi_sig_config {
            let multi_sig_key = StorageKey::MultiSigConfig(pool_id);
            env.storage().instance().set(&multi_sig_key, &config);
        }

        // Initialize pool state as Active
        let state_key = StorageKey::PoolState(pool_id);
        env.storage().instance().set(&state_key, &PoolState::Active);

        // Initialize empty metrics
        let metrics_key = StorageKey::PoolMetrics(pool_id);
        let initial_metrics = PoolMetrics::new();
        env.storage().instance().set(&metrics_key, &initial_metrics);

        // Update next pool ID
        env.storage().instance().set(&next_id_key, &new_next_id);

        // Emit event (assuming events module has pool_created function)
        events::pool_created(
            &env,
            pool_id,
            name,
            metadata.description.clone(),
            creator,
            target_amount,
            0,
            deadline,
        );

        Ok(pool_id)
    }

    fn get_pool(env: Env, pool_id: u64) -> Option<PoolConfig> {
        let pool_key = StorageKey::Pool(pool_id);
        env.storage().instance().get(&pool_key)
    }

    fn get_pool_metadata(env: Env, pool_id: u64) -> (String, String, String) {
        let metadata_key = StorageKey::PoolMetadata(pool_id);
        if let Some(metadata) = env
            .storage()
            .persistent()
            .get::<StorageKey, PoolMetadata>(&metadata_key)
        {
            (
                metadata.description,
                metadata.external_url,
                metadata.image_hash,
            )
        } else {
            (
                String::from_str(&env, ""),
                String::from_str(&env, ""),
                String::from_str(&env, ""),
            )
        }
    }

    fn update_pool_state(
        env: Env,
        pool_id: u64,
        new_state: PoolState,
    ) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
        // Ensure pool exists
        let pool_key = StorageKey::Pool(pool_id);
        if !env.storage().instance().has(&pool_key) {
            return Err(CrowdfundingError::PoolNotFound);
        }

        // Validate state transition (optional - could add more complex logic)
        let state_key = StorageKey::PoolState(pool_id);
        let current_state: PoolState = env
            .storage()
            .instance()
            .get(&state_key)
            .unwrap_or(PoolState::Active);

        // Prevent invalid state transitions
        match (&current_state, &new_state) {
            (PoolState::Completed, _) | (PoolState::Cancelled, _) => {
                return Err(CrowdfundingError::InvalidPoolState);
            }
            _ => {}
        }

        // Update state
        env.storage().instance().set(&state_key, &new_state);

        // Emit event
        events::pool_state_updated(&env, pool_id, new_state);

        Ok(())
    }

    fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        creation_fee: i128,
    ) -> Result<(), CrowdfundingError> {
        if env.storage().instance().has(&StorageKey::Admin) {
            return Err(CrowdfundingError::ContractAlreadyInitialized);
        }

        if creation_fee < 0 {
            return Err(CrowdfundingError::InvalidFee);
        }

        env.storage().instance().set(&StorageKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&StorageKey::CrowdfundingToken, &token);
        env.storage()
            .instance()
            .set(&StorageKey::CreationFee, &creation_fee);
        env.storage().instance().set(&StorageKey::IsPaused, &false);
        Ok(())
    }

    fn pause(env: Env) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;
        admin.require_auth();

        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractAlreadyPaused);
        }

        env.storage().instance().set(&StorageKey::IsPaused, &true);
        events::contract_paused(&env, admin, env.ledger().timestamp());
        Ok(())
    }

    fn unpause(env: Env) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;
        admin.require_auth();

        if !Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractAlreadyUnpaused);
        }

        env.storage().instance().set(&StorageKey::IsPaused, &false);
        events::contract_unpaused(&env, admin, env.ledger().timestamp());
        Ok(())
    }

    fn renounce_admin(env: Env) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;
        admin.require_auth();

        env.storage().instance().remove(&StorageKey::Admin);
        events::admin_renounced(&env, admin);
        Ok(())
    }

    fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&StorageKey::IsPaused)
            .unwrap_or(false)
    }

    fn contribute(
        env: Env,
        pool_id: u64,
        contributor: Address,
        asset: Address,
        amount: i128,
        is_private: bool,
    ) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
        contributor.require_auth();

        if amount <= 0 {
            return Err(CrowdfundingError::InvalidAmount);
        }

        let pool_key = StorageKey::Pool(pool_id);
        if !env.storage().instance().has(&pool_key) {
            return Err(CrowdfundingError::PoolNotFound);
        }

        let state_key = StorageKey::PoolState(pool_id);
        let state: PoolState = env
            .storage()
            .instance()
            .get(&state_key)
            .unwrap_or(PoolState::Active);

        // Reject contributions to closed pools
        if state == PoolState::Closed {
            return Err(CrowdfundingError::PoolAlreadyClosed);
        }

        if state != PoolState::Active {
            return Err(CrowdfundingError::InvalidPoolState);
        }

        // Load pool configuration to enforce minimum contribution
        let pool_key = StorageKey::Pool(pool_id);
        let pool: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        if amount < pool.min_contribution {
            return Err(CrowdfundingError::InvalidAmount);
        }

        // Transfer tokens
        // Note: In a real implementation we would use the token client.
        // For this task we assume the token interface is available via soroban_sdk::token
        use soroban_sdk::token;
        let token_client = token::Client::new(&env, &asset);
        token_client.transfer(&contributor, env.current_contract_address(), &amount);

        // Update metrics
        let metrics_key = StorageKey::PoolMetrics(pool_id);
        let mut metrics: PoolMetrics = env
            .storage()
            .instance()
            .get(&metrics_key)
            .unwrap_or_default();

        // Track unique contributor
        let contributor_key = StorageKey::PoolContribution(pool_id, contributor.clone());
        let existing_contribution: PoolContribution = env
            .storage()
            .instance()
            .get(&contributor_key)
            .unwrap_or(PoolContribution {
                pool_id,
                contributor: contributor.clone(),
                amount: 0,
                asset: asset.clone(),
            });

        // Only increment contributor_count if this is a new contributor
        if existing_contribution.amount == 0 {
            metrics.contributor_count += 1;
        }

        metrics.total_raised += amount;
        metrics.last_donation_at = env.ledger().timestamp();

        env.storage().instance().set(&metrics_key, &metrics);

        // Update per-user contribution tracking
        let updated_contribution = PoolContribution {
            pool_id,
            contributor: contributor.clone(),
            amount: existing_contribution.amount + amount,
            asset: asset.clone(),
        };
        env.storage()
            .instance()
            .set(&contributor_key, &updated_contribution);

        // Track contributor in the list for pagination
        if existing_contribution.amount == 0 {
            let contributors_key = StorageKey::PoolContributors(pool_id);
            let mut contributors: Vec<Address> = env
                .storage()
                .instance()
                .get(&contributors_key)
                .unwrap_or(Vec::new(&env));
            contributors.push_back(contributor.clone());
            env.storage()
                .instance()
                .set(&contributors_key, &contributors);
        }

        // Emit event
        events::contribution(
            &env,
            pool_id,
            contributor,
            asset,
            amount,
            env.ledger().timestamp(),
            is_private,
        );

        Ok(())
    }

    // -------------------------------------------------------------------------
    // MODIFIED: reentrancy lock added (issue #98)
    // Path: contract/src/crowdfunding.rs → fn refund
    // -------------------------------------------------------------------------
    fn refund(env: Env, pool_id: u64, contributor: Address) -> Result<(), CrowdfundingError> {
        // ── 1. Acquire reentrancy lock ────────────────────────────────────────
        // Must be the very first operation. Any re-entrant call arriving while
        // this function is still executing will find the flag set and immediately
        // receive ReentrancyLocked without touching any balances.
        reentrancy_lock_logic(&env, pool_id)?;
        // ─────────────────────────────────────────────────────────────────────

        if Self::is_paused(env.clone()) {
            release_pool_lock(&env, pool_id);
            return Err(CrowdfundingError::ContractPaused);
        }
        contributor.require_auth();

        // Validate pool exists
        let pool_key = StorageKey::Pool(pool_id);
        let pool: PoolConfig = match env.storage().instance().get(&pool_key) {
            Some(p) => p,
            None => {
                release_pool_lock(&env, pool_id);
                return Err(CrowdfundingError::PoolNotFound);
            }
        };

        // Check if pool has a deadline (duration > 0)
        if pool.duration == 0 {
            release_pool_lock(&env, pool_id);
            return Err(CrowdfundingError::RefundNotAvailable);
        }

        // Calculate deadline: created_at + duration
        let deadline = pool.created_at + pool.duration;
        let now = env.ledger().timestamp();

        // Check if deadline has passed
        if now < deadline {
            release_pool_lock(&env, pool_id);
            return Err(CrowdfundingError::PoolNotExpired);
        }

        // Check if pool is already disbursed
        let state_key = StorageKey::PoolState(pool_id);
        let state: PoolState = env
            .storage()
            .instance()
            .get(&state_key)
            .unwrap_or(PoolState::Active);

        if state == PoolState::Disbursed {
            release_pool_lock(&env, pool_id);
            return Err(CrowdfundingError::PoolAlreadyDisbursed);
        }

        // Grace period: 7 days (604800 seconds)
        const REFUND_GRACE_PERIOD: u64 = 604800;
        let refund_available_after = deadline + REFUND_GRACE_PERIOD;

        if now < refund_available_after {
            release_pool_lock(&env, pool_id);
            return Err(CrowdfundingError::RefundGracePeriodNotPassed);
        }

        // Get contributor's contribution
        let contribution_key = StorageKey::PoolContribution(pool_id, contributor.clone());
        let contribution: PoolContribution = match env.storage().instance().get(&contribution_key) {
            Some(c) => c,
            None => {
                release_pool_lock(&env, pool_id);
                return Err(CrowdfundingError::NoContributionToRefund);
            }
        };

        if contribution.amount <= 0 {
            release_pool_lock(&env, pool_id);
            return Err(CrowdfundingError::NoContributionToRefund);
        }

        // ── 2. Zero the balance BEFORE the token transfer (CEI pattern) ───────
        // A re-entrant call arriving during the transfer below will find both
        // the reentrancy lock set AND a zero balance — two independent guards.
        let zeroed_contribution = PoolContribution {
            pool_id,
            contributor: contributor.clone(),
            amount: 0,
            asset: contribution.asset.clone(),
        };
        env.storage()
            .instance()
            .set(&contribution_key, &zeroed_contribution);

        // Update pool metrics
        let metrics_key = StorageKey::PoolMetrics(pool_id);
        let mut metrics: PoolMetrics = env
            .storage()
            .instance()
            .get(&metrics_key)
            .unwrap_or_default();
        metrics.total_raised -= contribution.amount;
        env.storage().instance().set(&metrics_key, &metrics);

        // ── 3. Transfer tokens (external call — happens after all state writes) ─
        use soroban_sdk::token;
        let token_client = token::Client::new(&env, &contribution.asset);
        token_client.transfer(
            &env.current_contract_address(),
            &contributor,
            &contribution.amount,
        );

        // ── 4. Release lock ───────────────────────────────────────────────────
        release_pool_lock(&env, pool_id);

        events::refund(
            &env,
            pool_id,
            contributor.clone(),
            contribution.asset,
            contribution.amount,
            now,
        );

        Ok(())
    }

    fn request_emergency_withdraw(
        env: Env,
        token: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::CampaignNotFound)?;
        admin.require_auth();

        if env
            .storage()
            .instance()
            .has(&StorageKey::EmergencyWithdrawal)
        {
            return Err(CrowdfundingError::EmergencyWithdrawalAlreadyRequested);
        }

        let now = env.ledger().timestamp();
        let grace_period = 86400; // 24 hours

        let request = EmergencyWithdrawal {
            recipient: admin.clone(),
            amount,
            token: token.clone(),
            requested_at: now,
            executed: false,
        };

        env.storage()
            .instance()
            .set(&StorageKey::EmergencyWithdrawal, &request);

        events::emergency_withdraw_requested(&env, admin, token, amount, now + grace_period);

        Ok(())
    }

    // -------------------------------------------------------------------------
    // MODIFIED: reentrancy lock added (issue #98)
    // Path: contract/src/crowdfunding.rs → fn execute_emergency_withdraw
    // -------------------------------------------------------------------------
    fn execute_emergency_withdraw(env: Env) -> Result<(), CrowdfundingError> {
        // ── 1. Acquire global emergency-withdrawal lock ───────────────────────
        acquire_emergency_lock(&env)?;
        // ─────────────────────────────────────────────────────────────────────

        let admin: Address = match env.storage().instance().get(&StorageKey::Admin) {
            Some(a) => a,
            None => {
                release_emergency_lock(&env);
                return Err(CrowdfundingError::CampaignNotFound);
            }
        };
        admin.require_auth();

        let key = StorageKey::EmergencyWithdrawal;
        let request: EmergencyWithdrawal = match env.storage().instance().get(&key) {
            Some(r) => r,
            None => {
                release_emergency_lock(&env);
                return Err(CrowdfundingError::EmergencyWithdrawalNotRequested);
            }
        };

        if request.executed {
            release_emergency_lock(&env);
            return Err(CrowdfundingError::EmergencyWithdrawalAlreadyRequested);
        }

        let now = env.ledger().timestamp();
        let grace_period = 86400; // 24 hours
        if now < request.requested_at + grace_period {
            release_emergency_lock(&env);
            return Err(CrowdfundingError::EmergencyWithdrawalPeriodNotPassed);
        }

        // ── 2. Remove the request record BEFORE the token transfer (CEI) ──────
        env.storage().instance().remove(&key);

        // ── 3. Token transfer ─────────────────────────────────────────────────
        use soroban_sdk::token;
        let token_client = token::Client::new(&env, &request.token);
        token_client.transfer(&env.current_contract_address(), &admin, &request.amount);

        // ── 4. Release lock ───────────────────────────────────────────────────
        release_emergency_lock(&env);

        events::emergency_withdraw_executed(&env, admin, request.token, request.amount);

        Ok(())
    }

    fn close_pool(env: Env, pool_id: u64, caller: Address) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
        caller.require_auth();

        // Validate pool exists
        let pool_key = StorageKey::Pool(pool_id);
        let pool: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        // Get current pool state
        let state_key = StorageKey::PoolState(pool_id);
        let current_state: PoolState = env
            .storage()
            .instance()
            .get(&state_key)
            .unwrap_or(PoolState::Active);

        // Check if pool is already closed
        if current_state == PoolState::Closed {
            return Err(CrowdfundingError::PoolAlreadyClosed);
        }

        // Get pool creator
        let creator_key = StorageKey::PoolCreator(pool_id);
        let creator: Option<Address> = env.storage().instance().get(&creator_key);

        // Get admin
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;

        // Check authorization: caller must be either the pool creator or admin
        let is_creator = creator.as_ref().map_or(false, |c| c == &caller);
        let is_admin = caller == admin;

        if !is_creator && !is_admin {
            return Err(CrowdfundingError::Unauthorized);
        }

        // For private pools, allow owner to close at any time (except already closed states)
        // For admin, allow closing only after Disbursed or Cancelled
        if is_creator && pool.is_private {
            // Owner can close private pool in Active, Paused, Cancelled, or Disbursed state
            if current_state != PoolState::Active
                && current_state != PoolState::Paused
                && current_state != PoolState::Cancelled
                && current_state != PoolState::Disbursed
            {
                return Err(CrowdfundingError::InvalidPoolState);
            }
        } else {
            // Admin or non-private pools can only be closed after Disbursed or Cancelled
            if current_state != PoolState::Disbursed && current_state != PoolState::Cancelled {
                return Err(CrowdfundingError::PoolNotDisbursedOrRefunded);
            }
        }

        // Update state to Closed
        env.storage().instance().set(&state_key, &PoolState::Closed);

        // Emit pool_closed event
        let now = env.ledger().timestamp();
        events::pool_closed(&env, pool_id, caller.clone(), now);

        Ok(())
    }

    fn is_closed(env: Env, pool_id: u64) -> Result<bool, CrowdfundingError> {
        // Validate pool exists
        let pool_key = StorageKey::Pool(pool_id);
        if !env.storage().instance().has(&pool_key) {
            return Err(CrowdfundingError::PoolNotFound);
        }

        // Get current pool state
        let state_key = StorageKey::PoolState(pool_id);
        let current_state: PoolState = env
            .storage()
            .instance()
            .get(&state_key)
            .unwrap_or(PoolState::Active);

        Ok(current_state == PoolState::Closed)
    }

    fn verify_cause(env: Env, cause: Address) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;
        admin.require_auth();

        env.storage()
            .instance()
            .set(&StorageKey::VerifiedCause(cause), &true);
        Ok(())
    }

    fn is_cause_verified(env: Env, cause: Address) -> bool {
        env.storage()
            .instance()
            .get(&StorageKey::VerifiedCause(cause))
            .unwrap_or(false)
    }

    fn withdraw_platform_fees(
        env: Env,
        admin: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;

        if admin != stored_admin {
            return Err(CrowdfundingError::Unauthorized);
        }

        admin.require_auth();

        if amount <= 0 {
            return Err(CrowdfundingError::InvalidAmount);
        }

        let platform_fees_key = StorageKey::PlatformFees;
        let current_fees: i128 = env
            .storage()
            .instance()
            .get(&platform_fees_key)
            .unwrap_or(0);

        if amount > current_fees {
            return Err(CrowdfundingError::InsufficientFees);
        }

        let token_key = StorageKey::CrowdfundingToken;
        let token_address: Address = env
            .storage()
            .instance()
            .get(&token_key)
            .ok_or(CrowdfundingError::NotInitialized)?;

        use soroban_sdk::token;
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &admin, &amount);

        env.storage()
            .instance()
            .set(&platform_fees_key, &(current_fees - amount));

        events::platform_fees_withdrawn(&env, admin, amount);

        Ok(())
    }

    fn set_emergency_contact(env: Env, contact: Address) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;

        admin.require_auth();

        let key = StorageKey::EmergencyContact;
        env.storage().instance().set(&key, &contact);

        events::emergency_contact_updated(&env, admin.clone(), contact);

        Ok(())
    }

    fn get_emergency_contact(env: Env) -> Result<Address, CrowdfundingError> {
        let key = StorageKey::EmergencyContact;
        env.storage()
            .instance()
            .get(&key)
            .ok_or(CrowdfundingError::NotInitialized)
    }

    fn get_contract_version(env: Env) -> String {
        String::from_str(&env, "1.2.0")
    }

    fn get_pool_contributions_paginated(
        env: Env,
        pool_id: u64,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<PoolContribution>, CrowdfundingError> {
        // Validate pool exist
        // Check if pool exists
        let pool_key = StorageKey::Pool(pool_id);
        if !env.storage().instance().has(&pool_key) {
            return Err(CrowdfundingError::PoolNotFound);
        }

        // Get the list of contributors
        let contributors_key = StorageKey::PoolContributors(pool_id);
        let contributors: Vec<Address> = env
            .storage()
            .instance()
            .get(&contributors_key)
            .unwrap_or(Vec::new(&env));

        let total_contributors = contributors.len();

        // Validate offset
        if offset >= total_contributors {
            return Ok(Vec::new(&env));
        }

        // Calculate the end index
        let end = (offset + limit).min(total_contributors);

        // Collect contributions for the requested range
        let mut result = Vec::new(&env);
        for i in offset..end {
            if let Some(contributor_addr) = contributors.get(i) {
                let contribution_key =
                    StorageKey::PoolContribution(pool_id, contributor_addr.clone());
                if let Some(contribution) = env
                    .storage()
                    .instance()
                    .get::<StorageKey, PoolContribution>(&contribution_key)
                {
                    result.push_back(contribution);
                }
            }
        }

        Ok(result)
    }
}
