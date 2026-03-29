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

/// @notice Emitted when a new fundraising campaign is created.
/// @dev Publishes campaign identity and creator as indexed topics; title, goal,
///      and deadline are stored in the event data payload.
/// @param id The unique 32-byte identifier assigned to the campaign.
/// @param title The human-readable name of the campaign.
/// @param creator The address of the account that created the campaign.
/// @param goal The fundraising target amount in the smallest token unit.
/// @param deadline The Unix timestamp (seconds) after which the campaign closes.
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

/// @notice Emitted when the fundraising goal of an existing campaign is updated.
/// @dev Only the campaign ID is indexed; the new goal value is stored in the
///      event data payload.
/// @param id The unique 32-byte identifier of the campaign being updated.
/// @param new_goal The revised fundraising target amount in the smallest token unit.
pub fn campaign_goal_updated(env: &Env, id: BytesN<32>, new_goal: i128) {
    let topics = (Symbol::new(env, "campaign_goal_updated"), id);
    env.events().publish(topics, new_goal);
    record_event(env, "campaign_goal_updated");
}

/// @notice Emitted when a new donation pool is created.
/// @dev Pool ID and creator are indexed topics. The `details` tuple
///      (title, description, min_contribution, target_amount, deadline) is
///      stored in the event data payload.
/// @param pool_id The auto-incremented numeric identifier of the new pool.
/// @param creator The address of the account that created the pool.
/// @param details A tuple containing pool metadata: title, description,
///        minimum contribution, target amount, and deadline timestamp.
#[allow(clippy::too_many_arguments)]
pub fn pool_created(
    env: &Env,
    pool_id: u64,
    creator: Address,
    details: (String, String, i128, i128, u64),
) {
    let topics = (Symbol::new(env, "pool_created"), pool_id, creator);
    env.events().publish(topics, details);
}

/// @notice Emitted when a new ticketed event pool is created.
/// @dev Pool ID and creator are indexed topics. Event name, target amount,
///      and deadline are stored in the event data payload.
/// @param pool_id The numeric identifier of the newly created event pool.
/// @param name The human-readable name of the event.
/// @param creator The address of the account that created the event pool.
/// @param target_amount The fundraising target for the event in the smallest token unit.
/// @param deadline The Unix timestamp (seconds) after which the event pool closes.
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

/// @notice Emitted when the state of a pool changes (e.g., active, closed, cancelled).
/// @dev Pool ID is the indexed topic; the new state variant is stored in the
///      event data payload.
/// @param pool_id The numeric identifier of the pool whose state was updated.
/// @param new_state The updated `PoolState` enum variant reflecting the pool's new status.
pub fn pool_state_updated(env: &Env, pool_id: u64, new_state: PoolState) {
    let topics = (Symbol::new(env, "pool_state_updated"), pool_id);
    env.events().publish(topics, new_state);
    record_event(env, "pool_state_updated");
}

/// @notice Emitted when the contract is paused by an administrator.
/// @dev Admin address is the indexed topic; the pause timestamp is stored in
///      the event data payload. All state-mutating operations are blocked while
///      the contract is paused.
/// @param admin The address of the administrator who triggered the pause.
/// @param timestamp The Unix timestamp (seconds) at which the contract was paused.
pub fn contract_paused(env: &Env, admin: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "contract_paused"), admin);
    env.events().publish(topics, timestamp);
    record_event(env, "contract_paused");
}

/// @notice Emitted when the contract is unpaused by an administrator.
/// @dev Admin address is the indexed topic; the unpause timestamp is stored in
///      the event data payload. Normal operations resume after this event.
/// @param admin The address of the administrator who lifted the pause.
/// @param timestamp The Unix timestamp (seconds) at which the contract was unpaused.
pub fn contract_unpaused(env: &Env, admin: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "contract_unpaused"), admin);
    env.events().publish(topics, timestamp);
    record_event(env, "contract_unpaused");
}

/// @notice Emitted when the current administrator permanently renounces their role.
/// @dev Admin address is the indexed topic; no additional data is published.
///      After this event the admin role is vacant and cannot be reclaimed.
/// @param admin The address of the administrator who renounced their privileges.
pub fn admin_renounced(env: &Env, admin: Address) {
    let topics = (Symbol::new(env, "admin_renounced"), admin);
    env.events().publish(topics, ());
    record_event(env, "admin_renounced");
}

