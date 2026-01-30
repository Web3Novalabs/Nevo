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

pub fn donation_made(env: &Env, campaign_id: BytesN<32>, contributor: Address, amount: i128) {
    let topics = (Symbol::new(env, "donation_made"), campaign_id);
    env.events().publish(topics, (contributor, amount));
}

pub fn contribution(
    env: &Env,
    pool_id: u64,
    contributor: Address,
    asset: Address,
    amount: i128,
    timestamp: u64,
    is_private: bool,
) {
    let topics = (Symbol::new(env, "contribution"), pool_id, contributor);
    env.events()
        .publish(topics, (asset, amount, timestamp, is_private));
}

pub fn emergency_withdraw_requested(
    env: &Env,
    admin: Address,
    token: Address,
    amount: i128,
    unlock_time: u64,
) {
    let topics = (Symbol::new(env, "emergency_withdraw_requested"), admin);
    env.events().publish(topics, (token, amount, unlock_time));
}

pub fn emergency_withdraw_executed(env: &Env, admin: Address, token: Address, amount: i128) {
    let topics = (Symbol::new(env, "emergency_withdraw_executed"), admin);
    env.events().publish(topics, (token, amount));
}

pub fn crowdfunding_token_set(env: &Env, admin: Address, token: Address) {
    let topics = (Symbol::new(env, "crowdfunding_token_set"), admin);
    env.events().publish(topics, token);
}

pub fn creation_fee_set(env: &Env, admin: Address, fee: i128) {
    let topics = (Symbol::new(env, "creation_fee_set"), admin);
    env.events().publish(topics, fee);
}

pub fn creation_fee_paid(env: &Env, creator: Address, amount: i128) {
    let topics = (Symbol::new(env, "creation_fee_paid"), creator);
    env.events().publish(topics, amount);
}

pub fn refund(
    env: &Env,
    pool_id: u64,
    contributor: Address,
    asset: Address,
    amount: i128,
    timestamp: u64,
) {
    let topics = (Symbol::new(env, "refund"), pool_id, contributor);
    env.events().publish(topics, (asset, amount, timestamp));
}

pub fn pool_closed(env: &Env, pool_id: u64, closed_by: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "pool_closed"), pool_id, closed_by);
    env.events().publish(topics, timestamp);
}
