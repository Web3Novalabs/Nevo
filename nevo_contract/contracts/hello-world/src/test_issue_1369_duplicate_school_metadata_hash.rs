#![cfg(test)]
//! Issue #1369 — Tests for duplicate school metadata_hash registration.
//!
//! Covers:
//! 1. Two different school addresses can register with the same metadata_hash
//!    without collision.
//! 2. Re-registering the same school address with a new hash updates the stored
//!    value.
//! 3. get_school_metadata remains correctly isolated per address even when
//!    identical hashes are stored.
//! 4. is_school_registered is unaffected by hash collisions.
//! 5. No storage key collision occurs between schools sharing a hash.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup_admin(env: &Env) -> (ContractClient<'_>, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.set_admin(&admin);
    (client, admin)
}

/// (1) Two different school addresses can register with the same metadata_hash
/// without collision — both are registered independently.
#[test]
fn test_two_schools_same_hash_no_collision() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_admin(&env);

    let school_a = Address::generate(&env);
    let school_b = Address::generate(&env);
    let shared_hash = BytesN::from_array(&env, &[0xABu8; 32]);

    client.register_school(&school_a, &shared_hash);
    client.register_school(&school_b, &shared_hash);

    // Both schools must be independently registered.
    assert!(client.is_school_registered(&school_a));
    assert!(client.is_school_registered(&school_b));
}

/// (2) Re-registering the same school address with a new hash updates the
/// stored value (not rejected).
#[test]
fn test_reregister_same_address_updates_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_admin(&env);

    let school = Address::generate(&env);
    let first_hash = BytesN::from_array(&env, &[0x01u8; 32]);
    let second_hash = BytesN::from_array(&env, &[0x02u8; 32]);

    client.register_school(&school, &first_hash);
    assert_eq!(client.get_school_metadata(&school), first_hash);

    // Re-registering with a new hash must overwrite the previous one.
    client.register_school(&school, &second_hash);
    assert_eq!(
        client.get_school_metadata(&school),
        second_hash,
        "re-registering must replace the stored hash, not keep the old one"
    );
    assert!(client.is_school_registered(&school), "school must still be registered after update");
}

/// (3) get_school_metadata is isolated per address even when two schools share
/// the same hash — each address returns its own stored value.
#[test]
fn test_get_school_metadata_isolated_per_address_with_shared_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_admin(&env);

    let school_a = Address::generate(&env);
    let school_b = Address::generate(&env);
    let shared_hash = BytesN::from_array(&env, &[0xFFu8; 32]);

    client.register_school(&school_a, &shared_hash);
    client.register_school(&school_b, &shared_hash);

    // Each address must return exactly its own stored hash.
    assert_eq!(client.get_school_metadata(&school_a), shared_hash);
    assert_eq!(client.get_school_metadata(&school_b), shared_hash);

    // Now update school_b's hash; school_a's must remain unchanged.
    let different_hash = BytesN::from_array(&env, &[0x11u8; 32]);
    client.register_school(&school_b, &different_hash);

    assert_eq!(
        client.get_school_metadata(&school_a),
        shared_hash,
        "school_a metadata must not be affected by school_b re-registration"
    );
    assert_eq!(
        client.get_school_metadata(&school_b),
        different_hash,
        "school_b must reflect its updated hash"
    );
}

/// (4) is_school_registered is unaffected by hash collisions — the flag is
/// keyed by address, not by hash value.
#[test]
fn test_is_school_registered_unaffected_by_hash_collisions() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_admin(&env);

    let school_a = Address::generate(&env);
    let school_b = Address::generate(&env);
    let school_c = Address::generate(&env); // never registered
    let shared_hash = BytesN::from_array(&env, &[0x55u8; 32]);

    client.register_school(&school_a, &shared_hash);
    client.register_school(&school_b, &shared_hash);

    assert!(client.is_school_registered(&school_a), "school_a must be registered");
    assert!(client.is_school_registered(&school_b), "school_b must be registered");
    assert!(!client.is_school_registered(&school_c), "school_c must NOT be registered");
}