/// @notice Emitted when the emergency contact address is updated by the administrator.
/// @dev Admin address is the indexed topic; the new contact address is stored
///      in the event data payload.
/// @param admin The address of the administrator who performed the update.
/// @param contact The new emergency contact address that was set.
pub fn emergency_contact_updated(env: &Env, admin: Address, contact: Address) {
    let topics = (Symbol::new(env, "emergency_contact_updated"), admin);
    env.events().publish(topics, contact);
    record_event(env, "emergency_contact_updated");
}

/// @notice Emitted when a donation is made to a campaign.
/// @dev Campaign ID is the indexed topic; contributor address and amount are
///      stored in the event data payload.
/// @param campaign_id The unique 32-byte identifier of the campaign receiving the donation.
/// @param contributor The address of the account making the donation.
/// @param amount The donated amount in the smallest token unit.
pub fn donation_made(env: &Env, campaign_id: BytesN<32>, contributor: Address, amount: i128) {
    let topics = (Symbol::new(env, "donation_made"), campaign_id);
    env.events().publish(topics, (contributor, amount));
    record_event(env, "donation_made");
}

/// @notice Emitted when a campaign is cancelled by its creator or an administrator.
/// @dev Campaign ID is the indexed topic; no additional data is published.
///      Contributors become eligible for refunds after this event.
/// @param id The unique 32-byte identifier of the cancelled campaign.
pub fn campaign_cancelled(env: &Env, id: BytesN<32>) {
    let topics = (Symbol::new(env, "campaign_cancelled"), id);
    env.events().publish(topics, ());
    record_event(env, "campaign_cancelled");
}

/// @notice Emitted when a contributor is refunded for a cancelled or failed campaign.
/// @dev Campaign ID and contributor address are indexed topics; the refunded
///      amount is stored in the event data payload.
/// @param id The unique 32-byte identifier of the campaign from which the refund originates.
/// @param contributor The address of the account receiving the refund.
/// @param amount The refunded amount in the smallest token unit.
pub fn campaign_refunded(env: &Env, id: BytesN<32>, contributor: Address, amount: i128) {
    let topics = (Symbol::new(env, "campaign_refunded"), id, contributor);
    env.events().publish(topics, amount);
    record_event(env, "campaign_refunded");
}

/// @notice Emitted when a contribution is made to a donation pool.
/// @dev Pool ID and contributor address are indexed topics. Asset, amount,
///      timestamp, and privacy flag are stored in the event data payload.
/// @param pool_id The numeric identifier of the pool receiving the contribution.
/// @param contributor The address of the account making the contribution.
/// @param asset The contract address of the token used for the contribution.
/// @param amount The contributed amount in the smallest token unit.
/// @param timestamp The Unix timestamp (seconds) at which the contribution was recorded.
/// @param is_private Whether the contribution was made to a private pool.
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

/// @notice Emitted when an emergency withdrawal is requested by the administrator.
/// @dev Admin address is the indexed topic. Token address, amount, and the
///      time-lock expiry are stored in the event data payload. Funds cannot be
///      moved until the unlock time has passed.
/// @param admin The address of the administrator who initiated the withdrawal request.
/// @param token The contract address of the token to be withdrawn.
/// @param amount The amount requested for withdrawal in the smallest token unit.
/// @param unlock_time The Unix timestamp (seconds) after which the withdrawal can be executed.
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

/// @notice Emitted when a previously requested emergency withdrawal is executed.
/// @dev Admin address is the indexed topic; token address and transferred amount
///      are stored in the event data payload.
/// @param admin The address of the administrator who executed the withdrawal.
/// @param token The contract address of the token that was withdrawn.
/// @param amount The amount transferred in the smallest token unit.
pub fn emergency_withdraw_executed(env: &Env, admin: Address, token: Address, amount: i128) {
    let topics = (Symbol::new(env, "emergency_withdraw_executed"), admin);
    env.events().publish(topics, (token, amount));
    record_event(env, "emergency_withdraw_executed");
}

