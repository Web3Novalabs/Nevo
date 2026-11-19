#![cfg(test)]
//! Tests for `get_campaign_balance` edge cases — issue #1269 (and #1061).
//!
//! Covers:
//!   1. New campaign returns 0 balance.
//!   2. Campaign with multiple donations returns the correct accumulated total.
//!   3. Nonexistent campaign returns `ContractError::PoolNotFound` (`Error(Contract, #1)`).
//!   4. Campaign balance precisely matches the mathematical sum of all individual donations.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ── helpers ──────────────────────────────────────────────────────────────

/// Create a pool and return its ID. Uses sensible defaults so test bodies
/// stay focused on the behaviour under test.
fn setup_campaign(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Test Campaign"),
        &String::from_str(env, "A campaign used in unit tests"),
        &10_000_000_000u128, // goal: 10 billion stroops
        &100_000u64,         // application deadline (ledger sequence)
    )
}

// ── Test 1: new campaign returns 0 balance ────────────────────────────────

/// A freshly created campaign must report a balance of exactly 0 before
/// any donations have been made.
#[test]
fn test_new_campaign_balance_is_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    assert_eq!(
        client.get_campaign_balance(&pool_id),
        0u128,
        "A newly created campaign must start with a balance of 0"
    );
}

// ── Test 2: campaign with donations returns correct accumulated total ─────

/// After multiple donations the reported balance must equal their running sum.
/// Each subsequent call to `donate` must increment the stored total, not
/// replace it.
#[test]
fn test_campaign_balance_accumulates_multiple_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);

    let amount1 = 100_000_000u128; // 100 XLM in stroops
    let amount2 = 250_000_000u128; // 250 XLM in stroops
    let amount3 = 50_000_000u128;  //  50 XLM in stroops

    client.donate(&pool_id, &donor1, &amount1);
    // Balance must reflect the single donation immediately.
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1,
        "Balance after first donation must equal that donation"
    );

    client.donate(&pool_id, &donor2, &amount2);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1 + amount2,
        "Balance must accumulate after second donation"
    );

    client.donate(&pool_id, &donor3, &amount3);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1 + amount2 + amount3,
        "Balance must accumulate after third donation"
    );
}

// ── Test 3: querying a nonexistent campaign returns CampaignNotFound ──────

/// Calling `get_campaign_balance` with an ID that has never been created
/// must panic with `ContractError::PoolNotFound` (encoded as
/// `Error(Contract, #1)` in the Soroban XDR error format).
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_campaign_balance_nonexistent_campaign_returns_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Pool ID 9999 has never been created — must panic with PoolNotFound.
    let _ = client.get_campaign_balance(&9999u32);
}

/// Calling `try_get_campaign_balance` with a nonexistent campaign ID returns
/// `ContractError::PoolNotFound` error variant.
#[test]
fn test_campaign_balance_nonexistent_campaign_try_returns_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let res = client.try_get_campaign_balance(&9999u32);
    assert_eq!(
        res,
        Err(Ok(ContractError::PoolNotFound)),
        "Nonexistent campaign must return ContractError::PoolNotFound"
    );
}

// ── Test 4: balance equals mathematical sum of all donations ─────────────

/// The stored campaign balance must equal the exact arithmetic sum of every
/// individual donation amount, with no rounding, truncation, or off-by-one
/// errors. A larger set of donations surfaces any accumulation bugs.
#[test]
fn test_campaign_balance_equals_sum_of_all_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    // Individual donation amounts (in stroops)
    let donation_amounts: &[u128] = &[
        1u128,
        999u128,
        500_000_000u128,
        1_234_567_890u128,
        7u128,
        100_000_000u128,
        42u128,
        333_333_333u128,
    ];

    let mut expected_total: u128 = 0;
    for &amount in donation_amounts {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &amount);
        expected_total += amount;
    }

    let reported_balance = client.get_campaign_balance(&pool_id);

    assert_eq!(
        reported_balance,
        expected_total,
        "Campaign balance ({}) must exactly equal the arithmetic sum of all donations ({})",
        reported_balance,
        expected_total
    );
}
