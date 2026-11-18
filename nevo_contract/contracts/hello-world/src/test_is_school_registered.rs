#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    Address, BytesN, Env,
};

/// Test is_school_registered returns false for unregistered address.
#[test]
fn test_is_school_registered_unregistered_address() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let school = Address::generate(&env);
    assert!(!client.is_school_registered(&school));
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
    client.register_school(&school, &BytesN::from_array(&env, &[1u8; 32]));

    // Verify it's immediately registered
    assert!(client.is_school_registered(&school));
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
    client.register_school(&school1, &BytesN::from_array(&env, &[1u8; 32]));

    // Check registration status
    assert!(client.is_school_registered(&school1));
    assert!(!client.is_school_registered(&school2));
    assert!(!client.is_school_registered(&school3));

    // Register school2
    client.register_school(&school2, &BytesN::from_array(&env, &[1u8; 32]));

    // Check registration status again
    assert!(client.is_school_registered(&school1));
    assert!(client.is_school_registered(&school2));
    assert!(!client.is_school_registered(&school3));

    // Register school3
    client.register_school(&school3, &BytesN::from_array(&env, &[1u8; 32]));

    // Check final registration status
    assert!(client.is_school_registered(&school1));
    assert!(client.is_school_registered(&school2));
    assert!(client.is_school_registered(&school3));
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
    assert!(!client.is_school_registered(&school));

    // Register the school
    client.register_school(&school, &BytesN::from_array(&env, &[1u8; 32]));
    assert!(client.is_school_registered(&school));

    // Create a pool linked to the school
    let _pool_id = client.create_pool_for_school(
        &creator,
        &soroban_sdk::String::from_str(&env, "School Pool"),
        &soroban_sdk::String::from_str(&env, "Pool for school"),
        &5_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Verify registration status is still true after pool creation
    assert!(client.is_school_registered(&school));

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
    assert!(client.is_school_registered(&school));
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
    client.register_school(&school, &BytesN::from_array(&env, &[1u8; 32]));

    // Call is_school_registered multiple times and verify consistency
    for _ in 0..5 {
        assert!(client.is_school_registered(&school));
    }

    let unregistered = Address::generate(&env);

    // Check unregistered school multiple times
    for _ in 0..5 {
        assert!(!client.is_school_registered(&unregistered));
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
    let schools: std::vec::Vec<Address> = (0..5)
        .map(|_| Address::generate(&env))
        .collect();

    client.set_admin(&admin);

    // Register first two schools
    client.register_school(&schools[0], &BytesN::from_array(&env, &[1u8; 32]));
    client.register_school(&schools[1], &BytesN::from_array(&env, &[1u8; 32]));

    // Verify status
    assert!(client.is_school_registered(&schools[0]));
    assert!(client.is_school_registered(&schools[1]));
    assert!(!client.is_school_registered(&schools[2]));
    assert!(!client.is_school_registered(&schools[3]));

    // Register remaining schools
    client.register_school(&schools[2], &BytesN::from_array(&env, &[1u8; 32]));
    client.register_school(&schools[3], &BytesN::from_array(&env, &[1u8; 32]));
    client.register_school(&schools[4], &BytesN::from_array(&env, &[1u8; 32]));

    // Verify all are now registered
    for school in &schools {
        assert!(client.is_school_registered(school));
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
    assert!(!client.is_school_registered(&school));

    // Register school
    client.register_school(&school, &BytesN::from_array(&env, &[1u8; 32]));
    assert!(client.is_school_registered(&school));

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
    assert!(client.is_school_registered(&school));

    // Verify with get_pool_school that school link is maintained
    let linked_school = client.get_pool_school(&pool_id);
    assert_eq!(linked_school, school);

    // Verify registration still true after pool operations
    assert!(client.is_school_registered(&school));
}
