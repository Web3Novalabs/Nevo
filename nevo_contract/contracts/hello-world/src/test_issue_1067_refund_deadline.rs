#![cfg(test)]
//! Tests for pool refund deadline validation — issues #1067 and #1664.
//!
//! The refund window is derived from the funding deadline so that the two
//! deadline models can never contradict each other:
//!
//!   * If an explicit `pool_deadline` (ledger sequence) is set, it must be
//!     `>= application_deadline` (timestamp) — otherwise `InvalidDeadline`.
//!   * If no explicit `pool_deadline` is set, the refund window opens at the
//!     funding deadline plus the grace period, so every pool is refundable.
//!
//! `refund_donation` permits a refund only when ALL conditions hold:
//!   1. The current ledger sequence is strictly greater than the effective
//!      refund deadline (explicit `pool_deadline`, or the funding deadline
//!      when no explicit deadline is set).
//!   2. The grace period has elapsed:
//!      `current_ledger >= deadline + REFUND_GRACE_PERIOD_LEDGERS` (17 280 ledgers ≈ 24 h).
//!
//! Scenarios:
//!   1. Refund before the deadline → `PoolNotExpired` (#12).
//!   2. Refund exactly at the deadline → `PoolNotExpired` (#12).
//!   3. Refund after deadline but before grace period ends → `PoolNotExpired` (#12).
//!   4. Refund after the grace period expires → success.
//!   5. Pool without an explicit refund deadline is refundable after the
//!      funding deadline plus the grace period.
//!   6. Contradictory deadlines (refund deadline before funding deadline) are
//!      rejected with `InvalidDeadline` (#13).
//!
//! TTL note: the Soroban test host archives persistent storage entries once the
//! ledger sequence advances past `min_persistent_entry_ttl` ledgers from the
//! time the entry was written.  To avoid spurious `InternalError` failures at
//! high sequence numbers we configure a very large `max_entry_ttl` before
//! advancing the ledger.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::StellarAssetClient,
    Address, Env, String,
};

// REFUND_GRACE_PERIOD_LEDGERS = 17_280 (from lib.rs)
const GRACE: u32 = 17_280;

// A TTL large enough that no entry ever expires during these tests.
const LARGE_TTL: u32 = 10_000_000;

// ── helper ───────────────────────────────────────────────────────────────

/// Mint `amount` tokens to `recipient` and return the token address.
fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// Set the ledger sequence number to `seq`, keeping the existing timestamp and
/// protocol version, but with a very large TTL ceiling so no entries expire.
fn set_ledger_sequence(env: &Env, seq: u32) {
    env.ledger().set(LedgerInfo {
        sequence_number: seq,
        timestamp: env.ledger().timestamp(),
        protocol_version: env.ledger().protocol_version(),
        network_id: Default::default(),
        base_reserve: 5_000_000,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: LARGE_TTL,
        max_entry_ttl: LARGE_TTL,
    });
}

// ── Test 1: refund before the deadline fails ──────────────────────────────

/// Requesting a refund when the current ledger is still before the deadline
/// must panic with `ContractError::PoolNotExpired` (Error(Contract, #12)).
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_before_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();

    // Start at sequence 100
    set_ledger_sequence(&env, 100);

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let donation: i128 = 500_000_000;

    let token = create_token(&env, donation, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Test Pool"),
        &String::from_str(&env, "Testing deadline logic"),
        &5_000_000_000i128,
        &2_000_000u64,
    );
    client.donate_with_token(&pool_id, &donor, &token, &donation);

    // Deadline = 1_000 (future from seq 100)
    client.set_pool_deadline(&pool_id, &1_000u32);

    // Advance to 500 — still before the deadline of 1_000
    set_ledger_sequence(&env, 500);

    // Must panic with PoolNotExpired
    client.refund_donation(&pool_id, &donor, &token);
}

// ── Test 2: refund exactly at the deadline fails ──────────────────────────

