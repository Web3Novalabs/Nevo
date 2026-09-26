#![cfg(test)]

// ============= ISSUE #1291: `donate` TOKEN VALIDATION TESTS =============

use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn setup_pool_with_token(env: &Env) -> (ContractClient<'_>, u32, Address, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let creator = Address::generate(env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(env, "Token Pool"),
        &String::from_str(env, "Description"),
        &1_000_000u128,
        &200_000u64,
    );

    let donor = Address::generate(env);
    let token = create_token(env, 10_000i128, &donor);
    client.set_pool_token(&pool_id, &token);

    (client, pool_id, donor, token)
}

/// Donating with the pool's configured token succeeds and updates the
/// pool's collected total.
#[test]
fn test_donate_with_token_correct_token_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, donor, token) = setup_pool_with_token(&env);

    client.donate_with_token(&pool_id, &donor, &token, &1_000i128);

    assert_eq!(client.get_pool(&pool_id).3, 1_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 1_000u128);
}

/// Donating with a token other than the pool's configured token fails with
/// `TokenTransferFailed`.
#[test]
#[should_panic(expected = "TokenTransferFailed")]
fn test_donate_with_token_wrong_token_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, donor, _) = setup_pool_with_token(&env);
    let wrong_token = create_token(&env, 10_000i128, &donor);

    client.donate_with_token(&pool_id, &donor, &wrong_token, &1_000i128);
}

/// Donating with an address that is not a deployed token contract fails
/// cleanly rather than corrupting pool state.
#[test]
fn test_donate_with_token_invalid_token_address_handled() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "No Token Configured"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &200_000u64,
    );

    let donor = Address::generate(&env);
    let invalid_token = Address::generate(&env);

    let res = client.try_donate_with_token(&pool_id, &donor, &invalid_token, &1_000i128);
    assert!(res.is_err(), "Donation with an invalid token address must fail");
    assert_eq!(client.get_pool(&pool_id).3, 0u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 0u128);
}

/// A successful token donation actually moves the token balance from the
/// donor to the contract, in addition to updating the internal accounting.
#[test]
fn test_donate_with_token_transfer_mechanics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, donor, token) = setup_pool_with_token(&env);
    let contract_id = client.address.clone();

    client.donate_with_token(&pool_id, &donor, &token, &4_000i128);

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&donor), 6_000i128);
    assert_eq!(token_client.balance(&contract_id), 4_000i128);
    assert_eq!(client.get_pool(&pool_id).3, 4_000u128);
}
