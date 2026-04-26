#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token, &0);
    (client, admin, token)
}

fn pool_config(env: &Env, token: &Address, validator: &Address) -> PoolConfig {
    PoolConfig {
        name: String::from_str(env, "Scholarship Pool"),
        description: String::from_str(env, "Test pool"),
        target_amount: 10_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: validator.clone(),
    }
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    soroban_sdk::token::StellarAssetClient::new(env, token).mint(to, &amount);
}

// ── register_school ───────────────────────────────────────────────────────────

#[test]
fn test_register_school_succeeds() {
    let env = Env::default();
    let (client, admin, _) = setup(&env);

    let school = Address::generate(&env);
    let result = client.try_register_school(
        &school,
        &String::from_str(&env, "MIT"),
        &String::from_str(&env, "US"),
        &String::from_str(&env, "ACC-MIT-001"),
    );
    assert_eq!(result, Ok(Ok(())));
}

#[test]
fn test_is_validator_registered_returns_false_before_registration() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let school = Address::generate(&env);
    assert!(!client.is_validator_registered(&school));
}

#[test]
fn test_is_validator_registered_returns_true_after_registration() {
    let env = Env::default();
    let (client, admin, _) = setup(&env);

    let school = Address::generate(&env);
    client.register_school(
        &school,
        &String::from_str(&env, "Harvard"),
        &String::from_str(&env, "US"),
        &String::from_str(&env, "ACC-HARV-001"),
    );

    assert!(client.is_validator_registered(&school));
}

#[test]
fn test_register_school_can_be_overwritten() {
    let env = Env::default();
    let (client, admin, _) = setup(&env);

    let school = Address::generate(&env);
    client.register_school(
        &school,
        &String::from_str(&env, "Old Name"),
        &String::from_str(&env, "US"),
        &String::from_str(&env, "ACC-OLD"),
    );
    // Re-register with updated metadata
    client.register_school(
        &school,
        &String::from_str(&env, "New Name"),
        &String::from_str(&env, "UK"),
        &String::from_str(&env, "ACC-NEW"),
    );

    assert!(client.is_validator_registered(&school));
}

#[test]
fn test_multiple_schools_can_be_registered() {
    let env = Env::default();
    let (client, admin, _) = setup(&env);

    let school1 = Address::generate(&env);
    let school2 = Address::generate(&env);
    let school3 = Address::generate(&env);

    client.register_school(
        &school1,
        &String::from_str(&env, "MIT"),
        &String::from_str(&env, "US"),
        &String::from_str(&env, "ACC-1"),
    );
    client.register_school(
        &school2,
        &String::from_str(&env, "Oxford"),
        &String::from_str(&env, "UK"),
        &String::from_str(&env, "ACC-2"),
    );
    client.register_school(
        &school3,
        &String::from_str(&env, "ETH Zurich"),
        &String::from_str(&env, "CH"),
        &String::from_str(&env, "ACC-3"),
    );

    assert!(client.is_validator_registered(&school1));
    assert!(client.is_validator_registered(&school2));
    assert!(client.is_validator_registered(&school3));
}

// ── create_pool registry enforcement ─────────────────────────────────────────

#[test]
fn test_create_pool_fails_with_unregistered_validator() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let unregistered_validator = Address::generate(&env);

    mint(&env, &token, &creator, 10_000);

    let config = pool_config(&env, &token, &unregistered_validator);
    let result = client.try_create_pool(&creator, &config);

    assert_eq!(result, Err(Ok(CrowdfundingError::UnrecognizedValidator)));
}

#[test]
fn test_create_pool_succeeds_with_registered_validator() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);

    client.register_school(
        &validator,
        &String::from_str(&env, "Test University"),
        &String::from_str(&env, "US"),
        &String::from_str(&env, "ACC-001"),
    );

    mint(&env, &token, &creator, 10_000);

    let config = pool_config(&env, &token, &validator);
    let pool_id = client.create_pool(&creator, &config);

    assert_eq!(pool_id, 1);
    let saved = client.get_pool(&pool_id).unwrap();
    assert_eq!(saved.validator, validator);
}

#[test]
fn test_create_pool_unregistered_validator_does_not_create_pool() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let unregistered = Address::generate(&env);

    mint(&env, &token, &creator, 10_000);

    let config = pool_config(&env, &token, &unregistered);
    let _ = client.try_create_pool(&creator, &config);

    // Pool must not have been created
    assert!(client.get_pool(&1).is_none());
}

#[test]
fn test_create_pool_registry_check_fires_before_balance_check() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let unregistered = Address::generate(&env);

    // No mint — creator has zero balance
    let config = pool_config(&env, &token, &unregistered);
    let result = client.try_create_pool(&creator, &config);

    // Registry check fires before balance check
    assert_eq!(result, Err(Ok(CrowdfundingError::UnrecognizedValidator)));
}

#[test]
fn test_create_pool_registry_check_fires_before_token_check() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let creator = Address::generate(&env);
    let unregistered = Address::generate(&env);

    // Use a wrong token
    let wrong_token_admin = Address::generate(&env);
    let wrong_token = env
        .register_stellar_asset_contract_v2(wrong_token_admin)
        .address();

    let config = pool_config(&env, &wrong_token, &unregistered);
    let result = client.try_create_pool(&creator, &config);

    // Registry check fires before token check
    assert_eq!(result, Err(Ok(CrowdfundingError::UnrecognizedValidator)));
}
