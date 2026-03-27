#![allow(deprecated)]
use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};

use crate::base::types::{EventRecord, PoolState, StorageKey};

// ---------------------------------------------------------------------------
// Global event tracker
// ---------------------------------------------------------------------------

/// Increment the persistent event counter and append a record to `AllEvents`.
///
/// Uses persistent storage so the log survives ledger TTL expiry.
/// Called by every public event emitter in this module.
fn record_event(env: &Env, name: &str) {
    let count_key = StorageKey::AllEventsCount;
    let list_key = StorageKey::AllEvents;

    // Increment counter (starts at 0 if not yet initialised)
    let new_index: u64 = env
        .storage()
        .persistent()
        .get::<_, u64>(&count_key)
        .unwrap_or(0)
        + 1;

    env.storage().persistent().set(&count_key, &new_index);

    // Append a lightweight record to the global list
    let mut list: Vec<EventRecord> = env
        .storage()
        .persistent()
        .get::<_, Vec<EventRecord>>(&list_key)
        .unwrap_or_else(|| Vec::new(env));

    list.push_back(EventRecord {
        index: new_index,
        name: String::from_str(env, name),
        timestamp: env.ledger().timestamp(),
    });

    env.storage().persistent().set(&list_key, &list);
}

// ---------------------------------------------------------------------------
// Event emitters
// ---------------------------------------------------------------------------

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
    record_event(env, "campaign_created");
}

pub fn campaign_goal_updated(env: &Env, id: BytesN<32>, new_goal: i128) {
    let topics = (Symbol::new(env, "campaign_goal_updated"), id);
    env.events().publish(topics, new_goal);
    record_event(env, "campaign_goal_updated");
}

#[allow(clippy::too_many_arguments)]
pub fn pool_created(
    env: &Env,
    pool_id: u64,
    creator: Address,
    details: (String, String, i128, i128, u64),
) {
    let topics = (Symbol::new(env, "pool_created"), pool_id, creator);
    env.events().publish(topics, details);
    record_event(env, "pool_created");
}

pub fn event_created(
    env: &Env,
    pool_id: u64,
    name: String,
    creator: Address,
    target_amount: i128,
    deadline: u64,
) {
    let topics = (Symbol::new(env, "event_created"), pool_id, creator);
    env.events()
        .publish(topics, (name, target_amount, deadline));
    record_event(env, "event_created");
}

pub fn pool_state_updated(env: &Env, pool_id: u64, new_state: PoolState) {
    let topics = (Symbol::new(env, "pool_state_updated"), pool_id);
    env.events().publish(topics, new_state);
    record_event(env, "pool_state_updated");
}

pub fn contract_paused(env: &Env, admin: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "contract_paused"), admin);
    env.events().publish(topics, timestamp);
    record_event(env, "contract_paused");
}

pub fn contract_unpaused(env: &Env, admin: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "contract_unpaused"), admin);
    env.events().publish(topics, timestamp);
    record_event(env, "contract_unpaused");
}

pub fn admin_renounced(env: &Env, admin: Address) {
    let topics = (Symbol::new(env, "admin_renounced"), admin);
    env.events().publish(topics, ());
    record_event(env, "admin_renounced");
}

pub fn emergency_contact_updated(env: &Env, admin: Address, contact: Address) {
    let topics = (Symbol::new(env, "emergency_contact_updated"), admin);
    env.events().publish(topics, contact);
    record_event(env, "emergency_contact_updated");
}

pub fn donation_made(env: &Env, campaign_id: BytesN<32>, contributor: Address, amount: i128) {
    let topics = (Symbol::new(env, "donation_made"), campaign_id);
    env.events().publish(topics, (contributor, amount));
    record_event(env, "donation_made");
}

pub fn campaign_cancelled(env: &Env, id: BytesN<32>) {
    let topics = (Symbol::new(env, "campaign_cancelled"), id);
    env.events().publish(topics, ());
    record_event(env, "campaign_cancelled");
}

pub fn campaign_refunded(env: &Env, id: BytesN<32>, contributor: Address, amount: i128) {
    let topics = (Symbol::new(env, "campaign_refunded"), id, contributor);
    env.events().publish(topics, amount);
    record_event(env, "campaign_refunded");
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
    record_event(env, "contribution");
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
    record_event(env, "emergency_withdraw_requested");
}

pub fn emergency_withdraw_executed(env: &Env, admin: Address, token: Address, amount: i128) {
    let topics = (Symbol::new(env, "emergency_withdraw_executed"), admin);
    env.events().publish(topics, (token, amount));
    record_event(env, "emergency_withdraw_executed");
}

pub fn crowdfunding_token_set(env: &Env, admin: Address, token: Address) {
    let topics = (Symbol::new(env, "crowdfunding_token_set"), admin);
    env.events().publish(topics, token);
    record_event(env, "crowdfunding_token_set");
}

pub fn creation_fee_set(env: &Env, admin: Address, fee: i128) {
    let topics = (Symbol::new(env, "creation_fee_set"), admin);
    env.events().publish(topics, fee);
    record_event(env, "creation_fee_set");
}

pub fn creation_fee_paid(env: &Env, creator: Address, amount: i128) {
    let topics = (Symbol::new(env, "creation_fee_paid"), creator);
    env.events().publish(topics, amount);
    record_event(env, "creation_fee_paid");
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
    record_event(env, "refund");
}

pub fn pool_closed(env: &Env, pool_id: u64, closed_by: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "pool_closed"), pool_id, closed_by);
    env.events().publish(topics, timestamp);
    record_event(env, "pool_closed");
}

pub fn platform_fees_withdrawn(env: &Env, to: Address, amount: i128) {
    let topics = (Symbol::new(env, "platform_fees_withdrawn"), to);
    env.events().publish(topics, amount);
    record_event(env, "platform_fees_withdrawn");
}

pub fn event_fees_withdrawn(env: &Env, admin: Address, to: Address, amount: i128) {
    let topics = (Symbol::new(env, "event_fees_withdrawn"), admin, to);
    env.events().publish(topics, amount);
    record_event(env, "event_fees_withdrawn");
}

pub fn address_blacklisted(env: &Env, admin: Address, address: Address) {
    let topics = (Symbol::new(env, "address_blacklisted"), admin);
    env.events().publish(topics, address);
    record_event(env, "address_blacklisted");
}

pub fn address_unblacklisted(env: &Env, admin: Address, address: Address) {
    let topics = (Symbol::new(env, "address_unblacklisted"), admin);
    env.events().publish(topics, address);
    record_event(env, "address_unblacklisted");
}

pub fn pool_metadata_updated(env: &Env, pool_id: u64, updater: Address, new_metadata_hash: String) {
    let topics = (Symbol::new(env, "pool_metadata_updated"), pool_id, updater);
    env.events().publish(topics, new_metadata_hash);
    record_event(env, "pool_metadata_updated");
}

pub fn platform_fee_bps_set(env: &Env, admin: Address, fee_bps: u32) {
    let topics = (Symbol::new(env, "platform_fee_bps_set"), admin);
    env.events().publish(topics, fee_bps);
    record_event(env, "platform_fee_bps_set");
}

pub fn ticket_sold(
    env: &Env,
    pool_id: u64,
    buyer: Address,
    price: i128,
    event_amount: i128,
    fee_amount: i128,
) {
    let topics = (Symbol::new(env, "ticket_sold"), pool_id, buyer);
    env.events()
        .publish(topics, (price, event_amount, fee_amount));
    record_event(env, "ticket_sold");
}
