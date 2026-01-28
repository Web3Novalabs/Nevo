#![allow(deprecated)]
use soroban_sdk::{Address, BytesN, Env, String, Symbol};

use crate::base::types::PoolState;

pub fn campaign_created(
    env: &Env,
    id: BytesN<32>,
    title: String,
    creator: Address,
    goal: i128,
    deadline: u64,
) {
    let topics = (Symbol::new(env, "campaign_created"), id, creator);
    env.events().publish(topics, (title, goal, deadline));
}

pub fn pool_created(
    env: &Env,
    pool_id: u64,
    name: String,
    description: String,
    creator: Address,
    target_amount: i128,
    deadline: u64,
) {
    let topics = (Symbol::new(env, "pool_created"), pool_id, creator);
    env.events()
        .publish(topics, (name, description, target_amount, deadline));
}

pub fn pool_state_updated(env: &Env, pool_id: u64, new_state: PoolState) {
    let topics = (Symbol::new(env, "pool_state_updated"), pool_id);
    env.events().publish(topics, new_state);
}

pub fn contract_paused(env: &Env, admin: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "contract_paused"), admin);
    env.events().publish(topics, timestamp);
}

pub fn contract_unpaused(env: &Env, admin: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "contract_unpaused"), admin);
    env.events().publish(topics, timestamp);
}

// pub fn contribution(
//     env: &Env,
//     pool_id: u64,
//     contributor: Address,
//     asset: Address,
//     amount: i128,
//     timestamp: u64,
//     is_private: bool,
// ) {
//     let topics = (Symbol::new(env, "contribution"), pool_id);
//     env.events()
//         .publish(topics, (contributor, asset, amount, timestamp, is_private));
// }
