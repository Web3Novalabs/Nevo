#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, String,
};

/// Test create_pool_for_school: Only registered school can create its own pool.
#[test]
fn test_create_pool_for_school_registered_school() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Registered school should be able to create a pool
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Registered School Pool"),
        &String::from_str(&env, "For registered school"),
        &5_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Verify pool was created
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);

    // Verify school linkage
    let linked_school = client.get_pool_school(&pool_id);
    assert_eq!(linked_school, school);
}

/// Test create_pool_for_school: Unregistered address fails.
#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_create_pool_for_school_unregistered_address_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let unregistered_school = Address::generate(&env);

    // Attempt to create pool for unregistered school should fail
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Unauthorized Pool"),
        &String::from_str(&env, "Should fail"),
        &5_000_000_000u128,
        &unregistered_school,
        &100_000u64,
    );
}

/// Test create_pool_for_school: Pool correctly linked to calling school via get_pool_school.
#[test]
fn test_create_pool_for_school_correct_linkage() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school1 = Address::generate(&env);
    let school2 = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school1);
    client.register_school(&admin, &school2);

    // Create pools for each school
    let pool_id_1 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School 1 Pool"),
        &String::from_str(&env, "For school 1"),
        &5_000_000_000u128,
        &school1,
        &100_000u64,
    );

    let pool_id_2 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School 2 Pool"),
        &String::from_str(&env, "For school 2"),
        &7_000_000_000u128,
        &school2,
        &100_000u64,
    );

    // Verify correct linkage
    assert_eq!(client.get_pool_school(&pool_id_1), school1);
    assert_eq!(client.get_pool_school(&pool_id_2), school2);
    assert_ne!(client.get_pool_school(&pool_id_1), school2);
}

/// Test create_pool_for_school: Pool ID sequencing consistent with create_pool.
#[test]
fn test_create_pool_for_school_sequencing() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Create regular pool
    let pool_id_1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Regular Pool"),
        &String::from_str(&env, "Regular pool"),
        &5_000_000_000u128,
        &100_000u64,
    );

    // Create school pool
    let pool_id_2 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "School pool"),
        &3_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Create another regular pool
    let pool_id_3 = client.create_pool(
        &creator,
        &String::from_str(&env, "Regular Pool 2"),
        &String::from_str(&env, "Regular pool 2"),
        &2_000_000_000u128,
        &100_000u64,
    );

    // Verify consistent sequencing
    assert_eq!(pool_id_1, 1);
    assert_eq!(pool_id_2, 2);
    assert_eq!(pool_id_3, 3);
    assert_eq!(client.get_pool_count(), 3);
}

/// Test create_pool_for_school: Duplicate calls create distinct pools, not overwrite.
#[test]
fn test_create_pool_for_school_duplicate_calls_create_distinct_pools() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Create first pool
    let pool_id_1 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "First Pool"),
        &String::from_str(&env, "Description 1"),
        &5_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Create second pool with same school but different metadata
    let pool_id_2 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Second Pool"),
        &String::from_str(&env, "Description 2"),
        &3_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Verify they are different pools
    assert_ne!(pool_id_1, pool_id_2);

    // Verify both pools exist independently
    let pool1 = client.get_pool(&pool_id_1);
    let pool2 = client.get_pool(&pool_id_2);

    assert_eq!(pool1.2, 5_000_000_000u128); // First pool goal
    assert_eq!(pool2.2, 3_000_000_000u128); // Second pool goal

    // Verify metadata is distinct
    let (title1, desc1) = client.get_pool_metadata(&pool_id_1);
    let (title2, desc2) = client.get_pool_metadata(&pool_id_2);

    assert_ne!(title1, title2);
    assert_ne!(desc1, desc2);

    // Verify both are linked to the same school
    assert_eq!(client.get_pool_school(&pool_id_1), school);
    assert_eq!(client.get_pool_school(&pool_id_2), school);
}

/// Test create_pool_for_school with authorization requirement.
#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_create_pool_for_school_requires_authorization() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    // Set up without mocking all auths
    let client_with_mock = ContractClient::new(&env, &contract_id);
    
    // This should fail because creator authorization is not mocked
    let admin_key = soroban_sdk::Symbol::new(&env, "admin");
    env.storage().persistent().set(&admin_key, &admin);
    
    let school_key = (soroban_sdk::Symbol::new(&env, "school_reg"), school.clone());
    env.storage().persistent().set(&school_key, &true);

    client_with_mock.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Unauthorized Pool"),
        &String::from_str(&env, "Should fail"),
        &5_000_000_000u128,
        &school,
        &100_000u64,
    );
}

/// Test create_pool_for_school attributes are properly stored.
#[test]
fn test_create_pool_for_school_attributes_stored() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    let title = String::from_str(&env, "School Scholarship Fund");
    let description = String::from_str(&env, "Help students achieve their dreams");
    let goal = 10_000_000_000u128;
    let deadline = 150_000u64;

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    let pool_id = client.create_pool_for_school(
        &creator,
        &title,
        &description,
        &goal,
        &school,
        &deadline,
    );

    // Verify all attributes
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, goal);
    assert_eq!(pool.3, 0u128); // Initial collected should be 0
    assert_eq!(pool.4, false); // Initially not closed
    assert_eq!(pool.5, deadline);

    // Verify metadata
    let (retrieved_title, retrieved_desc) = client.get_pool_metadata(&pool_id);
    assert_eq!(retrieved_title, title);
    assert_eq!(retrieved_desc, description);

    // Verify school linkage
    assert_eq!(client.get_pool_school(&pool_id), school);
}

/// Test create_pool_for_school maintains pool count correctly.
#[test]
fn test_create_pool_for_school_maintains_count() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school1 = Address::generate(&env);
    let school2 = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school1);
    client.register_school(&admin, &school2);

    // Initial count
    assert_eq!(client.get_pool_count(), 0);

    // Create first school pool
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "School 1 pool"),
        &5_000_000_000u128,
        &school1,
        &100_000u64,
    );
    assert_eq!(client.get_pool_count(), 1);

    // Create second school pool
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "School 2 pool"),
        &3_000_000_000u128,
        &school2,
        &100_000u64,
    );
    assert_eq!(client.get_pool_count(), 2);

    // Create regular pool
    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 3"),
        &String::from_str(&env, "Regular pool"),
        &2_000_000_000u128,
        &100_000u64,
    );
    assert_eq!(client.get_pool_count(), 3);

    // Create third school pool
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool 4"),
        &String::from_str(&env, "School 1 pool 2"),
        &7_000_000_000u128,
        &school1,
        &100_000u64,
    );
    assert_eq!(client.get_pool_count(), 4);
}

/// Test create_pool_for_school with various goal amounts.
#[test]
fn test_create_pool_for_school_various_goals() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    let goals = vec![
        100_000_000u128,
        1_000_000_000u128,
        10_000_000_000u128,
        100_000_000_000u128,
    ];

    for (i, goal) in goals.iter().enumerate() {
        let pool_id = client.create_pool_for_school(
            &creator,
            &String::from_str(&env, &format!("Pool {}", i)),
            &String::from_str(&env, &format!("Goal: {}", goal)),
            goal,
            &school,
            &100_000u64,
        );

        let pool = client.get_pool(&pool_id);
        assert_eq!(pool.2, *goal);
    }
}