/// At exactly the deadline ledger `current_ledger == deadline`, the condition
/// `current_ledger > deadline` is false → must panic with `PoolNotExpired`.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_at_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();

    set_ledger_sequence(&env, 100);

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let donation: i128 = 500_000_000;

    let token = create_token(&env, donation, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Test Pool"),
        &String::from_str(&env, "Testing at-deadline behaviour"),
        &5_000_000_000i128,
        &2_000_000u64,
    );
    client.donate_with_token(&pool_id, &donor, &token, &donation);
    client.set_pool_deadline(&pool_id, &1_000u32);

    // Advance to exactly the deadline
    set_ledger_sequence(&env, 1_000);

    // Must panic — current_ledger == deadline, not > deadline
    client.refund_donation(&pool_id, &donor, &token);
}

// ── Test 3: refund after deadline but before grace period fails ───────────

/// After the deadline has passed (`current_ledger > deadline`) but before the
/// grace period has elapsed (`current_ledger < deadline + GRACE`),
/// `PoolNotExpired` must still be returned.
///
/// Boundary: deadline = 1_000, grace ends at 1_000 + 17_280 = 18_280.
/// We test at sequence 18_279 (one ledger before the grace period ends).
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_after_deadline_before_grace_period_fails() {
    let env = Env::default();
    env.mock_all_auths();

    set_ledger_sequence(&env, 100);

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let donation: i128 = 500_000_000;

    let token = create_token(&env, donation, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Test Pool"),
        &String::from_str(&env, "Testing pre-grace behaviour"),
        &5_000_000_000i128,
        &2_000_000u64,
    );
    client.donate_with_token(&pool_id, &donor, &token, &donation);
    client.set_pool_deadline(&pool_id, &1_000u32);

    // One ledger before the grace period ends: 1_000 + 17_280 - 1 = 18_279
    let one_before_grace_end = 1_000u32 + GRACE - 1;
    set_ledger_sequence(&env, one_before_grace_end);

    // Must still panic — grace period not yet elapsed
    client.refund_donation(&pool_id, &donor, &token);
}

// ── Test 4: refund after the grace period succeeds ────────────────────────

/// Once the grace period has fully elapsed
/// (`current_ledger >= deadline + GRACE`), `refund_donation` must succeed
/// and return the donor's contribution.
///
/// The refund clears the donor's contribution record and reduces the pool
/// `collected` amount.
#[test]
fn test_refund_after_grace_period_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    set_ledger_sequence(&env, 100);

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let donation: i128 = 500_000_000;

    let token = create_token(&env, donation, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Test Pool"),
        &String::from_str(&env, "Testing post-grace behaviour"),
        &5_000_000_000i128,
        &2_000_000u64,
    );

    // donate_with_token transfers tokens donor → contract
    client.donate_with_token(&pool_id, &donor, &token, &donation);

    // Set deadline to 1_000 (future from seq 100)
    client.set_pool_deadline(&pool_id, &1_000u32);

    // Verify contribution is recorded before refund
    let contrib_before = client.get_contribution(&pool_id, &donor);
    assert_eq!(
        contrib_before, donation,
        "Contribution must be recorded before refund"
    );

    // Advance to exactly grace end: 1_000 + 17_280 = 18_280
    let grace_end = 1_000u32 + GRACE;
    set_ledger_sequence(&env, grace_end);

    // Refund must succeed
    client.refund_donation(&pool_id, &donor, &token);

    // Contribution record must be cleared
    let contrib_after = client.get_contribution(&pool_id, &donor);
    assert_eq!(
        contrib_after, 0i128,
        "Contribution must be zeroed after a successful refund"
    );

    // Pool collected must be zero (the only donation was refunded)
    let pool_collected = client.get_total_raised(&pool_id);
    assert_eq!(
        pool_collected, 0i128,
        "Pool collected must decrease by the refunded donation"
    );
}

// ── Test 5: pool without an explicit refund deadline is refundable ────────

