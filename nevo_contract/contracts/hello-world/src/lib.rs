#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};

// Storage key constants
const POOL_COUNT: &str = "pool_count";
const POOL_PREFIX: &str = "p";
const CREATOR_SUFFIX: &str = "_creator";
const GOAL_SUFFIX: &str = "_goal";
const COLLECTED_SUFFIX: &str = "_collected";
const CLOSED_SUFFIX: &str = "_closed";
const APPLICATION_COUNT_PREFIX: &str = "a_count_";
const APPLICATION_PREFIX: &str = "a_";
const APPLICANT_PREFIX: &str = "ap_";

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Create a new donation pool
    pub fn create_pool(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: u128,
    ) -> u32 {
        // creator.require_auth();  // TODO: Enable auth validation in production

        // Get the next pool ID
        let pool_count_key = Symbol::new(&env, POOL_COUNT);
        let mut pool_count: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&pool_count_key)
            .unwrap_or(0);

        let pool_id = pool_count + 1;
        pool_count = pool_id;

        // Store pool data - using numeric pool ID as key
        let pool_key = pool_id;

        env.storage()
            .persistent()
            .set(&pool_key, &(creator.clone(), goal, 0u128, false));

        env.storage().persistent().set(&pool_count_key, &pool_count);

        pool_id
    }

    /// Donate to an existing pool
    pub fn donate(env: Env, pool_id: u32, donor: Address, amount: u128) {
        // donor.require_auth();  // TODO: Enable auth validation in production

        let pool_key = pool_id;
        let pool_data: (Address, u128, u128, bool) = env
            .storage()
            .persistent()
            .get::<_, (Address, u128, u128, bool)>(&pool_key)
            .expect("Pool not found");

        if pool_data.3 {
            panic!("Pool is closed");
        }

        // Update pool balance
        let new_collected = pool_data.2 + amount;
        env.storage().persistent().set(
            &pool_key,
            &(pool_data.0.clone(), pool_data.1, new_collected, pool_data.3),
        );

        // Record the donation (using a simple counter approach)
        let donor_index: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&(pool_id, "d_count"))
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&(pool_id, "d_count"), &(donor_index + 1));
    }

    /// Get pool information as a tuple (id, creator, goal, collected, is_closed)
    pub fn get_pool(env: Env, pool_id: u32) -> (u32, Address, u128, u128, bool) {
        let pool_key = pool_id;
        let pool_data: (Address, u128, u128, bool) = env
            .storage()
            .persistent()
            .get::<_, (Address, u128, u128, bool)>(&pool_key)
            .expect("Pool not found");

        (pool_id, pool_data.0, pool_data.1, pool_data.2, pool_data.3)
    }

    /// Close a donation pool
    pub fn close_pool(env: Env, pool_id: u32) {
        let pool_key = pool_id;
        let pool_data: (Address, u128, u128, bool) = env
            .storage()
            .persistent()
            .get::<_, (Address, u128, u128, bool)>(&pool_key)
            .expect("Pool not found");

        // pool_data.0.require_auth();  // TODO: Enable auth validation in production

        env.storage()
            .persistent()
            .set(&pool_key, &(pool_data.0, pool_data.1, pool_data.2, true));
    }

    /// Get the total number of pools
    pub fn get_pool_count(env: Env) -> u32 {
        let pool_count_key = Symbol::new(&env, POOL_COUNT);
        env.storage()
            .persistent()
            .get::<_, u32>(&pool_count_key)
            .unwrap_or(0)
    }

    /// Apply for a scholarship in a pool
    pub fn apply_for_scholarship(
        env: Env,
        pool_id: u32,
        student: Address,
        application_data: String,
    ) -> (u32, Address, String) {
        // student.require_auth();  // TODO: Enable auth validation in production

        // Check if pool exists
        let pool_key = pool_id;
        let _pool_data: (Address, u128, u128, bool) = env
            .storage()
            .persistent()
            .get::<_, (Address, u128, u128, bool)>(&pool_key)
            .expect("Pool not found");

        // Check if already applied
        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if env.storage().persistent().has(&applicant_key) {
            panic!("Duplicate application");
        }

        // Get next application id for this pool
        let count_key = (Symbol::new(&env, APPLICATION_COUNT_PREFIX), pool_id);
        let mut app_count: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&count_key)
            .unwrap_or(0);
        app_count += 1;

        // Store application
        let app_key = (Symbol::new(&env, APPLICATION_PREFIX), pool_id, app_count);
        env.storage().persistent().set(
            &app_key,
            &(app_count, student.clone(), application_data.clone()),
        );

        // Mark as applied
        env.storage().persistent().set(&applicant_key, &true);

        // Update count
        env.storage().persistent().set(&count_key, &app_count);

        (app_count, student, application_data)
    }
}

mod test;