/// @notice Emitted when the administrator sets the token accepted for crowdfunding contributions.
/// @dev Admin address is the indexed topic; the token contract address is stored
///      in the event data payload.
/// @param admin The address of the administrator who set the token.
/// @param token The contract address of the token now accepted for crowdfunding.
pub fn crowdfunding_token_set(env: &Env, admin: Address, token: Address) {
    let topics = (Symbol::new(env, "crowdfunding_token_set"), admin);
    env.events().publish(topics, token);
    record_event(env, "crowdfunding_token_set");
}

/// @notice Emitted when the administrator updates the pool creation fee.
/// @dev Admin address is the indexed topic; the new fee amount is stored in
///      the event data payload.
/// @param admin The address of the administrator who updated the fee.
/// @param fee The new creation fee amount in the smallest token unit.
pub fn creation_fee_set(env: &Env, admin: Address, fee: i128) {
    let topics = (Symbol::new(env, "creation_fee_set"), admin);
    env.events().publish(topics, fee);
    record_event(env, "creation_fee_set");
}

/// @notice Emitted when a creator pays the pool creation fee.
/// @dev Creator address is the indexed topic; the fee amount paid is stored in
///      the event data payload.
/// @param creator The address of the account that paid the creation fee.
/// @param amount The creation fee amount paid in the smallest token unit.
pub fn creation_fee_paid(env: &Env, creator: Address, amount: i128) {
    let topics = (Symbol::new(env, "creation_fee_paid"), creator);
    env.events().publish(topics, amount);
    record_event(env, "creation_fee_paid");
}

/// @notice Emitted when a contributor is refunded from a donation pool.
/// @dev Pool ID and contributor address are indexed topics. Asset, amount, and
///      timestamp are stored in the event data payload.
/// @param pool_id The numeric identifier of the pool from which the refund originates.
/// @param contributor The address of the account receiving the refund.
/// @param asset The contract address of the token being refunded.
/// @param amount The refunded amount in the smallest token unit.
/// @param timestamp The Unix timestamp (seconds) at which the refund was processed.
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

/// @notice Emitted when a donation pool is closed.
/// @dev Pool ID and the address that triggered the closure are indexed topics;
///      the closure timestamp is stored in the event data payload.
/// @param pool_id The numeric identifier of the pool that was closed.
/// @param closed_by The address of the account (creator or admin) that closed the pool.
/// @param timestamp The Unix timestamp (seconds) at which the pool was closed.
pub fn pool_closed(env: &Env, pool_id: u64, closed_by: Address, timestamp: u64) {
    let topics = (Symbol::new(env, "pool_closed"), pool_id, closed_by);
    env.events().publish(topics, timestamp);
    record_event(env, "pool_closed");
}

/// @notice Emitted when accumulated platform fees are withdrawn by the administrator.
/// @dev Recipient address is the indexed topic; the withdrawn amount is stored
///      in the event data payload.
/// @param to The address that received the withdrawn platform fees.
/// @param amount The total platform fee amount withdrawn in the smallest token unit.
pub fn platform_fees_withdrawn(env: &Env, to: Address, amount: i128) {
    let topics = (Symbol::new(env, "platform_fees_withdrawn"), to);
    env.events().publish(topics, amount);
    record_event(env, "platform_fees_withdrawn");
}

/// @notice Emitted when accumulated event fees are withdrawn by the administrator.
/// @dev Admin and recipient addresses are indexed topics; the withdrawn amount
///      is stored in the event data payload.
/// @param admin The address of the administrator who initiated the withdrawal.
/// @param to The address that received the withdrawn event fees.
/// @param amount The total event fee amount withdrawn in the smallest token unit.
pub fn event_fees_withdrawn(env: &Env, admin: Address, to: Address, amount: i128) {
    let topics = (Symbol::new(env, "event_fees_withdrawn"), admin, to);
    env.events().publish(topics, amount);
    record_event(env, "event_fees_withdrawn");
}

