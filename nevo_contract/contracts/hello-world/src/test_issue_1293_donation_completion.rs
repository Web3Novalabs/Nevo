#![cfg(test)]

// ============= ISSUE #1293: CAMPAIGN DONATION COMPLETION CHECKS =============
//
// `donate` and `donate_with_token` gate on `pool.is_closed` and
// `pool.state != Active` only — neither checks `collected` against `goal`.
// Reaching or exceeding the goal does not transition pool state and does
// not block further donations. These tests verify that actual behavior.

use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// A donation to a pool that has not yet reached its goal succeeds.
#[test]
fn test_donation_to_incomplete_campaign_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &200_000u64,
    );

    client.donate(&pool_id, &donor, &400_000u128);
    assert_eq!(client.get_pool(&pool_id).3, 400_000u128);
}

/// A donation that reaches or exceeds the pool's goal succeeds — there is
/// no cap enforced at the goal amount.
#[test]
fn test_donation_that_reaches_or_exceeds_goal_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 1_000_000u128;
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Description"),
        &goal,
        &200_000u64,
    );

    client.donate(&pool_id, &donor, &1_500_000u128);
    assert_eq!(client.get_pool(&pool_id).3, 1_500_000u128);
    assert!(client.get_pool(&pool_id).3 > goal);
}

/// Once a pool's collected total has passed its goal, further donations
/// still succeed and continue to accumulate — nothing rejects them.
#[test]
fn test_subsequent_donations_after_goal_reached_continue_to_succeed() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 1_000_000u128;
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Description"),
        &goal,
        &200_000u64,
    );

    client.donate(&pool_id, &donor, &goal);
    assert_eq!(client.get_pool(&pool_id).3, goal);

    // A second donation, made after the goal has already been reached,
    // still succeeds and keeps accumulating.
    client.donate(&pool_id, &donor, &250_000u128);
    assert_eq!(client.get_pool(&pool_id).3, goal + 250_000u128);
}

/// The same lack of a completion check applies to `donate_with_token`:
/// donations past the goal succeed, including via the token-backed path.
#[test]
fn test_donate_with_token_also_accepts_donations_past_goal() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 1_000_000u128;
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Description"),
        &goal,
        &200_000u64,
    );

    let token = create_token(&env, 2_000_000i128, &donor);
    client.donate_with_token(&pool_id, &donor, &token, &1_200_000i128);
    assert_eq!(client.get_pool(&pool_id).3, 1_200_000u128);

    // A further donation after the goal is already exceeded still succeeds.
    client.donate_with_token(&pool_id, &donor, &token, &100_000i128);
    assert_eq!(client.get_pool(&pool_id).3, 1_300_000u128);
}
