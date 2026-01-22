use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

use crate::base::{
    errors::CrowdfundingError,
    events,
    types::{CampaignDetails, DisbursementRequest, MultiSigConfig, PoolConfig, PoolMetrics, PoolState, StorageKey},
};
use crate::interfaces::crowdfunding::CrowdfundingTrait;

#[contract]
pub struct CrowdfundingContract;

#[contractimpl]
impl CrowdfundingTrait for CrowdfundingContract {
    fn create_campaign(
        env: Env,
        id: BytesN<32>,
        title: String,
        creator: Address,
        goal: i128,
        deadline: u64,
    ) -> Result<(), CrowdfundingError> {
        creator.require_auth();

        if title.len() == 0 {
            return Err(CrowdfundingError::InvalidTitle);
        }

        if goal <= 0 {
            return Err(CrowdfundingError::InvalidGoal);
        }

        if deadline <= env.ledger().timestamp() {
            return Err(CrowdfundingError::InvalidDeadline);
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
        };

        env.storage().instance().set(&campaign_key, &campaign);

        events::campaign_created(&env, id, title, creator, goal, deadline);

        Ok(())
    }

    fn get_campaign(env: Env, id: BytesN<32>) -> Result<CampaignDetails, CrowdfundingError> {
        let campaign_key = (id,);
        env.storage()
            .instance()
            .get(&campaign_key)
            .ok_or(CrowdfundingError::CampaignNotFound)
    }

