#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, StorageKey},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

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

fn create_pool_with_funds(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token: &Address,
    amount: i128,
) -> u64 {
    let creator = Address::generate(env);
    let config = PoolConfig {
        name: soroban_sdk::String::from_str(env, "Event Pool"),
        description: soroban_sdk::String::from_str(env, "Test event pool"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
    };
    let pool_id = client.create_pool(&creator, &config);

    // Seed EventPool storage directly to simulate ticket sales
    let token_admin_client = token::StellarAssetClient::new(env, token);
    token_admin_client.mint(&client.address, &amount);

    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .set(&StorageKey::EventPool(pool_id), &amount);
    });

    pool_id
}

#[test]
fn test_withdraw_event_pool_success() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool_with_funds(&env, &client, &token, 1_000);

    let recipient = Address::generate(&env);
    let token_client = token::Client::new(&env, &token);

    let before = token_client.balance(&recipient);
    client.withdraw_event_pool(&pool_id, &recipient);
    let after = token_client.balance(&recipient);

    assert_eq!(after - before, 1_000);
}

#[test]
fn test_withdraw_event_pool_double_withdrawal_prevented() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool_with_funds(&env, &client, &token, 1_000);

    let recipient = Address::generate(&env);

    // First withdrawal succeeds
    assert_eq!(
        client.try_withdraw_event_pool(&pool_id, &recipient),
        Ok(Ok(()))
    );

    // Second withdrawal is blocked — funds already drained
    assert_eq!(
        client.try_withdraw_event_pool(&pool_id, &recipient),
        Err(Ok(CrowdfundingError::EventAlreadyDrained)),
        "double withdrawal must be prevented after funds are drained"
    );
}

#[test]
fn test_withdraw_event_pool_not_found() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let recipient = Address::generate(&env);
    assert_eq!(
        client.try_withdraw_event_pool(&999, &recipient),
        Err(Ok(CrowdfundingError::PoolNotFound))
    );
}

#[test]
fn test_withdraw_event_pool_no_funds() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let config = PoolConfig {
        name: soroban_sdk::String::from_str(&env, "Empty Pool"),
        description: soroban_sdk::String::from_str(&env, "no funds"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
    };
    let pool_id = client.create_pool(&creator, &config);

    let recipient = Address::generate(&env);
    assert_eq!(
        client.try_withdraw_event_pool(&pool_id, &recipient),
        Err(Ok(CrowdfundingError::InsufficientFees))
    );
}
