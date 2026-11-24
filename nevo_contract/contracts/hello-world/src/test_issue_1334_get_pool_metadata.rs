#![cfg(test)]
//! Issue #1334: `get_pool_metadata` retrieval accuracy.
//!
//! Scenarios:
//!   1. Returns the correct (name, description) tuple for an existing pool.
//!   2. Fails (panics `PoolNotFound`) for a nonexistent pool_id.
//!   3. Reflects the metadata as set at pool-creation time.
//!   4. Unicode and long string metadata round-trip correctly.
//!   5. Metadata is unaffected by donations or pool state changes.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// (1) + (3): the returned tuple matches the title/description set at creation.
#[test]
fn test_get_pool_metadata_returns_creation_values() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let title = String::from_str(&env, "School Supplies Drive");
    let description = String::from_str(&env, "Notebooks and uniforms for the term");
    let pool_id = client.create_pool(
        &Address::generate(&env),
        &title,
        &description,
        &4_000_000_000u128,
        &250_000u64,
    );

    let (stored_title, stored_description) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, title);
    assert_eq!(stored_description, description);
    assert_eq!(stored_title, String::from_str(&env, "School Supplies Drive"));
    assert_eq!(
        stored_description,
        String::from_str(&env, "Notebooks and uniforms for the term")
    );
}

/// (2): a nonexistent pool_id panics with `PoolNotFound` (#1).
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_get_pool_metadata_nonexistent_panics() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let _ = client.get_pool_metadata(&424_242u32);
}

/// (4): unicode metadata round-trips byte-for-byte.
#[test]
fn test_get_pool_metadata_unicode_round_trips() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let title = String::from_str(&env, "🌍 School Supplies Édition");
    let description = String::from_str(&env, "Café résumé — 日本語のテスト 📚✏️");
    let pool_id = client.create_pool(
        &Address::generate(&env),
        &title,
        &description,
        &1_000_000_000u128,
        &250_000u64,
    );

    let (stored_title, stored_description) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, title);
    assert_eq!(stored_description, description);
}

/// (4): a maximum-length (500-char) description round-trips exactly.
#[test]
fn test_get_pool_metadata_long_description_round_trips() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let long: std::string::String = "a".repeat(500);
    let title = String::from_str(&env, "Long Metadata Pool");
    let description = String::from_str(&env, long.as_str());
    let pool_id = client.create_pool(
        &Address::generate(&env),
        &title,
        &description,
        &1_000_000_000u128,
        &250_000u64,
    );

    let (stored_title, stored_description) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, title);
    assert_eq!(stored_description, description);
}

/// (5): donations (including one that flips the pool to `Completed`) do not
/// alter the stored metadata.
#[test]
fn test_get_pool_metadata_unaffected_by_donations_and_state_changes() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let title = String::from_str(&env, "Immutable Metadata Pool");
    let description = String::from_str(&env, "Metadata must not change after donations");
    let goal = 1_000_000_000u128;
    let pool_id = client.create_pool(
        &Address::generate(&env),
        &title,
        &description,
        &goal,
        &250_000u64,
    );

    let (t0, d0) = client.get_pool_metadata(&pool_id);
    assert_eq!(t0, title);
    assert_eq!(d0, description);

    // Donation that does not complete the pool (state stays Active).
    let donor_a = Address::generate(&env);
    client.donate(&pool_id, &donor_a, &400_000_000u128);
    let (t1, d1) = client.get_pool_metadata(&pool_id);
    assert_eq!(t1, title);
    assert_eq!(d1, description);

    // Donation that completes the pool (collected reaches goal -> Completed).
    let donor_b = Address::generate(&env);
    client.donate(&pool_id, &donor_b, &600_000_000u128);
    let (t2, d2) = client.get_pool_metadata(&pool_id);
    assert_eq!(t2, title);
    assert_eq!(d2, description);
}
