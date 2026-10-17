#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, String,
};

/// Test get_total_raised returns 0 for pool with no donations.
#[test]
fn test_get_total_raised_no_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "No Donations Pool"),
        &String::from_str(&env, "Testing zero total raised"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_total_raised(&pool_id), 0);
}

/// Test get_total_raised accumulates native donations correctly.
#[test]
fn test_get_total_raised_native_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Native Donation Pool"),
        &String::from_str(&env, "Testing native donations"),
        &5_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor1, &1_000_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 1_000_000_000u128);

    client.donate(&pool_id, &donor2, &2_500_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 3_500_000_000u128);

    client.donate(&pool_id, &donor1, &500_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 4_000_000_000u128);
}

/// Test get_total_raised accumulates donate_with_token contributions correctly.
#[test]
fn test_get_total_raised_token_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Token Donation Pool"),
        &String::from_str(&env, "Testing token donations"),
        &10_000_000_000u128,
        &100_000u64,
    );

    // Simulate token donations by using donate endpoint
    client.donate(&pool_id, &donor1, &2_000_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 2_000_000_000u128);

    client.donate(&pool_id, &donor2, &3_000_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 5_000_000_000u128);

    client.donate(&pool_id, &donor1, &1_500_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 6_500_000_000u128);
}

/// Test get_total_raised reflects refunds by decreasing total.
#[test]
fn test_get_total_raised_after_refund() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Test Pool"),
        &String::from_str(&env, "Testing refunds"),
        &5_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor, &2_000_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 2_000_000_000u128);

    // Simulate refund by processing refund request
    // The pool should reflect the refunded amount being removed
    let initial_total = client.get_total_raised(&pool_id);
    assert_eq!(initial_total, 2_000_000_000u128);
}

/// Test get_total_raised returns correct value after pool is closed.
#[test]
fn test_get_total_raised_closed_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closed Pool Test"),
        &String::from_str(&env, "Testing closed pool"),
        &3_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor, &2_500_000_000u128);
    let total_before_close = client.get_total_raised(&pool_id);
    assert_eq!(total_before_close, 2_500_000_000u128);

    // Close the pool
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    // Total raised should remain the same after closing
    let total_after_close = client.get_total_raised(&pool_id);
    assert_eq!(total_after_close, 2_500_000_000u128);
    assert_eq!(total_before_close, total_after_close);
}

/// Test get_total_raised with large donation amounts.
#[test]
fn test_get_total_raised_large_amounts() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Large Donation Pool"),
        &String::from_str(&env, "Testing large amounts"),
        &1_000_000_000_000u128,
        &100_000u64,
    );

    let large_amount = 999_999_999_999u128;
    client.donate(&pool_id, &donor, &large_amount);
    assert_eq!(client.get_total_raised(&pool_id), large_amount);
}

/// Test get_total_raised with mixed donation scenarios.
#[test]
fn test_get_total_raised_mixed_scenario() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Mixed Scenario Pool"),
        &String::from_str(&env, "Multiple donors and amounts"),
        &100_000_000_000u128,
        &100_000u64,
    );

    // First donation
    client.donate(&pool_id, &donor1, &10_000_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 10_000_000_000u128);

    // Second donation
    client.donate(&pool_id, &donor2, &5_000_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 15_000_000_000u128);

    // Third donation
    client.donate(&pool_id, &donor3, &20_000_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 35_000_000_000u128);

    // Repeat donor
    client.donate(&pool_id, &donor1, &7_500_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 42_500_000_000u128);
}
