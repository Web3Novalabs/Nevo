#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    Address, Env,
};

/// Test is_school_registered returns false for unregistered address.
#[test]
fn test_is_school_registered_unregistered_address() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let school = Address::generate(&env);
    assert_eq!(client.is_school_registered(&school), false);
}

/// Test is_school_registered returns true immediately after register_school.
#[test]
fn test_is_school_registered_immediately_after_registration() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);

    // Set admin first
    client.set_admin(&admin);

    // Register the school
    client.register_school(&admin, &school);

    // Verify it's immediately registered
    assert_eq!(client.is_school_registered(&school), true);
}

/// Test is_school_registered is independent across multiple distinct school addresses.
#[test]
fn test_is_school_registered_independent_schools() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school1 = Address::generate(&env);
    let school2 = Address::generate(&env);
    let school3 = Address::generate(&env);

    client.set_admin(&admin);

    // Register only school1
    client.register_school(&admin, &school1);

    // Check registration status
    assert_eq!(client.is_school_registered(&school1), true);
    assert_eq!(client.is_school_registered(&school2), false);
    assert_eq!(client.is_school_registered(&school3), false);

    // Register school2
    client.register_school(&admin, &school2);

    // Check registration status again
    assert_eq!(client.is_school_registered(&school1), true);
    assert_eq!(client.is_school_registered(&school2), true);
    assert_eq!(client.is_school_registered(&school3), false);

    // Register school3
    client.register_school(&admin, &school3);

    // Check final registration status
    assert_eq!(client.is_school_registered(&school1), true);
    assert_eq!(client.is_school_registered(&school2), true);
    assert_eq!(client.is_school_registered(&school3), true);
}

/// Test is_school_registered is unaffected by pool creation activity.
#[test]
fn test_is_school_registered_unaffected_by_pool_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);

    client.set_admin(&admin);

    // Before registration, registration status is false
    assert_eq!(client.is_school_registered(&school), false);

    // Register the school
    client.register_school(&admin, &school);
    assert_eq!(client.is_school_registered(&school), true);

    // Create a pool linked to the school
    let pool_id = client.create_pool_for_school(
        &creator,
        &soroban_sdk::String::from_str(&env, "School Pool"),
        &soroban_sdk::String::from_str(&env, "Pool for school"),
        &5_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Verify registration status is still true after pool creation
    assert_eq!(client.is_school_registered(&school), true);

    // Create another pool linked to the same school
    client.create_pool_for_school(
        &creator,
        &soroban_sdk::String::from_str(&env, "Another School Pool"),
        &soroban_sdk::String::from_str(&env, "Another pool for school"),
        &3_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Verify registration status remains true
    assert_eq!(client.is_school_registered(&school), true);
}

/// Test is_school_registered gives consistent result across repeated calls.
#[test]
fn test_is_school_registered_consistent_repeated_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Call is_school_registered multiple times and verify consistency
    for _ in 0..5 {
        assert_eq!(client.is_school_registered(&school), true);
    }

    let unregistered = Address::generate(&env);

    // Check unregistered school multiple times
    for _ in 0..5 {
        assert_eq!(client.is_school_registered(&unregistered), false);
    }
}

/// Test is_school_registered with multiple registrations and queries.
#[test]
fn test_is_school_registered_multiple_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let schools: Vec<Address> = (0..5)
        .map(|_| Address::generate(&env))
        .collect();

    client.set_admin(&admin);

    // Register first two schools
    client.register_school(&admin, &schools[0]);
    client.register_school(&admin, &schools[1]);

    // Verify status
    assert_eq!(client.is_school_registered(&schools[0]), true);
    assert_eq!(client.is_school_registered(&schools[1]), true);
    assert_eq!(client.is_school_registered(&schools[2]), false);
    assert_eq!(client.is_school_registered(&schools[3]), false);

    // Register remaining schools
    client.register_school(&admin, &schools[2]);
    client.register_school(&admin, &schools[3]);
    client.register_school(&admin, &schools[4]);

    // Verify all are now registered
    for school in &schools {
        assert_eq!(client.is_school_registered(school), true);
    }
}

/// Test is_school_registered persistence across different operations.
#[test]
fn test_is_school_registered_persistence() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    client.set_admin(&admin);

    // Verify not registered initially
    assert_eq!(client.is_school_registered(&school), false);

    // Register school
    client.register_school(&admin, &school);
    assert_eq!(client.is_school_registered(&school), true);

    // Create a school-linked pool
    let pool_id = client.create_pool_for_school(
        &creator,
        &soroban_sdk::String::from_str(&env, "School Pool"),
        &soroban_sdk::String::from_str(&env, "Pool for school"),
        &5_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Make donations to the pool
    client.donate(&pool_id, &donor, &1_000_000_000u128);

    // Verify registration status persists
    assert_eq!(client.is_school_registered(&school), true);

    // Verify with get_pool_school that school link is maintained
    let linked_school = client.get_pool_school(&pool_id);
    assert_eq!(linked_school, school);

    // Verify registration still true after pool operations
    assert_eq!(client.is_school_registered(&school), true);
}
