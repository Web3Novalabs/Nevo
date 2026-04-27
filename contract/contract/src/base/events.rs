#![allow(deprecated)]
use soroban_sdk::{symbol_short, Address, BytesN, Env, String, Symbol};

use crate::base::types::PoolState;

/// Executes the campaign created operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `id` - The id.
/// * `title` - The title.
/// * `creator` - The creator.
/// * `goal` - The goal.
/// * `deadline` - The deadline.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
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

/// Executes the campaign goal updated operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `id` - The id.
/// * `new_goal` - The new goal.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn campaign_goal_updated(env: &Env, id: BytesN<32>, new_goal: i128) {
    let topics = (Symbol::new(env, "campaign_goal_updated"), id);
    env.events().publish(topics, new_goal);
}

#[allow(clippy::too_many_arguments)]
/// Executes the pool created operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `pool_id` - The pool id.
/// * `creator` - The creator.
/// * `details` - The details.
/// * `String` - The String.
/// * `i128` - The i128.
/// * `i128` - The i128.
/// * `u64` - The u64.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn pool_created(
    env: &Env,
    pool_id: u64,
    creator: Address,
    details: (String, String, i128, i128, u64),
) {
    let topics = (symbol_short!("PoolCre"), pool_id, creator);
    env.events().publish(topics, details);
}

/// Executes the event created operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `pool_id` - The pool id.
/// * `name` - The name.
/// * `creator` - The creator.
/// * `target_amount` - The target amount.
/// * `deadline` - The deadline.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn pool_metadata_updated_v2(
    env: &Env,
    pool_id: u64,
    updater: Address,
    new_metadata_hash: String,
) {
    let topics = (symbol_short!("PoolUpd"), pool_id, updater);
    env.events().publish(topics, new_metadata_hash);
}

pub fn pool_paused(env: &Env, pool_id: u64) {
    let topics = (symbol_short!("PoolPau"), pool_id);
    env.events().publish(topics, ());
}

pub fn pool_unpaused(env: &Env, pool_id: u64) {
    let topics = (symbol_short!("PoolUnp"), pool_id);
    env.events().publish(topics, ());
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
}

/// Executes the pool state updated operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `pool_id` - The pool id.
/// * `new_state` - The new state.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn pool_state_updated(env: &Env, pool_id: u64, new_state: PoolState) {
    let topics = (Symbol::new(env, "pool_state_updated"), pool_id);
    env.events().publish(topics, new_state);
}

/// Executes the contract paused operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `timestamp` - The timestamp.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn contract_paused(env: &Env, admin: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "contract_paused"), admin);
    env.events().publish(topics, timestamp);
}

/// Executes the contract unpaused operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `timestamp` - The timestamp.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn contract_unpaused(env: &Env, admin: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "contract_unpaused"), admin);
    env.events().publish(topics, timestamp);
}

/// Executes the admin renounced operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn admin_renounced(env: &Env, admin: Address) {
    let topics = (Symbol::new(env, "admin_renounced"), admin);
    env.events().publish(topics, ());
}

/// Executes the emergency contact updated operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `contact` - The contact.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn emergency_contact_updated(env: &Env, admin: Address, contact: Address) {
    let topics = (Symbol::new(env, "emergency_contact_updated"), admin);
    env.events().publish(topics, contact);
}

/// Executes the donation made operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `campaign_id` - The campaign id.
/// * `contributor` - The contributor.
/// * `amount` - The amount.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn donation_made(env: &Env, campaign_id: BytesN<32>, contributor: Address, amount: i128) {
    let topics = (Symbol::new(env, "donation_made"), campaign_id);
    env.events().publish(topics, (contributor, amount));
}

/// Executes the campaign cancelled operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `id` - The id.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn campaign_cancelled(env: &Env, id: BytesN<32>) {
    let topics = (Symbol::new(env, "campaign_cancelled"), id);
    env.events().publish(topics, ());
}

/// Executes the campaign refunded operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `id` - The id.
/// * `contributor` - The contributor.
/// * `amount` - The amount.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn campaign_refunded(env: &Env, id: BytesN<32>, contributor: Address, amount: i128) {
    let topics = (Symbol::new(env, "campaign_refunded"), id, contributor);
    env.events().publish(topics, amount);
}

/// Executes the contribution operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `pool_id` - The pool id.
/// * `contributor` - The contributor.
/// * `asset` - The asset.
/// * `amount` - The amount.
/// * `timestamp` - The timestamp.
/// * `is_private` - The is private.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
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

/// Executes the emergency withdraw requested operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `token` - The token.
/// * `amount` - The amount.
/// * `unlock_time` - The unlock time.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
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

/// Executes the emergency withdraw executed operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `token` - The token.
/// * `amount` - The amount.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn emergency_withdraw_executed(env: &Env, admin: Address, token: Address, amount: i128) {
    let topics = (Symbol::new(env, "emergency_withdraw_executed"), admin);
    env.events().publish(topics, (token, amount));
}

/// Executes the crowdfunding token set operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `token` - The token.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn crowdfunding_token_set(env: &Env, admin: Address, token: Address) {
    let topics = (Symbol::new(env, "crowdfunding_token_set"), admin);
    env.events().publish(topics, token);
}

/// Executes the creation fee set operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `fee` - The fee.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn creation_fee_set(env: &Env, admin: Address, fee: i128) {
    let topics = (Symbol::new(env, "creation_fee_set"), admin);
    env.events().publish(topics, fee);
}

