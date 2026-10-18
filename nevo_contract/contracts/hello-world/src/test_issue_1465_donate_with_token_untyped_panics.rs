#![cfg(test)]
//! Regression tests for `donate_with_token` untyped `InvalidAmount` and
//! `TokenTransferFailed` panics - issue #1465.
//!
//! Covers:
//!   (1) `amount == 0` and `amount < 0` panic with the exact raw string `"InvalidAmount"`.
//!   (2) `token_address` mismatch against a pool with `set_pool_token` configured
//!       panics with the exact raw string `"TokenTransferFailed"`.
//!   (3) Follow-up noted: migrate these two raw string panics to typed
//!       `ContractError` variants alongside `PoolNotFound`/`PoolIsClosed`/`InvalidPoolState`.

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin);
    let sac = StellarAssetClient::new(env, &token.address());
    if amount > 0 {
        sac.mint(recipient, &amount);
    }
    token.address()
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_donate_with_token_zero_amount_panics_with_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000_000, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Zero Amount Pool"),
        &String::from_str(&env, "Testing zero donation amount"),
        &10_000_000u128,
        &100_000u64,
    );

    client.donate_with_token(&pool_id, &donor, &token, &0i128);
}

#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_donate_with_token_negative_amount_panics_with_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000_000, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Negative Amount Pool"),
        &String::from_str(&env, "Testing negative donation amount"),
        &10_000_000u128,
        &100_000u64,
    );

    client.donate_with_token(&pool_id, &donor, &token, &-500i128);
}

#[test]
#[should_panic(expected = "TokenTransferFailed")]
fn test_donate_with_token_mismatched_pool_token_panics_with_token_transfer_failed() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let configured_token = create_token(&env, 1_000_000, &donor);
    let wrong_token = create_token(&env, 1_000_000, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Token Mismatch Pool"),
        &String::from_str(&env, "Testing set_pool_token mismatch"),
        &10_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &configured_token);
    client.donate_with_token(&pool_id, &donor, &wrong_token, &500_000i128);
}