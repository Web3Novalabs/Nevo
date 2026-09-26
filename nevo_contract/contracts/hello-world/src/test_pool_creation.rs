#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, Address, Env, String,
};

// ============= ISSUE #1088: POOL CREATION WITH create_pool FUNCTION TESTS =============

/// Test 1: Valid config creates pool successfully
#[test]
fn test_create_pool_valid_config() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Education Fund"),
        &String::from_str(&env, "Supporting students in need"),
        &5_000_000_000u128,
        &200_000u64,
    );

    assert_eq!(pool_id, 1);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, 5_000_000_000u128);
    assert_eq!(pool.3, 0u128);
    assert_eq!(pool.4, false);
    assert_eq!(pool.5, 200_000u64);
}

/// Test 2: Invalid description (too long) fails validation
#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_create_pool_invalid_description_too_long() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let long_desc = String::from_str(&env, &"x".repeat(501));
    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Title"),
        &long_desc,
        &1_000_000_000u128,
        &100_000u64,
    );
}

/// Test 3: Pool ID increments correctly across multiple creations
#[test]
fn test_create_pool_id_increments() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let id1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "First"),
        &1_000_000u128,
        &100_000u64,
    );
    let id2 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Second"),
        &2_000_000u128,
        &100_000u64,
    );
    let id3 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 3"),
        &String::from_str(&env, "Third"),
        &3_000_000u128,
        &100_000u64,
    );

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
}

/// Test 4: Pool state is initialized as Active
#[test]
fn test_create_pool_state_initialized_active() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Active Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Pool should be in Active state - donate should succeed (Active is the only accepting state)
    let donor = Address::generate(&env);
    client.donate(&pool_id, &donor, &100_000_000u128);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000u128);
    assert_eq!(pool.4, false); // is_closed = false
}

/// Test 5: Pool metrics are initialized correctly
#[test]
fn test_create_pool_metrics_initialized_correctly() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool"),
        &String::from_str(&env, "Check metrics"),
        &10_000_000_000u128,
        &100_000u64,
    );

    // Verify all initial metrics
    assert_eq!(client.get_pool_count(), 1);
    assert_eq!(client.get_total_raised(&pool_id), 0u128);
    assert_eq!(client.get_donor_count(&pool_id), 0u32);
}

/// Test 6: Pool metadata is stored and retrievable
#[test]
fn test_create_pool_metadata_stored() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Emergency Relief");
    let description = String::from_str(&env, "Helping flood victims in the region");

    let pool_id = client.create_pool(&creator, &title, &description, &1_000_000_000u128, &100_000u64);

    let (stored_title, stored_desc) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, title);
    assert_eq!(stored_desc, description);
}

/// Test 7: Pool with zero goal is rejected
#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_create_pool_zero_goal() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Zero Goal Pool"),
        &String::from_str(&env, "Testing zero goal"),
        &0u128,
        &100_000u64,
    );
}

/// Test 8: Maximum description length (500) is accepted
#[test]
fn test_create_pool_max_description_length() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let max_desc = String::from_str(&env, &"x".repeat(500));
    let pool_id = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Title"),
        &max_desc,
        &1_000_000u128,
        &100_000u64,
    );

    let (_, stored_desc) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_desc.len(), 500);
}

/// Test 9: Description of 501 characters fails
#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_create_pool_501_char_description_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let long_desc = String::from_str(&env, &"a".repeat(501));
    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Title"),
        &long_desc,
        &1_000_000u128,
        &100_000u64,
    );
}

/// Test 10: Pool count tracks total created pools
#[test]
fn test_create_pool_pool_count_tracks_total() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_pool_count(), 0);

    let creator = Address::generate(&env);
    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "Desc"),
        &1_000_000u128,
        &100_000u64,
    );
    assert_eq!(client.get_pool_count(), 1);

    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Desc"),
        &2_000_000u128,
        &100_000u64,
    );
    assert_eq!(client.get_pool_count(), 2);
}

/// Test 11: Invalid config with empty title fails validation
#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_create_pool_invalid_empty_title() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Valid description"),
        &1_000_000u128,
        &100_000u64,
    );
}

/// Test 12: Invalid config with empty description fails validation
#[test]
#[should_panic(expected = "Description cannot be empty")]
fn test_create_pool_invalid_empty_description() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Valid Title"),
        &String::from_str(&env, ""),
        &1_000_000u128,
        &100_000u64,
    );
}

/// Test 13: Invalid config with zero duration fails validation
#[test]
#[should_panic(expected = "Duration must be greater than zero")]
fn test_create_pool_invalid_zero_duration() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Valid Title"),
        &String::from_str(&env, "Valid description"),
        &1_000_000u128,
        &0u64,
    );
}

/// Test 14: Pool state is Active immediately after creation (before any donation)
#[test]
fn test_create_pool_state_active_before_donation() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Fresh Pool"),
        &String::from_str(&env, "Just created"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // A freshly created pool is Active: not closed and no funds raised yet
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, false); // is_closed = false
    assert_eq!(pool.3, 0u128); // total_raised = 0
}

/// Test 15: Pool metrics start at zero for a newly created pool
#[test]
fn test_create_pool_metrics_start_at_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Zero Metrics Pool"),
        &String::from_str(&env, "Metrics start at zero"),
        &5_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_total_raised(&pool_id), 0u128);
    assert_eq!(client.get_donor_count(&pool_id), 0u32);
    assert_eq!(client.get_pool_count(), 1);
}
