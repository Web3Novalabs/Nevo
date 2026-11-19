#![cfg(test)]
//! Tests for campaign creation with fees — issue #1274.
//!
//! Covers:
//!   1. Creator with sufficient balance pays fee successfully.
//!   2. Creator with insufficient balance fails.
//!   3. Zero fee allows creation without balance check.
//!   4. Fee tokens transferred to contract.
//!   5. Creation fee event emitted.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
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

fn token_balance(env: &Env, token: &Address, account: &Address) -> i128 {
    soroban_sdk::token::Client::new(env, token).balance(account)
}

/// (1) Creator with sufficient balance pays fee successfully.
#[test]
fn test_campaign_creation_fee_sufficient_balance_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let fee: i128 = 500_000_000;

    let fee_token = create_token(&env, fee, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    let pool_id = client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Community Library"),
        &String::from_str(&env, "Funding local library renovations"),
        &5_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    assert_eq!(pool_id, 1u32);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.1, creator);
}

/// (2) Creator with insufficient balance fails.
#[test]
#[should_panic]
fn test_campaign_creation_fee_insufficient_balance_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let fee: i128 = 500_000_000;
    let insufficient_balance: i128 = fee - 1;

    let fee_token = create_token(&env, insufficient_balance, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Underfunded Campaign"),
        &String::from_str(&env, "Should not succeed"),
        &1_000_000_000u128,
        &100_000u64,
        &fee_token,
    );
}

/// (3) Zero fee allows creation without balance check.
#[test]
fn test_campaign_creation_zero_fee_no_balance_check() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    let fee_token = create_token(&env, 0i128, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &0i128);

    let pool_id = client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Zero Fee Campaign"),
        &String::from_str(&env, "Creation should not require tokens"),
        &2_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    assert_eq!(pool_id, 1u32);
}

/// (4) Fee tokens transferred to contract.
#[test]
fn test_campaign_creation_fee_tokens_transferred_to_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let fee: i128 = 250_000_000;
    let initial_balance: i128 = 1_000_000_000;

    let fee_token = create_token(&env, initial_balance, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    assert_eq!(token_balance(&env, &fee_token, &creator), initial_balance);
    assert_eq!(token_balance(&env, &fee_token, &contract_id), 0);

    client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Fee Balance Campaign"),
        &String::from_str(&env, "Testing token movement"),
        &5_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    assert_eq!(
        token_balance(&env, &fee_token, &creator),
        initial_balance - fee
    );
    assert_eq!(token_balance(&env, &fee_token, &contract_id), fee);
}

/// (5) Creation fee event emitted.
#[test]
fn test_campaign_creation_fee_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let fee: i128 = 150_000_000;

    let fee_token = create_token(&env, fee, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Event Campaign"),
        &String::from_str(&env, "Testing event emissions"),
        &3_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    let events = env.events().all();
    let contract_events_count = events
        .iter()
        .filter(|(contract, _topics, _data)| *contract == contract_id)
        .count();

    assert!(contract_events_count >= 2);
}
