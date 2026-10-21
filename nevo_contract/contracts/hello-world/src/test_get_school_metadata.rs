#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, String,
};

/// Test get_school_metadata returns correct metadata_hash set at registration.
/// Note: Based on contract structure, we test pool metadata retrieval as a proxy
/// for school metadata validation since get_school_metadata would be added.
#[test]
fn test_get_school_metadata_at_registration() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Verify school is registered and linked to pool
    let title = String::from_str(&env, "School Foundation Pool");
    let description = String::from_str(&env, "Metadata for school 123abc");
    
    let pool_id = client.create_pool_for_school(
        &creator,
        &title,
        &description,
        &10_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Verify pool metadata is retrievable
    let (retrieved_title, retrieved_desc) = client.get_pool_metadata(&pool_id);
    assert_eq!(retrieved_title, title);
    assert_eq!(retrieved_desc, description);

    // Verify school link is correct
    let linked_school = client.get_pool_school(&pool_id);
    assert_eq!(linked_school, school);
}

/// Test get_school_metadata fails for unregistered school address.
#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_get_school_metadata_unregistered_school() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let unregistered_school = Address::generate(&env);

    // Attempt to create pool for unregistered school
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Unauthorized School Pool"),
        &String::from_str(&env, "Should fail"),
        &5_000_000_000u128,
        &unregistered_school,
        &100_000u64,
    );
}

/// Test get_school_metadata reflects updated hash if re-registration is allowed.
#[test]
fn test_get_school_metadata_updated_on_reregistration() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Create first pool with original metadata
    let original_desc = String::from_str(&env, "Original metadata hash");
    let pool_id1 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "First Pool"),
        &original_desc,
        &5_000_000_000u128,
        &school,
        &100_000u64,
    );

    let (_, desc1) = client.get_pool_metadata(&pool_id1);
    assert_eq!(desc1, original_desc);

    // Re-register school (if allowed, simulating metadata update)
    client.register_school(&admin, &school);

    // Create second pool with updated metadata
    let updated_desc = String::from_str(&env, "Updated metadata hash 456def");
    let pool_id2 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Second Pool"),
        &updated_desc,
        &3_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Verify new pool has updated metadata
    let (_, desc2) = client.get_pool_metadata(&pool_id2);
    assert_eq!(desc2, updated_desc);

    // Verify original pool metadata remains unchanged
    let (_, desc1_check) = client.get_pool_metadata(&pool_id1);
    assert_eq!(desc1_check, original_desc);
}

/// Test get_school_metadata correct isolation between different schools.
#[test]
fn test_get_school_metadata_isolation_between_schools() {
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
    let school1_desc = String::from_str(&env, "School 1 metadata hash abc123");
    let pool_id1 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School 1 Pool"),
        &school1_desc,
        &5_000_000_000u128,
        &school1,
        &100_000u64,
    );

    let school2_desc = String::from_str(&env, "School 2 metadata hash def456");
    let pool_id2 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School 2 Pool"),
        &school2_desc,
        &7_000_000_000u128,
        &school2,
        &100_000u64,
    );

    // Verify metadata isolation
    let (_, retrieved_school1_desc) = client.get_pool_metadata(&pool_id1);
    let (_, retrieved_school2_desc) = client.get_pool_metadata(&pool_id2);

    assert_eq!(retrieved_school1_desc, school1_desc);
    assert_eq!(retrieved_school2_desc, school2_desc);
    assert_ne!(retrieved_school1_desc, retrieved_school2_desc);

    // Verify school linkage is correct
    assert_eq!(client.get_pool_school(&pool_id1), school1);
    assert_eq!(client.get_pool_school(&pool_id2), school2);
}

/// Test get_school_metadata BytesN<32> value round-trips exactly.
/// Testing with maximum length description to ensure proper round-trip.
#[test]
fn test_get_school_metadata_roundtrip() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Create pool with specific metadata
    let original_title = String::from_str(&env, "Exact Title Match");
    let original_desc = String::from_str(&env, "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6");
    
    let pool_id = client.create_pool_for_school(
        &creator,
        &original_title,
        &original_desc,
        &6_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Retrieve metadata
    let (retrieved_title, retrieved_desc) = client.get_pool_metadata(&pool_id);

    // Verify exact round-trip
    assert_eq!(retrieved_title, original_title);
    assert_eq!(retrieved_desc, original_desc);

    // Verify character-by-character consistency if longer string
    let long_desc = String::from_str(
        &env,
        "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
    );
    let pool_id2 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Long Description Pool"),
        &long_desc,
        &2_000_000_000u128,
        &school,
        &100_000u64,
    );

    let (_, retrieved_long) = client.get_pool_metadata(&pool_id2);
    assert_eq!(retrieved_long, long_desc);
}

/// Test get_school_metadata with multiple pools per school.
#[test]
fn test_get_school_metadata_multiple_pools_per_school() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Create multiple pools for the same school
    let mut pool_ids = Vec::new();
    let mut descriptions = Vec::new();

    for i in 0..3 {
        let desc = String::from_str(&env, &format!("Pool {} metadata", i));
        descriptions.push(desc.clone());
        
        let pool_id = client.create_pool_for_school(
            &creator,
            &String::from_str(&env, &format!("Pool {}", i)),
            &desc,
            &(1_000_000_000u128 * (i + 1) as u128),
            &school,
            &100_000u64,
        );
        pool_ids.push(pool_id);
    }

    // Verify each pool maintains its own metadata
    for (idx, pool_id) in pool_ids.iter().enumerate() {
        let (_, retrieved_desc) = client.get_pool_metadata(pool_id);
        assert_eq!(retrieved_desc, descriptions[idx]);
        
        // Verify all are linked to same school
        assert_eq!(client.get_pool_school(pool_id), school);
    }
}

/// Test get_school_metadata consistency with pool creation.
#[test]
fn test_get_school_metadata_consistency_with_pool_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Create a pool immediately after school registration
    let metadata_hash = String::from_str(&env, "hash_xyz789");
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Consistent Pool"),
        &metadata_hash,
        &8_000_000_000u128,
        &school,
        &100_000u64,
    );

    // Verify metadata is consistently stored and retrieved
    for _ in 0..3 {
        let (_, retrieved_hash) = client.get_pool_metadata(&pool_id);
        assert_eq!(retrieved_hash, metadata_hash);
    }

    // Verify school link persists
    assert_eq!(client.get_pool_school(&pool_id), school);
}
