#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig, types::StorageKey},
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

fn create_pool(client: &CrowdfundingContractClient<'_>, env: &Env, token: &Address) -> u64 {
    let creator = Address::generate(env);
    let config = PoolConfig {
        name: soroban_sdk::String::from_str(env, "Ticket Pool"),
        description: soroban_sdk::String::from_str(env, "reentrancy test"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
    };
    client.create_pool(&creator, &config)
}

/// Verify that the reentrancy lock is set during buy_ticket by checking that
/// a concurrent call on the same pool is rejected with Unauthorized while the
/// lock is held, and that the lock is released cleanly after the call so a
/// subsequent call succeeds.
#[test]
fn test_buy_ticket_reentrancy_lock_engaged_and_released() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    // Manually set the reentrancy lock to simulate a concurrent in-flight call
    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .set(&StorageKey::ReentrancyLock(pool_id), &true);
    });

    // buy_ticket must be rejected while the lock is held
    let buyer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&buyer, &1_000);

    let result = client.try_buy_ticket(&pool_id, &buyer, &token, &1_000);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::Unauthorized)),
        "buy_ticket must be blocked while the reentrancy lock is held"
    );

    // Release the lock (simulates the original call completing)
    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .remove(&StorageKey::ReentrancyLock(pool_id));
    });

    // After the lock is released, buy_ticket must succeed
    let result = client.try_buy_ticket(&pool_id, &buyer, &token, &1_000);
    assert_eq!(
        result,
        Ok(Ok((1_000, 0))),
        "buy_ticket must succeed once the reentrancy lock is released"
    );
}

/// Verify that the lock is released after a successful buy_ticket call so
/// subsequent calls on the same pool are not permanently blocked.
#[test]
fn test_buy_ticket_lock_released_after_success() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    let buyer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&buyer, &2_000);

    // First call succeeds
    assert_eq!(
        client.try_buy_ticket(&pool_id, &buyer, &token, &1_000),
        Ok(Ok((1_000, 0)))
    );

    // Second call also succeeds — lock was released after the first
    assert_eq!(
        client.try_buy_ticket(&pool_id, &buyer, &token, &1_000),
        Ok(Ok((1_000, 0)))
    );
}