/// Executes the creation fee paid operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `creator` - The creator.
/// * `amount` - The amount.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn creation_fee_paid(env: &Env, creator: Address, amount: i128) {
    let topics = (Symbol::new(env, "creation_fee_paid"), creator);
    env.events().publish(topics, amount);
}

/// Executes the refund operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `pool_id` - The pool id.
/// * `contributor` - The contributor.
/// * `asset` - The asset.
/// * `amount` - The amount.
/// * `timestamp` - The timestamp.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
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

/// Executes the pool closed operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `pool_id` - The pool id.
/// * `closed_by` - The closed by.
/// * `timestamp` - The timestamp.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn pool_closed(env: &Env, pool_id: u64, closed_by: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "pool_closed"), pool_id, closed_by);
    env.events().publish(topics, timestamp);
}

/// Executes the platform fees withdrawn operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `to` - The to.
/// * `amount` - The amount.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn platform_fees_withdrawn(env: &Env, to: Address, amount: i128) {
    let topics = (Symbol::new(env, "platform_fees_withdrawn"), to);
    env.events().publish(topics, amount);
}

/// Executes the event fees withdrawn operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `to` - The to.
/// * `amount` - The amount.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn event_fees_withdrawn(env: &Env, admin: Address, to: Address, amount: i128) {
    let topics = (Symbol::new(env, "event_fees_withdrawn"), admin, to);
    env.events().publish(topics, amount);
}

/// Executes the address blacklisted operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `address` - The address.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn address_blacklisted(env: &Env, admin: Address, address: Address) {
    let topics = (Symbol::new(env, "address_blacklisted"), admin);
    env.events().publish(topics, address);
}

/// Executes the address unblacklisted operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `address` - The address.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn address_unblacklisted(env: &Env, admin: Address, address: Address) {
    let topics = (Symbol::new(env, "address_unblacklisted"), admin);
    env.events().publish(topics, address);
}

/// Executes the pool metadata updated operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `pool_id` - The pool id.
/// * `updater` - The updater.
/// * `new_metadata_hash` - The new metadata hash.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
pub fn pool_metadata_updated(env: &Env, pool_id: u64, updater: Address, new_metadata_hash: String) {
    let topics = (Symbol::new(env, "pool_metadata_updated"), pool_id, updater);
    env.events().publish(topics, new_metadata_hash);
}

/// Executes the platform fee bps set operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `admin` - The admin.
/// * `fee_bps` - The fee bps.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.

pub fn platform_fee_bps_set(env: &Env, admin: Address, fee_bps: u32) {
    let topics = (Symbol::new(env, "platform_fee_bps_set"), admin);
    env.events().publish(topics, fee_bps);
}

/// Executes the ticket sold operation.
///
/// # Arguments
///
/// * `env` - The execution environment.
/// * `pool_id` - The pool id.
/// * `buyer` - The buyer.
/// * `price` - The price.
/// * `event_amount` - The event amount.
/// * `fee_amount` - The fee amount.
///
/// # Panics
///
/// Panics if the internal state is invalid or required conditions are not met.
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
}

pub fn scholarship_applied(env: &Env, pool_id: u64, applicant: Address) {
    let topics = (Symbol::new(env, "scholarship_applied"), pool_id, applicant);
    env.events().publish(topics, ());
}

pub fn scholarship_approved(env: &Env, pool_id: u64, applicant: Address, validator: Address) {
    let topics = (Symbol::new(env, "scholarship_approved"), pool_id, applicant);
    env.events().publish(topics, validator);
}

pub fn scholarship_rejected(env: &Env, pool_id: u64, applicant: Address, validator: Address) {
    let topics = (Symbol::new(env, "scholarship_rejected"), pool_id, applicant);
    env.events().publish(topics, validator);
}
pub fn school_registered(env: &Env, school_addr: Address) {
    let topics = (symbol_short!("SchReg"), school_addr);
    env.events().publish(topics, ());
}

pub fn school_revoked(env: &Env, school_addr: Address) {
    let topics = (symbol_short!("SchRev"), school_addr);
    env.events().publish(topics, ());
}

pub fn application_approved(env: &Env, _admin: Address, cause: Address) {
    school_registered(env, cause);
}

pub fn application_rejected(env: &Env, _admin: Address, cause: Address) {
    school_revoked(env, cause);
}

pub fn application_submitted(env: &Env, pool_id: u64, student: Address, requested_amount: i128) {
    let topics = (symbol_short!("AppSub"), pool_id, student);
    env.events().publish(topics, requested_amount);
}

pub fn school_removed(env: &Env, admin: Address, school_addr: Address, pool_id: u64) {
    let topics = (Symbol::new(env, "school_removed"), admin, school_addr);
    env.events().publish(topics, pool_id);
}

/// Emitted when a sponsor or trusted school sets up milestone-based disbursements
/// for an approved scholarship application.
///
/// # Arguments
/// * `env`             - The execution environment.
/// * `pool_id`         - The pool the application belongs to.
/// * `student`         - The approved applicant whose milestones were configured.
/// * `milestone_count` - Number of milestones registered.
/// * `total_amount`    - Sum of all milestone amounts (must equal requested_amount).
pub fn milestones_set(
    env: &Env,
    pool_id: u64,
    student: Address,
    milestone_count: u32,
    total_amount: i128,
) {
    let topics = (symbol_short!("MilSet"), pool_id, student);
    env.events().publish(topics, (milestone_count, total_amount));
}

/// Emitted when the pool sponsor withdraws unallocated funds.
pub fn pool_unallocated_withdrawn(env: &Env, pool_id: u64, sponsor: Address, amount: i128) {
    let topics = (symbol_short!("PoolWit"), pool_id, sponsor);
    env.events().publish(topics, amount);
}