/// A pool that never calls `set_pool_deadline` must still have a reachable
/// refund window: the effective refund deadline is the funding deadline
/// (`application_deadline`), and the grace period applies on top of it.
///
/// Before the funding deadline + grace period → `PoolNotExpired`.
/// At/after the funding deadline + grace period → refund succeeds.
#[test]
fn test_refund_without_explicit_deadline_uses_funding_deadline() {
// ── Test 5: cancelled pool refunds are always available (issue #1678) ─────

/// A Cancelled pool must never strand donor funds: `refund_donation` must
/// succeed for a cancelled pool even when no deadline has been set (or the
/// deadline/grace period has not elapsed). Closing such a pool while
/// `collected > 0` must be rejected so funds stay recoverable.
#[test]
fn test_cancelled_pool_refund_without_deadline_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    set_ledger_sequence(&env, 100);

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let donation: i128 = 500_000_000;

    let token = create_token(&env, donation, &donor);

    // Funding deadline (timestamp) doubles as the refund deadline base.
    let funding_deadline: u64 = 2_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "No Refund Deadline Pool"),
        &String::from_str(&env, "Refund window derived from funding deadline"),
        &5_000_000_000u128,
        &funding_deadline,
    );
    client.donate_with_token(&pool_id, &donor, &token, &donation);

    // No explicit refund deadline is set.
    assert_eq!(
        client.get_pool_deadline(&pool_id),
        0u32,
        "No explicit refund deadline should be set"
    );

    // One ledger before the derived grace end → still not refundable.
    let derived_deadline = funding_deadline as u32;
    let one_before_grace_end = derived_deadline + GRACE - 1;
    set_ledger_sequence(&env, one_before_grace_end);

    let before = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.refund_donation(&pool_id, &donor, &token);
    }));
    assert!(
        before.is_err(),
        "Refund must fail before funding deadline + grace period"
    );

    // At the derived grace end → refund succeeds.
    let grace_end = derived_deadline + GRACE;
    set_ledger_sequence(&env, grace_end);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cancelled Refund Pool"),
        &String::from_str(&env, "Testing cancelled refunds"),
        &5_000_000_000i128,
        &2_000_000u64,
    );
    client.donate_with_token(&pool_id, &donor, &token, &donation);

    // Cancel the pool without ever setting a deadline.
    client.cancel_pool(&pool_id);

    // Closing while funds are still held must not be allowed.
    let close_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.close_pool(&pool_id);
    }));
    assert!(
        close_result.is_err(),
        "close_pool must be rejected while collected > 0"
    );

    // Refund must succeed immediately for a cancelled pool, regardless of
    // the (unset) deadline and grace period.
    client.refund_donation(&pool_id, &donor, &token);

    let contrib_after = client.get_contribution(&pool_id, &donor);
    assert_eq!(
        contrib_after, 0u128,
        "Contribution must be zeroed after a successful refund"
    );
}

// ── Test 6: contradictory deadlines are rejected ──────────────────────────

/// Setting an explicit refund deadline that precedes the funding deadline
/// would allow refunds while donations are still being accepted.  Such a
/// configuration must be rejected with `InvalidDeadline` (Error(Contract, #13)).
#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_contradictory_deadlines_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    set_ledger_sequence(&env, 100);

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    // Funding deadline (timestamp) is 2_000_000.
    let funding_deadline: u64 = 2_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Contradictory Deadlines Pool"),
        &String::from_str(&env, "Refund deadline before funding deadline"),
        &5_000_000_000u128,
        &funding_deadline,
    );

    // Refund deadline strictly before the funding deadline → must be rejected.
    client.set_pool_deadline(&pool_id, &((funding_deadline as u32) - 1));
        contrib_after, 0i128,
        "Contribution must be zeroed after a cancelled-pool refund"
    );

    let pool_collected = client.get_total_raised(&pool_id);
    assert_eq!(
        pool_collected, 0i128,
        "Pool collected must be zero after refunding a cancelled pool"
    );

    // With everything refunded, closing the pool is now safe.
    client.close_pool(&pool_id);
}
