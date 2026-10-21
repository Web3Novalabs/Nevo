//! Tests for pool contribution amount validation — issue #1266
//!
//! Covers:
//!   1. Zero amount contribution fails with InvalidAmount.
//!   2. Negative amount contribution fails.
//!   3. Maximum i128 amount contribution succeeds if balance allows.
//!   4. Contribution exceeding user balance fails with token transfer error.

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

/// (1) Zero amount contribution fails with InvalidAmount.
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_zero_amount_contribution_fails_with_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Zero Amount Pool"),
        &String::from_str(&env, "Testing zero contribution validation"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Contributing 0 must fail with InvalidAmount
    client.donate_with_token(&pool_id, &donor, &token, &0i128);
}

/// (2) Negative amount contribution fails.
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_negative_amount_contribution_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Negative Amount Pool"),
        &String::from_str(&env, "Testing negative contribution validation"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Negative contribution must fail with InvalidAmount
    client.donate_with_token(&pool_id, &donor, &token, &-50_000_000i128);
}

/// (2b) Edge case: minimal negative amount (-1) fails.
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_minus_one_amount_contribution_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Minus One Pool"),
        &String::from_str(&env, "Testing -1 contribution validation"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.donate_with_token(&pool_id, &donor, &token, &-1i128);
}

/// (3) Maximum i128 amount contribution succeeds if balance allows.
#[test]
fn test_maximum_i128_amount_contribution_succeeds_if_balance_allows() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let max_amount = i128::MAX;
    let token = create_token(&env, max_amount, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Max Amount Pool"),
        &String::from_str(&env, "Testing maximum i128 contribution"),
        &(max_amount as u128),
        &100_000u64,
    );

    // Maximum i128 amount should succeed when donor has sufficient balance
    client.donate_with_token(&pool_id, &donor, &token, &max_amount);

    let pool = client.get_pool(&pool_id);
    assert_eq!(
        pool.3, max_amount as u128,
        "Pool collected amount must equal maximum i128"
    );
    assert_eq!(
        client.get_total_raised(&pool_id),
        max_amount as u128,
        "Total raised must match maximum i128 contribution"
    );
}

/// (4) Contribution exceeding user balance fails with token transfer error.
#[test]
#[should_panic]
fn test_contribution_exceeding_user_balance_fails_with_token_transfer_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let balance = 100_000_000i128;
    let token = create_token(&env, balance, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Overdraw Pool"),
        &String::from_str(&env, "Testing donation exceeding balance"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Attempting to donate more than the donor's token balance (200M vs 100M)
    // must fail during token transfer
    let excessive_amount = balance * 2;
    client.donate_with_token(&pool_id, &donor, &token, &excessive_amount);
}
