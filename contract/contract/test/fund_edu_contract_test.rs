#![cfg(test)]

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig},
    contract::{FundEduContract, FundEduContractClient},
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (FundEduContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(FundEduContract, ());
    let client = FundEduContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token_address, &0);
    (client, admin, token_address)
}

#[test]
fn test_fund_edu_initialize_and_is_paused() {
    let env = Env::default();
    let (client, _, _) = setup(&env);
    assert!(!client.is_paused());
}

#[test]
fn test_fund_edu_create_pool_success() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let creator = Address::generate(&env);
    let config = PoolConfig {
        name: String::from_str(&env, "STEM 2026 Q1"),
        description: String::from_str(&env, "Scholarship for STEM students"),
        target_amount: 50_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: admin.clone(),
    };

    let pool_id = client.create_pool(&creator, &config);
    assert_eq!(pool_id, 1);

    let saved = client.get_pool(&pool_id).unwrap();
    assert_eq!(saved.name, config.name);
    assert_eq!(saved.target_amount, config.target_amount);
}

#[test]
fn test_fund_edu_pause_unpause() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    client.pause();
    assert!(client.is_paused());

    client.unpause();
    assert!(!client.is_paused());
}

#[test]
fn test_fund_edu_create_pool_paused_returns_error() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    client.pause();

    let creator = Address::generate(&env);
    let config = PoolConfig {
        name: String::from_str(&env, "Blocked Pool"),
        description: String::from_str(&env, "Should fail"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address,
    };

    let result = client.try_create_pool(&creator, &config);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractPaused)));
}
