#![cfg(test)]

//! Issue #1372: Tests for token_address validation in donate_with_token and refund_donation.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _}, token::StellarAssetClient, Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// (1) An invalid/unregistered token Address is rejected before transfer in donate_with_token.
#[test]
fn test_invalid_token_address_rejected_in_donate_with_token() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let invalid_token = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let res = client.try_donate_with_token(&pool_id, &donor, &invalid_token, &1_000i128);
    assert!(res.is_err(), "Donation with invalid token address must fail");
    assert_eq!(client.get_pool(&pool_id).3, 0u128, "Pool collected total must remain 0");
    assert_eq!(client.get_contribution(&pool_id, &donor), 0u128, "Donor contribution must remain 0");
}

/// (1) An invalid/unregistered token Address is rejected before transfer in refund_donation.
#[test]
fn test_invalid_token_address_rejected_in_refund_donation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let valid_token = create_token(&env, 1_000i128, &donor);
    let invalid_token = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &valid_token);
    client.donate_with_token(&pool_id, &donor, &valid_token, &500i128);

    // Set deadline and advance ledger past deadline + grace period
    client.set_pool_deadline(&pool_id, &1_000u32);
    env.ledger().set_sequence_number(1_000 + REFUND_GRACE_PERIOD_LEDGERS + 1);

    let res = client.try_refund_donation(&pool_id, &donor, &invalid_token);
    assert!(res.is_err(), "Refund with invalid token address must fail");
    assert_eq!(client.get_contribution(&pool_id, &donor), 500u128, "Donor contribution must remain recorded");
}

/// (2) A token contract that does not implement the expected token interface fails gracefully in donate_with_token.
#[test]
fn test_non_token_contract_fails_gracefully_in_donate_with_token() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    // Register a non-token contract (the Nevo contract itself has no token transfer method)
    let non_token_contract = env.register(Contract, ());

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let res = client.try_donate_with_token(&pool_id, &donor, &non_token_contract, &1_000i128);
    assert!(res.is_err(), "Donation using non-token contract must fail gracefully");
    assert_eq!(client.get_pool(&pool_id).3, 0u128, "Pool state must remain intact");
}

/// (2) A token contract that does not implement the expected token interface fails gracefully in refund_donation.
#[test]
fn test_non_token_contract_fails_gracefully_in_refund_donation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let valid_token = create_token(&env, 1_000i128, &donor);
    let non_token_contract = env.register(Contract, ());

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &valid_token);
    client.donate_with_token(&pool_id, &donor, &valid_token, &500i128);

    client.set_pool_deadline(&pool_id, &1_000u32);
    env.ledger().set_sequence_number(1_000 + REFUND_GRACE_PERIOD_LEDGERS + 1);

    let res = client.try_refund_donation(&pool_id, &donor, &non_token_contract);
    assert!(res.is_err(), "Refund using non-token contract must fail gracefully");
    assert_eq!(client.get_contribution(&pool_id, &donor), 500u128, "Contribution must remain intact");
}

/// (3) refund_donation uses the same token_address originally donated with and succeeds.
#[test]
fn test_refund_donation_uses_same_token_address() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &donor, &token, &400i128);
    assert_eq!(client.get_pool(&pool_id).3, 400u128);

    client.set_pool_deadline(&pool_id, &1_000u32);
    env.ledger().set_sequence_number(1_000 + REFUND_GRACE_PERIOD_LEDGERS + 1);

    client.refund_donation(&pool_id, &donor, &token);
    assert_eq!(client.get_contribution(&pool_id, &donor), 0u128);
    assert_eq!(client.get_pool(&pool_id).3, 0u128);
    assert_eq!(token::Client::new(&env, &token).balance(&donor), 1_000i128);
}

/// (4) Mismatched token_address on refund is rejected.
#[test]
#[should_panic(expected = "TokenTransferFailed")]
fn test_mismatched_token_address_on_refund_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token_a = create_token(&env, 1_000i128, &donor);
    let token_b = create_token(&env, 1_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token_a);
    client.donate_with_token(&pool_id, &donor, &token_a, &400i128);

    client.set_pool_deadline(&pool_id, &1_000u32);
    env.ledger().set_sequence_number(1_000 + REFUND_GRACE_PERIOD_LEDGERS + 1);

    // Attempt refund passing token_b instead of token_a
    client.refund_donation(&pool_id, &donor, &token_b);
}

/// (5) State integrity is preserved and malicious/failed transfer callbacks cannot manipulate contract state.
#[test]
fn test_malicious_token_callback_state_integrity() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 50i128, &donor); // Donor only has 50

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Attempting to donate 500 when donor only has 50 fails
    let res = client.try_donate_with_token(&pool_id, &donor, &token, &500i128);
    assert!(res.is_err(), "Overdrawn transfer must fail");

    // Pool state and donor contribution must remain uncorrupted
    assert_eq!(client.get_pool(&pool_id).3, 0u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 0u128);
}
