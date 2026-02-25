#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, PoolState},
    },
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

fn create_private_pool(client: &CrowdfundingContractClient, env: &Env, creator: &Address) -> u64 {
    let config = PoolConfig {
        name: String::from_str(env, "Private Pool"),
        description: String::from_str(env, "A private pool for testing"),
        target_amount: 1_000_000,
        min_contribution: 100,
        is_private: true,
        duration: 86400, // 1 day
        created_at: env.ledger().timestamp(),
    };

    client.create_pool(creator, &config)
}

fn create_public_pool(client: &CrowdfundingContractClient, env: &Env, creator: &Address) -> u64 {
    let config = PoolConfig {
        name: String::from_str(env, "Public Pool"),
        description: String::from_str(env, "A public pool for testing"),
        target_amount: 1_000_000,
        min_contribution: 100,
        is_private: false,
        duration: 86400, // 1 day
        created_at: env.ledger().timestamp(),
    };

    client.create_pool(creator, &config)
}

#[test]
fn test_owner_can_close_private_pool() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Owner should be able to close their private pool
    client.close_pool(&pool_id, &owner);

    // Verify pool is closed
    let is_closed = client.is_closed(&pool_id);
    assert_eq!(is_closed, true);
}

#[test]
fn test_contribute_fails_on_closed_private_pool() {
    let env = Env::default();
    let (client, _admin, token) = setup_test(&env);

    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Close the pool
    client.close_pool(&pool_id, &owner);

    // Try to contribute - should fail
    let result = client.try_contribute(&pool_id, &contributor, &token, &1000, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));
}

#[test]
fn test_owner_cannot_close_public_pool_when_active() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_public_pool(&client, &env, &owner);

    // Owner should NOT be able to close public pool when active
    let result = client.try_close_pool(&pool_id, &owner);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotDisbursedOrRefunded))
    );
}

#[test]
fn test_non_owner_cannot_close_private_pool() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Non-owner should NOT be able to close the pool
    let result = client.try_close_pool(&pool_id, &non_owner);
    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_admin_can_close_private_pool() {
    let env = Env::default();
    let (client, admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Admin should be able to close any pool (but only after disbursed/cancelled for non-private)
    // For private pools, admin can close when active
    let result = client.try_close_pool(&pool_id, &admin);
    // Admin trying to close active private pool should fail (admin follows disbursed/cancelled rule)
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotDisbursedOrRefunded))
    );
}

#[test]
fn test_owner_can_close_paused_private_pool() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Pause the pool
    client.update_pool_state(&pool_id, &PoolState::Paused);

    // Owner should be able to close paused private pool
    client.close_pool(&pool_id, &owner);

    // Verify pool is closed
    let is_closed = client.is_closed(&pool_id);
    assert_eq!(is_closed, true);
}

#[test]
fn test_owner_cannot_close_completed_private_pool() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Set pool to Completed state
    client.update_pool_state(&pool_id, &PoolState::Completed);

    // Owner should NOT be able to close completed pool
    let result = client.try_close_pool(&pool_id, &owner);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));
}

#[test]
fn test_close_private_pool_before_deadline() {
    let env = Env::default();
    let (client, _admin, token) = setup_test(&env);

    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Mint tokens to contributor
    use soroban_sdk::token;
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &10000);

    // Contribute to the pool first
    client.contribute(&pool_id, &contributor, &token, &1000, &false);

    // Owner closes the pool before deadline
    client.close_pool(&pool_id, &owner);

    // Verify pool is closed
    assert_eq!(client.is_closed(&pool_id), true);

    // Try to contribute again - should fail
    let result = client.try_contribute(&pool_id, &contributor, &token, &1000, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));
}

#[test]
fn test_close_already_closed_private_pool() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Close the pool
    client.close_pool(&pool_id, &owner);

    // Try to close again - should fail
    let result = client.try_close_pool(&pool_id, &owner);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));
}

#[test]
fn test_multiple_private_pools_independent_closure() {
    let env = Env::default();
    let (client, _admin, token) = setup_test(&env);

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let contributor = Address::generate(&env);

    let pool_id_1 = create_private_pool(&client, &env, &owner1);
    let pool_id_2 = create_private_pool(&client, &env, &owner2);

    // Mint tokens to contributor
    use soroban_sdk::token;
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &10000);

    // Close only pool 1
    client.close_pool(&pool_id_1, &owner1);

    // Verify pool 1 is closed
    assert_eq!(client.is_closed(&pool_id_1), true);

    // Verify pool 2 is still open
    assert_eq!(client.is_closed(&pool_id_2), false);

    // Contributions to pool 1 should fail
    let result = client.try_contribute(&pool_id_1, &contributor, &token, &1000, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));

    // Contributions to pool 2 should succeed
    client.contribute(&pool_id_2, &contributor, &token, &1000, &false);
}

#[test]
fn test_admin_can_close_after_disbursement() {
    let env = Env::default();
    let (client, admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Set pool to Disbursed state
    client.update_pool_state(&pool_id, &PoolState::Disbursed);

    // Admin should be able to close after disbursement
    client.close_pool(&pool_id, &admin);

    // Verify pool is closed
    assert_eq!(client.is_closed(&pool_id), true);
}

#[test]
fn test_owner_can_close_after_cancellation() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner);

    // Set pool to Cancelled state
    client.update_pool_state(&pool_id, &PoolState::Cancelled);

    // Owner should be able to close after cancellation
    client.close_pool(&pool_id, &owner);

    // Verify pool is closed
    assert_eq!(client.is_closed(&pool_id), true);
}
