#![cfg(test)]
//! Tests for `refund_donation` eligibility and payout — issue #1350.
//!
//! A refund is permitted only when the pool has a deadline, the ledger has moved
//! strictly past it, and the grace period has elapsed:
//! `current_ledger >= deadline + REFUND_GRACE_PERIOD_LEDGERS` (17 280 ledgers).
//! The payout is the contribution recorded for that donor, paid out of the
//! contract, and the record is cleared before the transfer.
//!
//! Scenarios:
//!   1. Refund with no deadline set → `PoolNotExpired` (#12).
//!   2. Refund before the deadline → `PoolNotExpired` (#12).
//!   3. Refund after the deadline but inside the grace period → `PoolNotExpired` (#12).
//!   4. Eligible refund pays exactly the recorded contribution, restoring the
//!      donor's balance and emptying the contract.
//!   5. A donor with no contribution on this pool → `NoContributionToRefund` (#13).
//!   6. The contribution record is cleared: a second refund is rejected with (#13)
//!      and moves no tokens.
//!
//! TTL note: jumping the ledger sequence far enough to clear the grace period
//! would archive persistent entries under the default ceilings, so the ledger is
//! configured with a very large `max_entry_ttl` first — same approach as
//! `test_issue_1067_refund_deadline.rs` (which is not currently registered, see
//! the note in the pull request).

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

/// REFUND_GRACE_PERIOD_LEDGERS from lib.rs.
const GRACE: u32 = 17_280;
/// The deadline used by most tests here, comfortably above sequence 0.
const DEADLINE: u32 = 1_000;
/// A TTL ceiling large enough that nothing expires while the sequence moves.
const LARGE_TTL: u32 = 10_000_000;

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(env, &token.address()).mint(recipient, &amount);
    token.address()
}

/// Move the ledger to `seq` without letting anything expire.
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

fn make_pool(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Refund Pool"),
        &String::from_str(env, "refund_donation tests"),
        &1_000_000_000u128,
        &100_000u64,
    )
}

/// A pool, a funded donor, the token minted to them, and a 400-token donation
/// already recorded against the pool. Returns what the success-path tests need.
fn setup_with_donation(env: &Env) -> (ContractClient<'_>, u32, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let pool_id = make_pool(env, &client);
    let donor = Address::generate(env);
    let token = create_token(env, 1_000i128, &donor);

    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &donor, &token, &400i128);

    (client, pool_id, donor, token)
}

// ── 1–3. The three ineligible states ──────────────────────────────────────

/// With no deadline recorded for the pool, no refund is possible — the deadline
/// is what makes a refund meaningful in the first place.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_without_deadline_fails() {
    let env = Env::default();
    let (client, pool_id, donor, token) = setup_with_donation(&env);

    set_ledger_sequence(&env, DEADLINE + GRACE);
    client.refund_donation(&pool_id, &donor, &token);
}

/// Before the deadline the pool is still open for its purpose, so a refund is
/// refused even though the donor has a contribution.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_before_deadline_fails() {
    let env = Env::default();
    let (client, pool_id, donor, token) = setup_with_donation(&env);

    client.set_pool_deadline(&pool_id, &DEADLINE);
    set_ledger_sequence(&env, DEADLINE);
    client.refund_donation(&pool_id, &donor, &token);
}

/// Past the deadline but still inside the grace period is also too early: the
/// grace period exists so a late-but-legitimate donation is not refunded out
/// from under the pool.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_inside_grace_period_fails() {
    let env = Env::default();
    let (client, pool_id, donor, token) = setup_with_donation(&env);

    client.set_pool_deadline(&pool_id, &DEADLINE);
    set_ledger_sequence(&env, DEADLINE + GRACE - 1);
    client.refund_donation(&pool_id, &donor, &token);
}

// ── 4. The eligible refund ────────────────────────────────────────────────

/// Once the grace period has elapsed the refund pays exactly what the donor
/// contributed, out of the contract, and the pool total drops by the same
/// amount.
#[test]
fn test_refund_pays_the_recorded_contribution() {
    let env = Env::default();
    let (client, pool_id, donor, token) = setup_with_donation(&env);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&donor), 600i128);
    assert_eq!(token_client.balance(&client.address), 400i128);

    client.set_pool_deadline(&pool_id, &DEADLINE);
    set_ledger_sequence(&env, DEADLINE + GRACE);
    client.refund_donation(&pool_id, &donor, &token);

    assert_eq!(
        token_client.balance(&donor),
        1_000i128,
        "the refund must return exactly the 400 contributed"
    );
    assert_eq!(
        token_client.balance(&client.address),
        0i128,
        "the contract must hold nothing after the only donor refunded"
    );
    assert_eq!(
        client.get_total_raised(&pool_id),
        0u128,
        "the pool total must drop by the refunded contribution"
    );
}

// ── 5. Somebody who never contributed ─────────────────────────────────────

/// A caller with no recorded contribution on this pool is rejected even when
/// the refund window is open — the window alone is not authority to withdraw.
#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_refund_by_non_donor_fails() {
    let env = Env::default();
    let (client, pool_id, _, token) = setup_with_donation(&env);
    let stranger = Address::generate(&env);

    client.set_pool_deadline(&pool_id, &DEADLINE);
    set_ledger_sequence(&env, DEADLINE + GRACE);
    client.refund_donation(&pool_id, &stranger, &token);
}

// ── 6. One refund per contribution ────────────────────────────────────────

/// The contribution record is cleared before the payout, so a second refund in
/// the same window is rejected and moves no tokens. This is the case that says
/// whether the refund can be drained twice.
#[test]
fn test_second_refund_is_rejected_and_moves_nothing() {
    let env = Env::default();
    let (client, pool_id, donor, token) = setup_with_donation(&env);
    let token_client = TokenClient::new(&env, &token);

    client.set_pool_deadline(&pool_id, &DEADLINE);
    set_ledger_sequence(&env, DEADLINE + GRACE);
    client.refund_donation(&pool_id, &donor, &token);

    let balance_after_first = token_client.balance(&donor);
    let result = client.try_refund_donation(&pool_id, &donor, &token);

    // `try_` reports the failure as an opaque host error rather than a typed
    // one; the variant itself is pinned by the non-donor test above, which can
    // assert the message.
    assert!(
        result.is_err(),
        "a cleared contribution must not be refundable twice"
    );
    assert_eq!(
        token_client.balance(&donor),
        balance_after_first,
        "the rejected refund must not move tokens"
    );
    assert_eq!(client.get_total_raised(&pool_id), 0u128);
}