/// @notice Emitted when an address is added to the contract blacklist.
/// @dev Admin address is the indexed topic; the blacklisted address is stored
///      in the event data payload. Blacklisted addresses cannot interact with
///      the contract.
/// @param admin The address of the administrator who performed the blacklisting.
/// @param address The address that was added to the blacklist.
pub fn address_blacklisted(env: &Env, admin: Address, address: Address) {
    let topics = (Symbol::new(env, "address_blacklisted"), admin);
    env.events().publish(topics, address);
    record_event(env, "address_blacklisted");
}

/// @notice Emitted when an address is removed from the contract blacklist.
/// @dev Admin address is the indexed topic; the reinstated address is stored
///      in the event data payload. The address regains the ability to interact
///      with the contract after this event.
/// @param admin The address of the administrator who removed the blacklist entry.
/// @param address The address that was removed from the blacklist.
pub fn address_unblacklisted(env: &Env, admin: Address, address: Address) {
    let topics = (Symbol::new(env, "address_unblacklisted"), admin);
    env.events().publish(topics, address);
    record_event(env, "address_unblacklisted");
}

/// @notice Emitted when the off-chain metadata of a pool is updated.
/// @dev Pool ID and updater address are indexed topics; the new metadata hash
///      string is stored in the event data payload.
/// @param pool_id The numeric identifier of the pool whose metadata was updated.
/// @param updater The address of the account that performed the metadata update.
/// @param new_metadata_hash The new content-addressed hash pointing to the updated metadata.
pub fn pool_metadata_updated(env: &Env, pool_id: u64, updater: Address, new_metadata_hash: String) {
    let topics = (Symbol::new(env, "pool_metadata_updated"), pool_id, updater);
    env.events().publish(topics, new_metadata_hash);
    record_event(env, "pool_metadata_updated");
}

/// @notice Emitted when the administrator updates the platform fee in basis points.
/// @dev Admin address is the indexed topic; the new fee in basis points is stored
///      in the event data payload. 100 bps equals 1%.
/// @param admin The address of the administrator who updated the fee.
/// @param fee_bps The new platform fee expressed in basis points (e.g., 250 = 2.5%).
pub fn platform_fee_bps_set(env: &Env, admin: Address, fee_bps: u32) {
    let topics = (Symbol::new(env, "platform_fee_bps_set"), admin);
    env.events().publish(topics, fee_bps);
    record_event(env, "platform_fee_bps_set");
}

/// @notice Emitted when the platform fee is changed, capturing both the previous
///         and the new value for full auditability.
/// @dev Admin address is the indexed topic; the (old_fee_bps, new_fee_bps) tuple
///      is stored in the event data payload. Off-chain indexers can use this event
///      to track the complete fee history without querying historical state.
/// @param admin The address of the administrator who performed the update.
/// @param old_fee_bps The platform fee in basis points that was in effect before the update.
/// @param new_fee_bps The platform fee in basis points that is now in effect after the update.
pub fn platform_fee_updated(env: &Env, admin: Address, old_fee_bps: u32, new_fee_bps: u32) {
    let topics = (Symbol::new(env, "platform_fee_updated"), admin);
    env.events().publish(topics, (old_fee_bps, new_fee_bps));
}

/// @notice Emitted when a ticket is sold for a ticketed event pool.
/// @dev Pool ID and buyer address are indexed topics. Ticket price, the portion
///      allocated to the event, and the platform fee portion are stored in the
///      event data payload.
/// @param pool_id The numeric identifier of the event pool for which the ticket was sold.
/// @param buyer The address of the account that purchased the ticket.
/// @param price The total ticket price paid in the smallest token unit.
/// @param event_amount The portion of the ticket price allocated to the event pool.
/// @param fee_amount The portion of the ticket price collected as a platform fee.
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

pub fn contract_upgraded(env: &Env, new_wasm_hash: BytesN<32>) {
    let topics = (Symbol::new(env, "contract_upgraded"), new_wasm_hash);
    env.events().publish(topics, ());
}

pub fn ticket_purchased(
    env: &Env,
    pool_id: u64,
    buyer: Address,
    price: i128,
    event_amount: i128,
    fee_amount: i128,
) {
    let topics = (Symbol::new(env, "ticket_purchased"), pool_id, buyer);
    env.events()
        .publish(topics, (price, event_amount, fee_amount));
}
