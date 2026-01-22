use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String};

use crate::base::{
    errors::CrowdfundingError,
    events,
    types::{CampaignDetails, PoolConfig, PoolMetrics, PoolState, StorageKey},
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
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
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
    ) -> Result<u64, CrowdfundingError> {
        if Self::is_paused(env.clone()) {
            return Err(CrowdfundingError::ContractPaused);
        }
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
            description: description.clone(),
            target_amount,
            is_private: false,
            duration,
            created_at: now,
        };

        // Store pool configuration
        env.storage().instance().set(&pool_key, &pool_config);

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

    fn initialize(env: Env, admin: Address) -> Result<(), CrowdfundingError> {
        if env.storage().instance().has(&StorageKey::Admin) {
            return Err(CrowdfundingError::ContractAlreadyInitialized);
        }
        env.storage().instance().set(&StorageKey::Admin, &admin);
        env.storage().instance().set(&StorageKey::IsPaused, &false);
        Ok(())
    }

    fn pause(env: Env) -> Result<(), CrowdfundingError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&StorageKey::Admin)
            .ok_or(CrowdfundingError::CampaignNotFound)?; // Or some other error if not initialized
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
            .ok_or(CrowdfundingError::CampaignNotFound)?;
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
        token_client.transfer(&contributor, &env.current_contract_address(), &amount);

        // Update metrics
        let metrics_key = StorageKey::PoolMetrics(pool_id);
        let mut metrics: PoolMetrics = env
            .storage()
            .instance()
            .get(&metrics_key)
            .unwrap_or(PoolMetrics::new());

        metrics.total_raised += amount;
        metrics.contributor_count += 1;
        metrics.last_donation_at = env.ledger().timestamp();

        env.storage().instance().set(&metrics_key, &metrics);

        // Emit event
        let topics = (soroban_sdk::Symbol::new(&env, "contribution"), pool_id);
        env.events().publish(
            topics,
            (contributor, asset, amount, env.ledger().timestamp(), is_private),
        );

        Ok(())
    }
}
