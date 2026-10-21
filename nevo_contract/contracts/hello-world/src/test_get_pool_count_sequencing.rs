#![cfg(test)]

//! Issue #1376: Tests for get_pool_count sequencing consistency.

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, BytesN, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// (1) Returns 0 before any pool is created.
#[test]
fn test_get_pool_count_initial_is_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_pool_count(), 0u32, "Initial pool count must be 0");
}

/// (2) Increments by exactly 1 after each create_pool/create_pool_for_school call.
#[test]
fn test_get_pool_count_increments_by_one() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[0u8; 32]);
    client.set_admin(&admin);
    client.register_school(&school, &hash);

    let creator = Address::generate(&env);

    // Initial count
    assert_eq!(client.get_pool_count(), 0u32);

    // First call: create_pool
    let pool_id_1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "Desc 1"),
        &1_000_000u128,
        &100_000u64,
    );
    assert_eq!(pool_id_1, 1u32);
    assert_eq!(client.get_pool_count(), 1u32);

    // Second call: create_pool_for_school
    let pool_id_2 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Desc 2"),
        &2_000_000u128,
        &school,
        &200_000u64,
    );
    assert_eq!(pool_id_2, 2u32);
    assert_eq!(client.get_pool_count(), 2u32);

    // Third call: create_pool
    let pool_id_3 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 3"),
        &String::from_str(&env, "Desc 3"),
        &3_000_000u128,
        &300_000u64,
    );
    assert_eq!(pool_id_3, 3u32);
    assert_eq!(client.get_pool_count(), 3u32);
}

/// (3) Value matches the highest valid pool_id usable with get_pool.
#[test]
fn test_get_pool_count_matches_highest_valid_pool_id() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for i in 1..=5 {
        let pool_id = client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "Desc"),
            &1_000_000u128,
            &100_000u64,
        );
        assert_eq!(pool_id, i);
    }

    let count = client.get_pool_count();
    assert_eq!(count, 5u32);

    // Count (5) is usable with get_pool
    let pool = client.get_pool(&count);
    assert_eq!(pool.0, count);

    // Count + 1 (6) panics with PoolNotFound
    let res = client.try_get_pool(&(count + 1));
    assert!(res.is_err(), "pool_id above get_pool_count must fail");
}

/// (4) Unaffected by closing or donating to existing pools.
#[test]
fn test_get_pool_count_unaffected_by_closing_or_donating() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &donor);

    let pool_id_1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "Desc 1"),
        &1_000_000u128,
        &100_000u64,
    );
    let _pool_id_2 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Desc 2"),
        &2_000_000u128,
        &200_000u64,
    );

    assert_eq!(client.get_pool_count(), 2u32);

    // Donate to pool 1
    client.donate_with_token(&pool_id_1, &donor, &token, &100i128);
    assert_eq!(client.get_pool_count(), 2u32, "Pool count must remain unchanged after donation");

    // Close pool 1 (must set state to Disbursed or Cancelled first)
    client.set_pool_state(&pool_id_1, &PoolState::Disbursed);
    client.close_pool(&pool_id_1);
    assert_eq!(client.get_pool_count(), 2u32, "Pool count must remain unchanged after closing pool");

    // Create another pool to ensure next pool_id is 3
    let pool_id_3 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 3"),
        &String::from_str(&env, "Desc 3"),
        &3_000_000u128,
        &300_000u64,
    );
    assert_eq!(pool_id_3, 3u32);
    assert_eq!(client.get_pool_count(), 3u32);
}

/// (5) Consistent count under interleaved calls from multiple schools/admins.
#[test]
fn test_get_pool_count_interleaved_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school_a = Address::generate(&env);
    let school_b = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[0u8; 32]);

    client.set_admin(&admin);
    client.register_school(&school_a, &hash);
    client.register_school(&school_b, &hash);

    let creator_1 = Address::generate(&env);
    let creator_2 = Address::generate(&env);

    // Interleaved sequence of pool creations
    let p1 = client.create_pool(&creator_1, &String::from_str(&env, "P1"), &String::from_str(&env, "D1"), &100u128, &100u64);
    assert_eq!(p1, 1);
    assert_eq!(client.get_pool_count(), 1);

    let p2 = client.create_pool_for_school(&creator_2, &String::from_str(&env, "P2"), &String::from_str(&env, "D2"), &200u128, &school_a, &200u64);
    assert_eq!(p2, 2);
    assert_eq!(client.get_pool_count(), 2);

    let p3 = client.create_pool_for_school(&creator_1, &String::from_str(&env, "P3"), &String::from_str(&env, "D3"), &300u128, &school_b, &300u64);
    assert_eq!(p3, 3);
    assert_eq!(client.get_pool_count(), 3);

    let p4 = client.create_pool(&creator_2, &String::from_str(&env, "P4"), &String::from_str(&env, "D4"), &400u128, &400u64);
    assert_eq!(p4, 4);
    assert_eq!(client.get_pool_count(), 4);
}
