#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{
    base::errors::CrowdfundingError,
    base::types::PoolConfig,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup_test(env: &Env) -> (CrowdfundingContractClient, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    (client, admin, token_address)
}

#[test]
fn test_pool_remaining_time_future() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    // Pin the clock to a known value
    env.ledger().set_timestamp(1_000_000);

    let creator = Address::generate(&env);
    let config = PoolConfig {
        name: String::from_str(&env, "Test Pool"),
        description: String::from_str(&env, "A test pool"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 500,
        created_at: env.ledger().timestamp(),
    };

    let pool_id = client.create_pool(&creator, &config);

    let remaining = client.get_pool_remaining_time(&pool_id);
    assert_eq!(remaining, 500);
}

#[test]
fn test_pool_remaining_time_expired_returns_zero() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    env.ledger().set_timestamp(1_000_000);

    let creator = Address::generate(&env);
    let config = PoolConfig {
        name: String::from_str(&env, "Expired Pool"),
        description: String::from_str(&env, "A test pool"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 100,
        created_at: env.ledger().timestamp(),
    };

    let pool_id = client.create_pool(&creator, &config);

    // Advance the clock past the deadline
    env.ledger().set_timestamp(1_000_200);

    let remaining = client.get_pool_remaining_time(&pool_id);
    assert_eq!(remaining, 0);
}

#[test]
fn test_pool_remaining_time_not_found() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let result = client.try_get_pool_remaining_time(&999u64);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}
