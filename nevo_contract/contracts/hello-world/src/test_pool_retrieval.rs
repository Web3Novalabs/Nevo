#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, Address, Env, String,
};

// ============= ISSUE #1089: POOL RETRIEVAL VALIDATION TESTS =============

/// Test 1: Existing pool returns correct config
#[test]
fn test_get_pool_existing_returns_correct_config() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = 5_000_000_000u128;
    let deadline = 300_000u64;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Scholarship Fund"),
        &String::from_str(&env, "For university students"),
        &goal,
        &deadline,
    );

    let (id, sponsor, pool_goal, collected, is_closed, app_deadline) = client.get_pool(&pool_id);
    assert_eq!(id, pool_id);
    assert_eq!(sponsor, creator);
    assert_eq!(pool_goal, goal);
    assert_eq!(collected, 0u128);
    assert_eq!(is_closed, false);
    assert_eq!(app_deadline, deadline);
}

/// Test 2: Nonexistent pool panics with PoolNotFound
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_get_pool_nonexistent_panics() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.get_pool(&999);
}

/// Test 3: Pool data matches creation parameters
#[test]
fn test_get_pool_data_matches_creation_parameters() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Medical Emergency Fund");
    let description = String::from_str(&env, "Urgent medical treatment needed");
    let goal = 25_000_000_000u128;
    let deadline = 500_000u64;

    let pool_id = client.create_pool(&creator, &title, &description, &goal, &deadline);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, goal);

    let (stored_title, stored_desc) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, title);
    assert_eq!(stored_desc, description);
}

/// Test 4: Multiple pools are retrieved independently
#[test]
fn test_get_pool_multiple_independently() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let id1 = client.create_pool(
        &creator1,
        &String::from_str(&env, "Pool One"),
        &String::from_str(&env, "First pool"),
        &1_000_000_000u128,
        &100_000u64,
    );
    let id2 = client.create_pool(
        &creator2,
        &String::from_str(&env, "Pool Two"),
        &String::from_str(&env, "Second pool"),
        &2_000_000_000u128,
        &200_000u64,
    );

    let pool1 = client.get_pool(&id1);
    let pool2 = client.get_pool(&id2);

    assert_eq!(pool1.1, creator1);
    assert_eq!(pool2.1, creator2);
    assert_eq!(pool1.2, 1_000_000_000u128);
    assert_eq!(pool2.2, 2_000_000_000u128);
    assert_eq!(pool1.3, 0u128);
    assert_eq!(pool2.3, 0u128);
}

/// Test 5: Get pool metadata for existing pool
#[test]
fn test_get_pool_metadata_existing() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let title = String::from_str(&env, "Research Grant");
    let desc = String::from_str(&env, "Climate change research funding");
    let pool_id = client.create_pool(
        &Address::generate(&env),
        &title,
        &desc,
        &10_000_000u128,
        &100_000u64,
    );

    let (t, d) = client.get_pool_metadata(&pool_id);
    assert_eq!(t, title);
    assert_eq!(d, desc);
}

/// Test 6: Get pool metadata for nonexistent pool returns empty strings
#[test]
fn test_get_pool_metadata_nonexistent_returns_empty() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (t, d) = client.get_pool_metadata(&999);
    assert_eq!(t, String::from_str(&env, ""));
    assert_eq!(d, String::from_str(&env, ""));
}

/// Test 7: Pool count starts at zero
#[test]
fn test_get_pool_count_initial_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_pool_count(), 0);
}

/// Test 8: Pool count increments with each pool creation
#[test]
fn test_get_pool_count_increments() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool A"),
        &String::from_str(&env, "Desc A"),
        &1_000_000u128,
        &100_000u64,
    );
    assert_eq!(client.get_pool_count(), 1);

    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool B"),
        &String::from_str(&env, "Desc B"),
        &2_000_000u128,
        &200_000u64,
    );
    assert_eq!(client.get_pool_count(), 2);
}

/// Test 9: Total raised starts at zero and reflects donations
#[test]
fn test_get_total_raised_after_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Donation Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_total_raised(&pool_id), 0u128);

    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    client.donate(&pool_id, &donor1, &50_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 50_000_000u128);

    client.donate(&pool_id, &donor2, &30_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 80_000_000u128);
}

/// Test 10: Individual contribution tracking across multiple donations
#[test]
fn test_get_contribution_tracks_individual_amounts() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Contribution Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let donor = Address::generate(&env);

    assert_eq!(client.get_contribution(&pool_id, &donor), 0u128);

    client.donate(&pool_id, &donor, &100_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 100_000_000u128);

    // Second donation from same donor adds to total
    client.donate(&pool_id, &donor, &50_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 150_000_000u128);
}

/// Test 11: Donor count after multiple unique donors
#[test]
fn test_get_donor_count_after_unique_donors() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Donor Count Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_donor_count(&pool_id), 0);

    let donor1 = Address::generate(&env);
    client.donate(&pool_id, &donor1, &10_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);

    let donor2 = Address::generate(&env);
    client.donate(&pool_id, &donor2, &20_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 2);

    client.donate(&pool_id, &donor1, &30_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 2);
}
