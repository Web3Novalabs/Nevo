#![cfg(test)]
//! Tests for campaign contribution getter validation — issue #1270.
//!
//! Covers `get_contribution` function:
//!   (1) Contributor with no donations returns 0.
//!   (2) Contributor with multiple donations returns sum.
//!   (3) Nonexistent contributor returns 0.
//!   (4) Nonexistent campaign returns CampaignNotFound (PoolNotFound / Error #1).
//!   (5) Multiple contributors tracked separately.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn setup_pool(env: &Env, client: &ContractClient) -> (u32, Address) {
    let creator = Address::generate(env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(env, "Contribution Validation Pool"),
        &String::from_str(env, "Tests for get_contribution getter validation"),
        &10_000_000_000u128,
        &100_000u64,
    );
    (pool_id, creator)
}

// ── Test 1: Contributor with no donations returns 0 ─────────────────────────

#[test]
fn test_contributor_with_no_donations_returns_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);
    let contributor = Address::generate(&env);

    assert_eq!(
        client.get_contribution(&pool_id, &contributor),
        0u128,
        "A contributor with no recorded donations must return 0"
    );
}

// ── Test 2: Contributor with multiple donations returns sum ─────────────────

#[test]
fn test_contributor_with_multiple_donations_returns_sum() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);
    let contributor = Address::generate(&env);

    let donation1 = 100_000_000u128;
    let donation2 = 250_000_000u128;
    let donation3 = 450_000_000u128;

    client.donate(&pool_id, &contributor, &donation1);
    assert_eq!(client.get_contribution(&pool_id, &contributor), donation1);

    client.donate(&pool_id, &contributor, &donation2);
    assert_eq!(
        client.get_contribution(&pool_id, &contributor),
        donation1 + donation2
    );

    client.donate(&pool_id, &contributor, &donation3);
    assert_eq!(
        client.get_contribution(&pool_id, &contributor),
        donation1 + donation2 + donation3,
        "Accumulated donations from a single contributor must equal their exact sum"
    );
}

// ── Test 3: Nonexistent contributor returns 0 ───────────────────────────────

#[test]
fn test_nonexistent_contributor_returns_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);

    // Active contributor makes a donation
    let active_contributor = Address::generate(&env);
    client.donate(&pool_id, &active_contributor, &1_000_000_000u128);

    // Nonexistent contributor who never interacted with the contract
    let nonexistent_contributor = Address::generate(&env);
    assert_eq!(
        client.get_contribution(&pool_id, &nonexistent_contributor),
        0u128,
        "A contributor who has never interacted with the pool must return 0"
    );
}

// ── Test 4: Nonexistent campaign returns CampaignNotFound ───────────────────

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_nonexistent_campaign_returns_campaign_not_found_panic() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let contributor = Address::generate(&env);
    let _ = client.get_contribution(&9999u32, &contributor);
}

#[test]
fn test_nonexistent_campaign_returns_campaign_not_found_result() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let contributor = Address::generate(&env);
    let result = client.try_get_contribution(&9999u32, &contributor);
    assert_eq!(
        result,
        Err(Ok(ContractError::PoolNotFound)),
        "Nonexistent campaign must return ContractError::PoolNotFound"
    );
}

// ── Test 5: Multiple contributors tracked separately ────────────────────────

#[test]
fn test_multiple_contributors_tracked_separately() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);

    let contributor_a = Address::generate(&env);
    let contributor_b = Address::generate(&env);
    let contributor_c = Address::generate(&env);

    let amount_a1 = 150_000_000u128;
    let amount_b = 300_000_000u128;
    let amount_a2 = 50_000_000u128;
    let amount_c = 750_000_000u128;

    client.donate(&pool_id, &contributor_a, &amount_a1);
    client.donate(&pool_id, &contributor_b, &amount_b);
    client.donate(&pool_id, &contributor_a, &amount_a2);
    client.donate(&pool_id, &contributor_c, &amount_c);

    assert_eq!(
        client.get_contribution(&pool_id, &contributor_a),
        amount_a1 + amount_a2,
        "Contributor A's total must be tracked independently"
    );
    assert_eq!(
        client.get_contribution(&pool_id, &contributor_b),
        amount_b,
        "Contributor B's total must be tracked independently"
    );
    assert_eq!(
        client.get_contribution(&pool_id, &contributor_c),
        amount_c,
        "Contributor C's total must be tracked independently"
    );
}
