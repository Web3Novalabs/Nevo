#![cfg(test)]
//! Tests for `donate` boundary amounts — issue #1348.
//!
//! `donate` is the accounting-only path: it takes no auth and moves no tokens
//! (that is `donate_with_token`), so everything here needs only a created pool.
//!
//! Scenarios:
//!   1. The smallest valid contribution (1) is accepted and recorded exactly.
//!   2. A zero-amount contribution is rejected and changes nothing.
//!   3. A contribution to a pool that does not exist fails with `PoolNotFound` (#1).
//!   4. A contribution to a closed pool fails with `PoolIsClosed` (#4), and to a
//!      cancelled pool with `InvalidPoolState` (#2).
//!   5. `u128::MAX` is recorded exactly; the contribution after it fails through
//!      the checked add instead of wrapping around.
//!   6. The pool total and the donor count move together: a new donor adds one
//!      to the count, a repeat donor does not, and both are updated by the same
//!      call.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

const GOAL: u128 = 1_000_000_000;

/// A live pool owned by a fresh creator.
fn make_pool(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Boundary Amounts"),
        &String::from_str(env, "donate boundary tests"),
        &GOAL,
        &100_000u64,
    )
}

fn client_for(env: &Env) -> (ContractClient<'_>, u32) {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let pool_id = make_pool(env, &client);
    (client, pool_id)
}

// ── 1. Smallest valid contribution ────────────────────────────────────────

/// A contribution of 1 is valid: the pool total becomes 1 and the donor is
/// counted once. This is the boundary the `amount <= 0` style of check must
/// not reject.
#[test]
fn test_donate_minimum_amount_is_accepted() {
    let env = Env::default();
    let (client, pool_id) = client_for(&env);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &1u128);

    assert_eq!(
        client.get_total_raised(&pool_id),
        1u128,
        "a contribution of 1 must be recorded, not rounded away"
    );
    assert_eq!(client.get_donor_count(&pool_id), 1u32);
}

// ── 2. Zero amount ────────────────────────────────────────────────────────

/// A zero-amount contribution is rejected with `InvalidAmount`, and leaves both
/// the total and the donor count untouched. A zero contribution that "succeeds"
/// still marks the caller as a donor, which is what makes this worth rejecting
/// rather than ignoring.
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_donate_zero_amount_is_rejected() {
    let env = Env::default();
    let (client, pool_id) = client_for(&env);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &0u128);
}

/// The rejected zero contribution must not have marked the donor before it
/// panicked — the check has to come before the storage writes.
#[test]
fn test_donate_zero_amount_leaves_no_trace() {
    let env = Env::default();
    let (client, pool_id) = client_for(&env);
    let donor = Address::generate(&env);

    // Soroban rolls the invocation back, so the panic is not observable from
    // the caller's side except through storage.
    let _ = client.try_donate(&pool_id, &donor, &0u128);

    assert_eq!(client.get_total_raised(&pool_id), 0u128);
    assert_eq!(client.get_donor_count(&pool_id), 0u32);
}

// ── 3. Pool that does not exist ───────────────────────────────────────────

/// Donating to an id that was never created fails with `PoolNotFound` (#1).
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_donate_to_nonexistent_pool_fails() {
    let env = Env::default();
    let (client, _) = client_for(&env);
    let donor = Address::generate(&env);

    client.donate(&4242u32, &donor, &100u128);
}

// ── 4. Closed and cancelled pools ─────────────────────────────────────────

/// A pool that has been closed rejects contributions with `PoolIsClosed` (#4).
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_donate_to_closed_pool_fails() {
    let env = Env::default();
    let (client, pool_id) = client_for(&env);
    let donor = Address::generate(&env);

    // `close_pool` only accepts a Disbursed or Cancelled pool, so cancel it
    // first. The closed check in `donate` is what fires afterwards.
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    client.close_pool(&pool_id);
    client.donate(&pool_id, &donor, &100u128);
}

/// A pool whose state is no longer `Active` rejects contributions with
/// `InvalidPoolState` (#2) — a different rejection from a closed pool, and the
/// state check has to survive alongside it.
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_donate_to_cancelled_pool_fails() {
    let env = Env::default();
    let (client, pool_id) = client_for(&env);
    let donor = Address::generate(&env);

    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    client.donate(&pool_id, &donor, &100u128);
}

// ── 5. Near-maximum amounts ───────────────────────────────────────────────

/// `u128::MAX` is recorded exactly rather than wrapping.
#[test]
fn test_donate_max_u128_is_recorded_exactly() {
    let env = Env::default();
    let (client, pool_id) = client_for(&env);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &u128::MAX);

    assert_eq!(client.get_total_raised(&pool_id), u128::MAX);
}

/// The contribution after `u128::MAX` overflows the checked add and is
/// rejected with the same message the token path uses, rather than silently
/// wrapping the pool total down to a small number.
#[test]
#[should_panic(expected = "Collected amount overflow")]
fn test_donate_past_max_u128_is_rejected() {
    let env = Env::default();
    let (client, pool_id) = client_for(&env);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &u128::MAX);
    client.donate(&pool_id, &donor, &1u128);
}

// ── 6. Total and donor count move together ────────────────────────────────

/// One call updates both the pool total and the donor count: a new donor
/// increments the count, a repeat donor leaves it alone, and the total tracks
/// every contribution either way.
#[test]
fn test_donate_updates_total_and_donor_count_together() {
    let env = Env::default();
    let (client, pool_id) = client_for(&env);
    let first = Address::generate(&env);
    let second = Address::generate(&env);

    client.donate(&pool_id, &first, &250u128);
    assert_eq!(client.get_total_raised(&pool_id), 250u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "the first contribution from a new donor must count them once"
    );

    client.donate(&pool_id, &first, &250u128);
    assert_eq!(client.get_total_raised(&pool_id), 500u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "a repeat contribution must not count the same donor twice"
    );

    client.donate(&pool_id, &second, &1u128);
    assert_eq!(client.get_total_raised(&pool_id), 501u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        2u32,
        "a second distinct donor must be counted"
    );
}
