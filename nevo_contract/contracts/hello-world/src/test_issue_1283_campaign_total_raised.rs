//! Tests for `get_total_raised` — issue #1283
//!
//! Covers:
//!   1. New campaign returns 0.
//!   2. Single donation updates total.
//!   3. Multiple donations sum correctly.
//!   4. Total matches campaign balance (pool.collected).
//!   5. Nonexistent campaign returns error (ContractError #1).

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// (1) New campaign returns 0.
#[test]
fn test_get_total_raised_new_campaign_returns_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Brand New Campaign"),
        &String::from_str(&env, "No donations yet"),
        &1_000_000_000u128,
        &200_000u64,
    );

    assert_eq!(
        client.get_total_raised(&campaign_id),
        0u128,
        "A freshly created campaign must report zero total raised"
    );
}

// (2) Single donation updates total.
#[test]
fn test_get_total_raised_single_donation_updates_total() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let donation_amount = 250_000_000u128;

    let campaign_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Single Donor Campaign"),
        &String::from_str(&env, "One contribution"),
        &1_000_000_000u128,
        &200_000u64,
    );

    client.donate(&campaign_id, &donor, &donation_amount);

    assert_eq!(
        client.get_total_raised(&campaign_id),
        donation_amount,
        "Total raised must equal the single donation amount"
    );
}

// (3) Multiple donations sum correctly.
#[test]
fn test_get_total_raised_multiple_donations_sum_correctly() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let amount_1 = 100_000_000u128;
    let amount_2 = 200_000_000u128;
    let amount_3 = 50_000_000u128;
    let expected_total = amount_1 + amount_2 + amount_3;

    let campaign_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Multi-Donor Campaign"),
        &String::from_str(&env, "Three contributors"),
        &1_000_000_000u128,
        &200_000u64,
    );

    client.donate(&campaign_id, &Address::generate(&env), &amount_1);
    client.donate(&campaign_id, &Address::generate(&env), &amount_2);
    client.donate(&campaign_id, &Address::generate(&env), &amount_3);

    assert_eq!(
        client.get_total_raised(&campaign_id),
        expected_total,
        "Total raised must be the arithmetic sum of all individual donations"
    );
}

// (4) Total raised matches campaign balance (pool.collected).
#[test]
fn test_get_total_raised_matches_campaign_balance() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donation_a = 300_000_000u128;
    let donation_b = 700_000_000u128;

    let campaign_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Balance Check Campaign"),
        &String::from_str(&env, "Verifying internal consistency"),
        &2_000_000_000u128,
        &200_000u64,
    );

    client.donate(&campaign_id, &Address::generate(&env), &donation_a);
    client.donate(&campaign_id, &Address::generate(&env), &donation_b);

    let total_raised = client.get_total_raised(&campaign_id);
    // get_pool returns (id, sponsor, goal, collected, is_closed, deadline)
    let pool = client.get_pool(&campaign_id);
    let pool_collected = pool.3;

    assert_eq!(
        total_raised, pool_collected,
        "get_total_raised must equal pool.collected reported by get_pool"
    );
    assert_eq!(total_raised, donation_a + donation_b);
}

// (5) Nonexistent campaign returns error (ContractError #1 — PoolNotFound).
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_get_total_raised_nonexistent_campaign_returns_error() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // No campaign created; id 42 does not exist.
    client.get_total_raised(&42u32);
}
