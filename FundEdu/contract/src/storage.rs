use soroban_sdk::Env;

use crate::types::{DataKey, ScholarshipPool};

const POOL_BUMP_AMOUNT: u32 = 100;
const POOL_LIFETIME_THRESHOLD: u32 = 50;

pub fn get_pool(env: &Env, pool_id: u64) -> Option<ScholarshipPool> {
    let key = DataKey::Pool(pool_id);
    if let Some(pool) = env
        .storage()
        .persistent()
        .get::<DataKey, ScholarshipPool>(&key)
    {
        env.storage()
            .persistent()
            .extend_ttl(&key, POOL_LIFETIME_THRESHOLD, POOL_BUMP_AMOUNT);
        Some(pool)
    } else {
        None
    }
}

pub fn set_pool(env: &Env, pool_id: u64, pool: &ScholarshipPool) {
    let key = DataKey::Pool(pool_id);
    env.storage().persistent().set(&key, pool);
    env.storage()
        .persistent()
        .extend_ttl(&key, POOL_LIFETIME_THRESHOLD, POOL_BUMP_AMOUNT);
}

pub fn next_pool_id(env: &Env) -> u64 {
    let id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextPoolId)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&DataKey::NextPoolId, &(id + 1));
    id
}
