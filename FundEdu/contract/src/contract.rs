use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate::{
    storage::{get_pool, next_pool_id, set_pool},
    types::ScholarshipPool,
};

#[contract]
pub struct FundEduContract;

#[contractimpl]
impl FundEduContract {
    /// Create a new scholarship pool and return its assigned pool_id.
    pub fn create_pool(
        env: Env,
        sponsor: Address,
        name: String,
        target_amount: i128,
        token_address: Address,
    ) -> u64 {
        sponsor.require_auth();

        let pool_id = next_pool_id(&env);
        let pool = ScholarshipPool {
            name,
            sponsor,
            target_amount,
            token_address,
            is_active: true,
        };
        set_pool(&env, pool_id, &pool);
        pool_id
    }

    /// Retrieve a scholarship pool by its id. Returns None if not found.
    pub fn get_pool(env: Env, pool_id: u64) -> Option<ScholarshipPool> {
        get_pool(&env, pool_id)
    }
}