/// (5) No storage key collision between schools sharing a hash — operations on
/// one school must not pollute or clobber the other school's data.
#[test]
fn test_no_storage_key_collision_between_schools_sharing_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_admin(&env);

    let school_a = Address::generate(&env);
    let school_b = Address::generate(&env);
    let shared_hash = BytesN::from_array(&env, &[0xCAu8; 32]);
    let unique_hash_a = BytesN::from_array(&env, &[0x0Au8; 32]);
    let unique_hash_b = BytesN::from_array(&env, &[0x0Bu8; 32]);

    // Both share the same initial hash, then each gets its own unique hash.
    client.register_school(&school_a, &shared_hash);
    client.register_school(&school_b, &shared_hash);
    client.register_school(&school_a, &unique_hash_a);
    client.register_school(&school_b, &unique_hash_b);

    // Each school must carry its own unique hash without cross-contamination.
    assert_eq!(
        client.get_school_metadata(&school_a),
        unique_hash_a,
        "school_a must have its own unique hash"
    );
    assert_eq!(
        client.get_school_metadata(&school_b),
        unique_hash_b,
        "school_b must have its own unique hash without interference from school_a"
    );

    // Both schools must still appear as registered.
    assert!(client.is_school_registered(&school_a));
    assert!(client.is_school_registered(&school_b));
}

/// Bonus: an unregistered school's metadata query must still panic correctly
/// when other schools have been registered (no key bleed-over).
#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_unregistered_school_panics_despite_registered_peers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_admin(&env);

    let registered = Address::generate(&env);
    let unregistered = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[0xDDu8; 32]);

    client.register_school(&registered, &hash);

    // Querying an unregistered school must panic with SchoolNotRegistered (#14).
    client.get_school_metadata(&unregistered);
}

/// Bonus: registering many schools with the identical hash does not break any
/// of them — all remain independently queryable.
#[test]
fn test_many_schools_same_hash_all_registered() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_admin(&env);

    let shared_hash = BytesN::from_array(&env, &[0x99u8; 32]);

    let school_1 = Address::generate(&env);
    let school_2 = Address::generate(&env);
    let school_3 = Address::generate(&env);
    let school_4 = Address::generate(&env);
    let school_5 = Address::generate(&env);

    client.register_school(&school_1, &shared_hash);
    client.register_school(&school_2, &shared_hash);
    client.register_school(&school_3, &shared_hash);
    client.register_school(&school_4, &shared_hash);
    client.register_school(&school_5, &shared_hash);

    assert!(client.is_school_registered(&school_1));
    assert!(client.is_school_registered(&school_2));
    assert!(client.is_school_registered(&school_3));
    assert!(client.is_school_registered(&school_4));
    assert!(client.is_school_registered(&school_5));

    assert_eq!(client.get_school_metadata(&school_1), shared_hash);
    assert_eq!(client.get_school_metadata(&school_2), shared_hash);
    assert_eq!(client.get_school_metadata(&school_3), shared_hash);
    assert_eq!(client.get_school_metadata(&school_4), shared_hash);
    assert_eq!(client.get_school_metadata(&school_5), shared_hash);
}

/// Bonus: a school linked to a pool via create_pool_for_school is still
/// independently registered even if another school shares its hash.
#[test]
fn test_pool_creation_unaffected_by_duplicate_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_admin(&env);

    let school_a = Address::generate(&env);
    let school_b = Address::generate(&env);
    let shared_hash = BytesN::from_array(&env, &[0xBBu8; 32]);

    client.register_school(&school_a, &shared_hash);
    client.register_school(&school_b, &shared_hash);

    let creator = Address::generate(&env);
    // Pool is created for school_a — school_b sharing the hash must not affect this.
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool for school_a"),
        &String::from_str(&env, "Description"),
        &1_000_000i128,
        &school_a,
        &200_000u64,
    );

    assert_eq!(
        client.get_pool_school(&pool_id),
        school_a,
        "pool must be linked to school_a, not school_b"
    );
    assert!(client.is_school_registered(&school_b), "school_b must remain registered");
}
