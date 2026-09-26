#![cfg(test)]
//! Tests for campaign creation with fees — issue #1066.
//!
//! `create_pool_with_fee` reads the fee configured by `set_creation_fee`,
//! transfers it from the creator to the contract, then creates the pool.
//!
//! Scenarios:
//!   1. Creator with sufficient balance successfully pays the fee.
//!   2. Creation fails when the creator has an insufficient balance.
//!   3. A zero-fee configuration allows creation without a token transfer.
//!   4. The correct fee amount is transferred to the contract.
//!   5. The `POOL_CREATED` event is emitted on successful creation.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::StellarAssetClient,
    Address, Env, String,
};

// ── helper ───────────────────────────────────────────────────────────────

/// Register a Stellar asset contract, mint `amount` to `recipient`, and
/// return the token address.
fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// Return the current token balance of `account` for `token`.
fn token_balance(env: &Env, token: &Address, account: &Address) -> i128 {
    soroban_sdk::token::Client::new(env, token).balance(account)
}

// ── Test 1: sufficient balance — creation succeeds and fee is paid ────────

/// When the creator holds exactly the configured fee amount, `create_pool_with_fee`
/// must succeed and the pool must be retrievable.
#[test]
fn test_creation_fee_sufficient_balance_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let fee: i128 = 500_000_000; // 50 XLM in stroops

    let fee_token = create_token(&env, fee, &creator);

    // Configure the creation fee
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    // Create a pool — creator has exactly the fee amount
    let pool_id = client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Scholarship Fund"),
        &String::from_str(&env, "Supporting students in need"),
        &5_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    // Pool must be created successfully
    assert_eq!(pool_id, 1u32);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.1, creator, "Pool sponsor must be the creator");
}

// ── Test 2: insufficient balance — creation fails ─────────────────────────

/// When the creator's token balance is less than the configured fee,
/// the token transfer must fail (panic) and no pool is created.
#[test]
#[should_panic]
fn test_creation_fee_insufficient_balance_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let fee: i128 = 500_000_000; // 50 XLM
    let insufficient_balance: i128 = fee - 1; // one stroop short

    let fee_token = create_token(&env, insufficient_balance, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    // Should panic — creator cannot cover the fee
    client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Failing Fund"),
        &String::from_str(&env, "This should not be created"),
        &1_000_000_000u128,
        &100_000u64,
        &fee_token,
    );
}

// ── Test 3: zero fee — creation succeeds without any token transfer ───────

/// When the configured creation fee is zero, `create_pool_with_fee` must
/// succeed even if the creator has no tokens at all — no balance check is
/// performed when the fee is zero.
#[test]
fn test_creation_fee_zero_allows_creation_without_balance_check() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    // A token with zero balance for the creator
    let fee_token = create_token(&env, 0i128, &creator);

    client.set_admin(&admin);
    // Explicitly set fee to zero
    client.set_creation_fee(&admin, &0i128);

    let pool_id = client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Free Pool"),
        &String::from_str(&env, "No fee required"),
        &1_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    assert_eq!(pool_id, 1u32, "Pool must be created even with zero fee");
}

// ── Test 4: correct fee tokens are transferred to the contract ────────────

/// After a successful `create_pool_with_fee`, the contract address must hold
/// exactly `fee` tokens and the creator's balance must be reduced by `fee`.
#[test]
fn test_creation_fee_tokens_transferred_to_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let fee: i128 = 200_000_000; // 20 XLM
    let initial_balance: i128 = 1_000_000_000; // 100 XLM

    let fee_token = create_token(&env, initial_balance, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    // Verify creator's balance before
    assert_eq!(token_balance(&env, &fee_token, &creator), initial_balance);
    assert_eq!(token_balance(&env, &fee_token, &contract_id), 0);

    client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Fee Transfer Pool"),
        &String::from_str(&env, "Checking token transfer"),
        &5_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    // Creator must have paid the fee
    assert_eq!(
        token_balance(&env, &fee_token, &creator),
        initial_balance - fee,
        "Creator balance must decrease by the fee amount"
    );
    // Contract must hold the fee
    assert_eq!(
        token_balance(&env, &fee_token, &contract_id),
        fee,
        "Contract must receive exactly the fee amount"
    );
}

// ── Test 5: POOL_CREATED event is emitted on successful creation ──────────

/// `create_pool_with_fee` must emit at least one event from the contract
/// after a successful creation (the `POOL_CREATED` event published by
/// `create_pool`). We verify that the event list is non-empty and that
/// the contract is the publisher, confirming successful event emission.
#[test]
fn test_creation_fee_pool_created_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let fee: i128 = 100_000_000;
    let goal = 3_000_000_000u128;
    let title = String::from_str(&env, "Event Test Pool");
    let description = String::from_str(&env, "Checking event emission");

    let fee_token = create_token(&env, fee, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    client.create_pool_with_fee(
        &creator,
        &title,
        &description,
        &goal,
        &100_000u64,
        &fee_token,
    );

    // Retrieve all events emitted in this test context
    let events = env.events().all();

    // At least one event must have been emitted by the contract
    let contract_events_count = events
        .iter()
        .filter(|(contract, _topics, _data)| *contract == contract_id)
        .count();

    assert!(
        contract_events_count > 0,
        "At least one event (POOL_CREATED) must be emitted by the contract on successful creation"
    );
}
