//! Tests for `get_campaign_goal` — issue #1282
//!
//! Covers:
//!   1. Returns correct goal for an existing campaign.
//!   2. Nonexistent campaign returns CampaignNotFound (error #1).
//!   3. Goal matches the value set during creation.
//!   4. Multiple campaigns have independent goals.

#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// (1) Returns correct goal for an existing campaign.
#[test]
fn test_get_campaign_goal_returns_correct_goal() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = 5_000_000_000u128;

    let campaign_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Education Fund"),
        &String::from_str(&env, "Funding students worldwide"),
        &goal,
        &200_000u64,
    );

    assert_eq!(client.get_campaign_goal(&campaign_id), goal);
}

// (2) Nonexistent campaign returns CampaignNotFound (ContractError #1).
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_get_campaign_goal_nonexistent_campaign_returns_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // No campaign has been created; id 999 does not exist.
    client.get_campaign_goal(&999u32);
}

// (3) Goal matches the value set during creation.
#[test]
fn test_get_campaign_goal_matches_creation_value() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let expected_goal = 1_234_567_890u128;

    let campaign_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Specific Goal Campaign"),
        &String::from_str(&env, "Goal must be preserved exactly"),
        &expected_goal,
        &200_000u64,
    );

    // Verify via get_campaign_goal
    let returned_goal = client.get_campaign_goal(&campaign_id);
    assert_eq!(
        returned_goal, expected_goal,
        "get_campaign_goal must return exactly the goal stored at creation"
    );

    // Cross-check with get_pool to confirm consistency
    let pool = client.get_pool(&campaign_id);
    assert_eq!(pool.2, expected_goal);
    assert_eq!(returned_goal, pool.2);
}

// (4) Multiple campaigns have independent goals.
#[test]
fn test_get_campaign_goal_multiple_campaigns_have_independent_goals() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let goal_a = 1_000_000u128;
    let goal_b = 9_999_999u128;
    let goal_c = 500_000_000_000u128;

    let id_a = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Campaign Alpha"),
        &String::from_str(&env, "First campaign"),
        &goal_a,
        &200_000u64,
    );

    let id_b = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Campaign Beta"),
        &String::from_str(&env, "Second campaign"),
        &goal_b,
        &200_000u64,
    );

    let id_c = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Campaign Gamma"),
        &String::from_str(&env, "Third campaign"),
        &goal_c,
        &200_000u64,
    );

    // Each campaign must return its own independently stored goal.
    assert_eq!(client.get_campaign_goal(&id_a), goal_a);
    assert_eq!(client.get_campaign_goal(&id_b), goal_b);
    assert_eq!(client.get_campaign_goal(&id_c), goal_c);

    // Goals must not bleed into one another.
    assert_ne!(
        client.get_campaign_goal(&id_a),
        client.get_campaign_goal(&id_b)
    );
    assert_ne!(
        client.get_campaign_goal(&id_b),
        client.get_campaign_goal(&id_c)
    );
}
