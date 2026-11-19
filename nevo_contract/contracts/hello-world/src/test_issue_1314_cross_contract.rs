#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(token_admin);
    StellarAssetClient::new(env, &token.address()).mint(recipient, &amount);
    token.address()
}
fn new_pool(env: &Env, client: &ContractClient, creator: &Address) -> u32 {
    client.create_pool(
        creator,
        &String::from_str(env, "Token pool"),
        &String::from_str(env, "Cross-contract test"),
        &1_000u128,
        &100_000u64,
    )
}

#[test]
fn test_cross_contract_token_transfer_updates_both_balances() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 500i128, &donor);
    let pool_id = new_pool(&env, &client, &creator);

    client.donate_with_token(&pool_id, &donor, &token, &200i128);

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&donor), 300i128);
    assert_eq!(token_client.balance(&contract_id), 200i128);
    assert_eq!(client.get_pool(&pool_id).3, 200u128);
}

#[test]
fn test_cross_contract_failed_transfer_preserves_pool_and_donor_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100i128, &donor);
    let pool_id = new_pool(&env, &client, &creator);

    let result = client.try_donate_with_token(&pool_id, &donor, &token, &101i128);

    assert!(result.is_err());
    assert_eq!(client.get_pool(&pool_id).3, 0u128);
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&donor),
        100i128
    );
}

#[test]
fn test_cross_contract_configured_token_rejects_other_token() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let accepted_token = create_token(&env, 100i128, &donor);
    let other_token = create_token(&env, 100i128, &donor);
    let pool_id = new_pool(&env, &client, &creator);

    client.set_pool_token(&pool_id, &accepted_token);
    let result = client.try_donate_with_token(&pool_id, &donor, &other_token, &1i128);

    assert!(result.is_err());
    assert_eq!(client.get_pool(&pool_id).3, 0u128);
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &other_token).balance(&donor),
        100i128
    );
}

#[test]
fn test_cross_contract_token_interface_accepts_repeated_transfers() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 300i128, &donor);
    let pool_id = new_pool(&env, &client, &creator);

    client.donate_with_token(&pool_id, &donor, &token, &100i128);
    client.donate_with_token(&pool_id, &donor, &token, &200i128);

    assert_eq!(client.get_pool(&pool_id).3, 300u128);
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&contract_id),
        300i128
    );
}
