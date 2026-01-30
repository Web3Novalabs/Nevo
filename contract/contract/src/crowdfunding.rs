#![allow(deprecated)]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

use crate::base::{
    errors::CrowdfundingError,
    events,
    types::{
        CampaignDetails, CampaignMetrics, Contribution, EmergencyWithdrawal, MultiSigConfig,
        PoolConfig, PoolContribution, PoolMetadata, PoolMetrics, PoolState, StorageKey,
        MAX_DESCRIPTION_LENGTH, MAX_HASH_LENGTH, MAX_URL_LENGTH,
    },
};
use crate::interfaces::crowdfunding::CrowdfundingTrait;

#[contract]
pub struct CrowdfundingContract;

#[contractimpl]
#[allow(clippy::too_many_arguments)]
impl CrowdfundingTrait for CrowdfundingContract {
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

    fn get_all_campaigns(env: Env) -> Vec<BytesN<32>> {
        env.storage()
            .instance()
            .get(&StorageKey::AllCampaigns)
            .unwrap_or(Vec::new(&env))
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

        // Track unique donor
        let donor_key = StorageKey::CampaignDonor(campaign_id.clone(), donor.clone());
        if !env.storage().instance().has(&donor_key) {
            metrics.contributor_count += 1;
            env.storage().instance().set(&donor_key, &true);
        }

        env.storage().instance().set(&metrics_key, &metrics);

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

        // Emit DonationMade event
        events::donation_made(&env, campaign_id, donor, amount);

        Ok(())
    }

    fn get_campaign(env: Env, id: BytesN<32>) -> Result<CampaignDetails, CrowdfundingError> {
        let campaign_key = (id,);
        env.storage()
            .instance()
            .get(&campaign_key)
            .ok_or(CrowdfundingError::CampaignNotFound)
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
            _ => {} // Allow other transitions
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

        if state != PoolState::Active {
            return Err(CrowdfundingError::InvalidPoolState);
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

    fn refund(env: Env, pool_id: u64, contributor: Address) -> Result<(), CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
        contributor.require_auth();

        // Validate pool exists
        let pool_key = StorageKey::Pool(pool_id);
        let pool: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        // Check if pool has a deadline (duration > 0)
        if pool.duration == 0 {
            return Err(CrowdfundingError::RefundNotAvailable);
        }

        // Calculate deadline: created_at + duration
        let deadline = pool.created_at + pool.duration;
        let now = env.ledger().timestamp();

        // Check if deadline has passed
        if now < deadline {
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
            return Err(CrowdfundingError::PoolAlreadyDisbursed);
        }

        // Grace period: 7 days (604800 seconds)
        const REFUND_GRACE_PERIOD: u64 = 604800;
        let refund_available_after = deadline + REFUND_GRACE_PERIOD;

        // Check if grace period has passed
        if now < refund_available_after {
            return Err(CrowdfundingError::RefundGracePeriodNotPassed);
        }

        // Get contributor's contribution
        let contribution_key = StorageKey::PoolContribution(pool_id, contributor.clone());
        let contribution: PoolContribution = env
            .storage()
            .instance()
            .get(&contribution_key)
            .ok_or(CrowdfundingError::NoContributionToRefund)?;

        if contribution.amount <= 0 {
            return Err(CrowdfundingError::NoContributionToRefund);
        }

        // Transfer tokens back to contributor
        use soroban_sdk::token;
        let token_client = token::Client::new(&env, &contribution.asset);
        token_client.transfer(
            &env.current_contract_address(),
            &contributor,
            &contribution.amount,
        );

        // Update pool metrics
        let metrics_key = StorageKey::PoolMetrics(pool_id);
        let mut metrics: PoolMetrics = env
            .storage()
            .instance()
            .get(&metrics_key)
            .unwrap_or(PoolMetrics::new());

        metrics.total_raised -= contribution.amount;
        // Note: We don't decrement contributor_count as the contributor may have other contributions
        // or we want to keep historical data

        env.storage().instance().set(&metrics_key, &metrics);

        // Remove or zero out the contribution record
        // We zero it out to prevent double refunds while keeping historical record
        let zeroed_contribution = PoolContribution {
            pool_id,
            contributor: contributor.clone(),
            amount: 0,
            asset: contribution.asset.clone(),
        };
        env.storage()
            .instance()
            .set(&contribution_key, &zeroed_contribution);

        // Emit refund event
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

    fn execute_emergency_withdraw(env: Env) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::CampaignNotFound)?;
        admin.require_auth();

        let key = StorageKey::EmergencyWithdrawal;
        let request: EmergencyWithdrawal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(CrowdfundingError::EmergencyWithdrawalNotRequested)?;

        // If for some reason it's already executed but not removed (shouldn't happen with remove)
        if request.executed {
            return Err(CrowdfundingError::EmergencyWithdrawalAlreadyRequested);
        }

        let now = env.ledger().timestamp();
        let grace_period = 86400; // 24 hours
        if now < request.requested_at + grace_period {
            return Err(CrowdfundingError::EmergencyWithdrawalPeriodNotPassed);
        }

        use soroban_sdk::token;
        let token_client = token::Client::new(&env, &request.token);
        token_client.transfer(&env.current_contract_address(), &admin, &request.amount);

        // Remove the request to allow future requests (or keep it as history? Requirement says "Define clear rules in storage to prevent abuse".
        // Removing it clears the storage. If we want history, we should use a map or log events.
        // Events are logged. Clearing storage prevents double withdrawal and clutter.
        env.storage().instance().remove(&key);

        events::emergency_withdraw_executed(&env, admin, request.token, request.amount);

        Ok(())
    }

    fn close_pool(env: Env, pool_id: u64, caller: Address) -> Result<(), CrowdfundingError> {
        caller.require_auth();

        // Validate pool exists
        let pool_key = StorageKey::Pool(pool_id);
        let _pool: PoolConfig = env
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

        // Only allow closing if pool is in Disbursed or Cancelled state
        if current_state != PoolState::Disbursed && current_state != PoolState::Cancelled {
            return Err(CrowdfundingError::PoolNotDisbursedOrRefunded);
        }

        // Verify caller is admin or pool creator
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::NotInitialized)?;

        // For now, we'll check if there's a creator stored separately
        // Since PoolConfig doesn't have creator field, we'll allow admin only
        // In a real implementation, you might want to add creator to PoolConfig or store it separately
        if caller != admin {
            return Err(CrowdfundingError::Unauthorized);
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
}