    fn save_pool(
        env: Env,
        name: String,
        description: String,
        creator: Address,
        target_amount: i128,
        deadline: u64,
        required_signatures: Option<u32>,
        signers: Option<Vec<Address>>,
    ) -> Result<u64, CrowdfundingError> {
        creator.require_auth();

        // Validate inputs
        if name.len() == 0 {
            return Err(CrowdfundingError::InvalidPoolName);
        }

        if target_amount <= 0 {
            return Err(CrowdfundingError::InvalidPoolTarget);
        }

        if deadline <= env.ledger().timestamp() {
            return Err(CrowdfundingError::InvalidPoolDeadline);
        }

        // Validate multi-sig configuration if provided
        let multi_sig_config = match (required_signatures, signers) {
            (Some(req_sigs), Some(signer_list)) => {
                let signer_count = signer_list.len() as u32;
                if req_sigs == 0 || req_sigs > signer_count {
                    return Err(CrowdfundingError::InvalidMultiSigConfig);
                }
                if signer_list.len() == 0 {
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

        // Create pool configuration
        let pool_config = PoolConfig {
            id: pool_id,
            name: name.clone(),
            description: description.clone(),
            creator: creator.clone(),
            target_amount,
            deadline,
            created_at: env.ledger().timestamp(),
            multi_sig_config,
        };

        // Store pool configuration
        env.storage().instance().set(&pool_key, &pool_config);

        // Initialize pool state as Active
        let state_key = StorageKey::PoolState(pool_id);
        env.storage().instance().set(&state_key, &PoolState::Active);

        // Initialize empty metrics
        let metrics_key = StorageKey::PoolMetrics(pool_id);
        let initial_metrics = PoolMetrics {
            total_donations: 0,
            donor_count: 0,
            last_donation_at: 0,
        };
        env.storage().instance().set(&metrics_key, &initial_metrics);

        // Update next pool ID
        env.storage().instance().set(&next_id_key, &new_next_id);

        // Emit event (assuming events module has pool_created function)
        events::pool_created(
            &env,
            pool_id,
            name,
            description,
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

    fn update_pool_state(
        env: Env,
        pool_id: u64,
        new_state: PoolState,
    ) -> Result<(), CrowdfundingError> {
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

    fn request_disbursement(
        env: Env,
        pool_id: u64,
        amount: i128,
        recipient: Address,
        requester: Address,
    ) -> Result<u64, CrowdfundingError> {
        requester.require_auth();

        // Get pool config
        let pool_key = StorageKey::Pool(pool_id);
        let pool_config: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        // Verify requester is either creator or a signer
        let is_authorized = if let Some(ref multi_sig) = pool_config.multi_sig_config {
            pool_config.creator == requester || multi_sig.signers.contains(&requester)
        } else {
            pool_config.creator == requester
        };

        if !is_authorized {
            return Err(CrowdfundingError::NotAuthorizedSigner);
        }

        // Generate disbursement ID
        let next_disb_key = StorageKey::NextDisbursementId(pool_id);
        let disbursement_id = env.storage().instance().get(&next_disb_key).unwrap_or(1u64);

        // Create disbursement request
        let disbursement = DisbursementRequest {
            pool_id,
            amount,
            recipient: recipient.clone(),
            approvals: Vec::new(&env),
            created_at: env.ledger().timestamp(),
            executed: false,
        };

        // Store disbursement
        let disb_key = StorageKey::DisbursementRequest(pool_id, disbursement_id);
        env.storage().instance().set(&disb_key, &disbursement);
        env.storage()
            .instance()
            .set(&next_disb_key, &(disbursement_id + 1));

        Ok(disbursement_id)
    }

    fn approve_disbursement(
        env: Env,
        pool_id: u64,
        disbursement_id: u64,
        signer: Address,
    ) -> Result<(), CrowdfundingError> {
        signer.require_auth();

        // Get pool config
        let pool_key = StorageKey::Pool(pool_id);
        let pool_config: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        // Verify signer is authorized
        if let Some(ref multi_sig) = pool_config.multi_sig_config {
            if !multi_sig.signers.contains(&signer) && pool_config.creator != signer {
                return Err(CrowdfundingError::NotAuthorizedSigner);
            }
        } else {
            if pool_config.creator != signer {
                return Err(CrowdfundingError::NotAuthorizedSigner);
            }
        }

        // Get disbursement request
        let disb_key = StorageKey::DisbursementRequest(pool_id, disbursement_id);
        let mut disbursement: DisbursementRequest = env
            .storage()
            .instance()
            .get(&disb_key)
            .ok_or(CrowdfundingError::DisbursementNotFound)?;

        if disbursement.executed {
            return Err(CrowdfundingError::DisbursementAlreadyExecuted);
        }

        // Check if already approved
        if disbursement.approvals.contains(&signer) {
            return Err(CrowdfundingError::AlreadyApproved);
        }

        // Add approval
        disbursement.approvals.push_back(signer);
        env.storage().instance().set(&disb_key, &disbursement);

        Ok(())
    }

    fn execute_disbursement(
        env: Env,
        pool_id: u64,
        disbursement_id: u64,
    ) -> Result<(), CrowdfundingError> {
        // Get pool config
        let pool_key = StorageKey::Pool(pool_id);
        let pool_config: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        // Get disbursement request
        let disb_key = StorageKey::DisbursementRequest(pool_id, disbursement_id);
        let mut disbursement: DisbursementRequest = env
            .storage()
            .instance()
            .get(&disb_key)
            .ok_or(CrowdfundingError::DisbursementNotFound)?;

        if disbursement.executed {
            return Err(CrowdfundingError::DisbursementAlreadyExecuted);
        }

        // Check if sufficient approvals
        let required_approvals = if let Some(ref multi_sig) = pool_config.multi_sig_config {
            multi_sig.required_signatures
        } else {
            1
        };

        let approval_count = disbursement.approvals.len() as u32;
        if approval_count < required_approvals {
            return Err(CrowdfundingError::InsufficientApprovals);
        }

        // Mark as executed
        disbursement.executed = true;
        env.storage().instance().set(&disb_key, &disbursement);

        // Here you would implement actual token transfer logic
        // For now, we just mark it as executed

        Ok(())
    }

    fn add_signer(
        env: Env,
        pool_id: u64,
        new_signer: Address,
        caller: Address,
    ) -> Result<(), CrowdfundingError> {
        caller.require_auth();

        // Get pool config
        let pool_key = StorageKey::Pool(pool_id);
        let mut pool_config: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        // Only creator can add signers
        if pool_config.creator != caller {
            return Err(CrowdfundingError::NotAuthorizedSigner);
        }

        // Initialize multi-sig if not present
        if pool_config.multi_sig_config.is_none() {
            let mut signers = Vec::new(&env);
            signers.push_back(new_signer);
            pool_config.multi_sig_config = Some(MultiSigConfig {
                required_signatures: 1,
                signers,
            });
        } else {
            let mut multi_sig = pool_config.multi_sig_config.unwrap();
            
            if multi_sig.signers.contains(&new_signer) {
                return Err(CrowdfundingError::SignerAlreadyExists);
            }

            multi_sig.signers.push_back(new_signer);
            pool_config.multi_sig_config = Some(multi_sig);
        }

        env.storage().instance().set(&pool_key, &pool_config);

        Ok(())
    }

    fn remove_signer(
        env: Env,
        pool_id: u64,
        signer_to_remove: Address,
        caller: Address,
    ) -> Result<(), CrowdfundingError> {
        caller.require_auth();

        // Get pool config
        let pool_key = StorageKey::Pool(pool_id);
        let mut pool_config: PoolConfig = env
            .storage()
            .instance()
            .get(&pool_key)
            .ok_or(CrowdfundingError::PoolNotFound)?;

        // Only creator can remove signers
        if pool_config.creator != caller {
            return Err(CrowdfundingError::NotAuthorizedSigner);
        }

        let multi_sig = pool_config
            .multi_sig_config
            .as_mut()
            .ok_or(CrowdfundingError::InvalidMultiSigConfig)?;

        // Find and remove signer
        let initial_len = multi_sig.signers.len();
        let mut new_signers = Vec::new(&env);
        
        for i in 0..multi_sig.signers.len() {
            let signer = multi_sig.signers.get(i).unwrap();
            if signer != signer_to_remove {
                new_signers.push_back(signer);
            }
        }

        if new_signers.len() == initial_len {
            return Err(CrowdfundingError::SignerNotFound);
        }

        if new_signers.len() == 0 {
            return Err(CrowdfundingError::CannotRemoveLastSigner);
        }

        // Adjust required signatures if needed
        let new_signer_count = new_signers.len() as u32;
        if multi_sig.required_signatures > new_signer_count {
            multi_sig.required_signatures = new_signer_count;
        }

        multi_sig.signers = new_signers;
        pool_config.multi_sig_config = Some(multi_sig.clone());
        
        env.storage().instance().set(&pool_key, &pool_config);

        Ok(())
    }

    fn get_disbursement(
        env: Env,
        pool_id: u64,
        disbursement_id: u64,
    ) -> Option<DisbursementRequest> {
        let disb_key = StorageKey::DisbursementRequest(pool_id, disbursement_id);
        env.storage().instance().get(&disb_key)
    }
}
