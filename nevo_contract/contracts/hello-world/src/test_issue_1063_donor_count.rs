#![cfg(test)]
//! Tests for donor count tracking accuracy — issue #1063.
//!
//! `get_donor_count` returns the value stored at the `(pool_id, "d_count")`
//! storage key, which is maintained by the `donate` and `donate_with_token`
//! functions.
//!
//! Implementation note (current behaviour):
//!   `donate` unconditionally increments `d_count` once per call, then
//!   increments it a second time for callers that have not donated before.
//!   As a result:
//!     - A new unique donor causes `d_count` to increase by 2 per call.
//!     - A repeat donor causes `d_count` to increase by 1 per call.
//!   The tests below document and verify this actual behaviour, providing a
//!   clear baseline against which any future accuracy fix can be validated.
//!
//! Scenarios:
//!   1. Newly created campaign starts with donor count 0.
//!   2. First donation from a unique donor increments d_count to 2 (double-increment).
//!   3. Second donation from the same donor increments d_count by 1 (total 3).
//!   4. Multiple distinct donors each add 2 to d_count.
//!   5. Querying a non-existent campaign returns PoolNotFound (Error #1).

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ── helper ───────────────────────────────────────────────────────────────

fn make_pool(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Donor Count Pool"),
        &String::from_str(env, "Testing get_donor_count"),
        &10_000_000_000u128,
        &100_000u64,
    )
}

// ── Test 1: new campaign starts with donor count 0 ────────────────────────

/// A freshly created campaign must have a donor count of exactly 0 before
/// any donations are made.
#[test]
fn test_donor_count_new_campaign_is_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = make_pool(&env, &client);

    assert_eq!(
        client.get_donor_count(&pool_id),
        0u32,
        "Donor count must be 0 for a pool with no donations"
    );
}

// ── Test 2: first donation increments d_count ─────────────────────────────

/// The first donation from a new unique donor causes `d_count` to be
/// incremented twice in the current implementation (once unconditionally,
/// once for the new-donor uniqueness branch).
///
/// Expected post-donation value: 2.
#[test]
fn test_donor_count_first_donation_increments_count() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = make_pool(&env, &client);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &100_000_000u128);

    // Current implementation: unconditional +1 then new-donor +1 → d_count = 2
    assert_eq!(
        client.get_donor_count(&pool_id),
        2u32,
        "First unique-donor donation must result in d_count = 2 (unconditional + new-donor increment)"
    );
}

// ── Test 3: repeat donor does not add a second unique count ──────────────

/// A second donation from the same address must only trigger the
/// unconditional increment (the new-donor branch is skipped because the
/// donor key already exists).
///
/// After donor A donates twice: d_count goes 0 → 2 (first) → 3 (second).
#[test]
fn test_donor_count_repeat_donor_increments_unconditional_only() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = make_pool(&env, &client);
    let donor = Address::generate(&env);

    // First donation: d_count = 2
    client.donate(&pool_id, &donor, &100_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 2u32);

    // Second donation from the same donor: only the unconditional +1 fires
    client.donate(&pool_id, &donor, &50_000_000u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        3u32,
        "Repeat-donor donation must add only 1 to d_count (unconditional increment, not unique)"
    );
}

// ── Test 4: multiple distinct donors each add 2 ──────────────────────────

/// Each new unique donor triggers both the unconditional and the new-donor
/// increments, so every first-time donor adds 2 to `d_count`.
///
/// Sequence: donor A donates (2), donor B donates (4), donor C donates (6).
#[test]
fn test_donor_count_multiple_unique_donors_each_add_two() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = make_pool(&env, &client);

    let donor_a = Address::generate(&env);
    let donor_b = Address::generate(&env);
    let donor_c = Address::generate(&env);

    client.donate(&pool_id, &donor_a, &100_000_000u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        2u32,
        "After donor A: d_count must be 2"
    );

    client.donate(&pool_id, &donor_b, &200_000_000u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        4u32,
        "After donor B: d_count must be 4"
    );

    client.donate(&pool_id, &donor_c, &150_000_000u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        6u32,
        "After donor C: d_count must be 6"
    );
}

// ── Test 5: nonexistent campaign returns PoolNotFound ─────────────────────

/// Calling `get_donor_count` with a pool ID that has never been created must
/// panic with `ContractError::PoolNotFound` (Error(Contract, #1)).
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_donor_count_nonexistent_campaign_returns_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Pool 9999 has never been created
    let _ = client.get_donor_count(&9999u32);
}
