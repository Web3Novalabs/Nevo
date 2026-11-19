//! Tests for pool contribution state constraints — issue #1265
//!
//! Covers:
//!   1. Contribute to Active pool succeeds.
//!   2. Contribute to Paused pool fails with InvalidPoolState.
//!   3. Contribute to Completed pool fails.
//!   4. Contribute to Cancelled pool fails.
//!   5. Contribute to Disbursed pool fails.
//!   6. Contribute to Closed pool fails.

#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// (1) Contribute to Active pool succeeds.
#[test]
fn test_contribute_to_active_pool_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Active Pool"),
        &String::from_str(&env, "Testing contribution to active pool"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Newly created pool is Active by default
    client.donate_with_token(&pool_id, &donor, &token, &50_000_000i128);

    let pool = client.get_pool(&pool_id);
    assert_eq!(
        pool.3, 50_000_000u128,
        "Active pool should accept contributions and update collected amount"
    );
    assert_eq!(client.get_total_raised(&pool_id), 50_000_000u128);
}

/// (2) Contribute to Paused pool fails with InvalidPoolState (ContractError #2).
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_contribute_to_paused_pool_fails_with_invalid_pool_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Paused Pool"),
        &String::from_str(&env, "Testing paused pool contribution constraint"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Paused);

    // Must fail with ContractError::InvalidPoolState (#2)
    client.donate_with_token(&pool_id, &donor, &token, &50_000_000i128);
}

/// (3) Contribute to Completed pool fails with InvalidPoolState (ContractError #2).
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_contribute_to_completed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Completed Pool"),
        &String::from_str(&env, "Testing completed pool contribution constraint"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Completed);

    // Must fail with ContractError::InvalidPoolState (#2)
    client.donate_with_token(&pool_id, &donor, &token, &50_000_000i128);
}

/// (4) Contribute to Cancelled pool fails with InvalidPoolState (ContractError #2).
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_contribute_to_cancelled_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cancelled Pool"),
        &String::from_str(&env, "Testing cancelled pool contribution constraint"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    // Must fail with ContractError::InvalidPoolState (#2)
    client.donate_with_token(&pool_id, &donor, &token, &50_000_000i128);
}

/// (5) Contribute to Disbursed pool fails with InvalidPoolState (ContractError #2).
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_contribute_to_disbursed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Disbursed Pool"),
        &String::from_str(&env, "Testing disbursed pool contribution constraint"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Must fail with ContractError::InvalidPoolState (#2)
    client.donate_with_token(&pool_id, &donor, &token, &50_000_000i128);
}

/// (6) Contribute to Closed pool fails with PoolIsClosed (ContractError #4).
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_contribute_to_closed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closed Pool"),
        &String::from_str(&env, "Testing closed pool contribution constraint"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Transition to Disbursed so close_pool is valid, then close the pool
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    assert!(client.is_closed(&pool_id), "Pool must be marked closed");

    // Must fail with ContractError::PoolIsClosed (#4)
    client.donate_with_token(&pool_id, &donor, &token, &50_000_000i128);
}

/// Verify native `donate` function also rejects non-active states.
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_native_donate_to_paused_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Native Donate Paused Pool"),
        &String::from_str(&env, "Testing native donate state validation"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Paused);
    client.donate(&pool_id, &donor, &50_000_000u128);
}
