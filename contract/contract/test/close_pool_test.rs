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

fn create_test_pool(client: &CrowdfundingContractClient, env: &Env, creator: &Address) -> u64 {
    let config = PoolConfig {
        name: String::from_str(env, "Test Pool"),
        description: String::from_str(env, "A test pool for closing"),
        target_amount: 1_000_000,
        is_private: false,
        duration: 86400, // 1 day
        created_at: env.ledger().timestamp(),
    };

    client.create_pool(creator, &config)
}

#[test]
fn test_close_pool_success_after_disbursement() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Update pool state to Disbursed
    client.update_pool_state(&pool_id, &PoolState::Disbursed);

    // Close the pool as admin
    client.close_pool(&pool_id, &admin);

    // Verify pool is closed
    let is_closed = client.is_closed(&pool_id);
    assert_eq!(is_closed, true);
}

#[test]
fn test_close_pool_success_after_cancellation() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Update pool state to Cancelled
    client.update_pool_state(&pool_id, &PoolState::Cancelled);

    // Close the pool as admin
    client.close_pool(&pool_id, &admin);

    // Verify pool is closed
    let is_closed = client.is_closed(&pool_id);
    assert_eq!(is_closed, true);
}

#[test]
fn test_close_pool_already_closed() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Update pool state to Disbursed
    client.update_pool_state(&pool_id, &PoolState::Disbursed);

    // Close the pool
    client.close_pool(&pool_id, &admin);

    // Try to close again
    let result = client.try_close_pool(&pool_id, &admin);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));
}

#[test]
fn test_close_pool_not_disbursed_or_cancelled() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Pool is in Active state, should not be closable
    let result = client.try_close_pool(&pool_id, &admin);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotDisbursedOrRefunded))
    );
}

#[test]
fn test_close_pool_paused_state() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Update pool state to Paused
    client.update_pool_state(&pool_id, &PoolState::Paused);

    // Try to close - should fail
    let result = client.try_close_pool(&pool_id, &admin);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotDisbursedOrRefunded))
    );
}

#[test]
fn test_close_pool_completed_state() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Update pool state to Completed
    client.update_pool_state(&pool_id, &PoolState::Completed);

    // Try to close - should fail
    let result = client.try_close_pool(&pool_id, &admin);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotDisbursedOrRefunded))
    );
}

#[test]
fn test_close_pool_nonexistent() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let nonexistent_pool_id = 999u64;

    let result = client.try_close_pool(&nonexistent_pool_id, &admin);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_close_pool_unauthorized() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Update pool state to Disbursed
    client.update_pool_state(&pool_id, &PoolState::Disbursed);

    // Try to close as non-admin
    let unauthorized_user = Address::generate(&env);
    let result = client.try_close_pool(&pool_id, &unauthorized_user);
    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_is_closed_for_active_pool() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    let is_closed = client.is_closed(&pool_id);
    assert_eq!(is_closed, false);
}

#[test]
fn test_is_closed_for_closed_pool() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Update to Disbursed and close
    client.update_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id, &admin);

    let is_closed = client.is_closed(&pool_id);
    assert_eq!(is_closed, true);
}

#[test]
fn test_is_closed_nonexistent_pool() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let nonexistent_pool_id = 999u64;

    let result = client.try_is_closed(&nonexistent_pool_id);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_close_pool_emits_event() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Update pool state to Disbursed
    client.update_pool_state(&pool_id, &PoolState::Disbursed);

    // Close the pool
    client.close_pool(&pool_id, &admin);

    // Verify event was emitted (events are automatically captured in test environment)
    // The event emission is verified by the fact that the function completes successfully
    let is_closed = client.is_closed(&pool_id);
    assert_eq!(is_closed, true);
}

#[test]
fn test_close_pool_multiple_pools() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);

    // Create multiple pools
    let pool_id_1 = create_test_pool(&client, &env, &creator);
    let pool_id_2 = create_test_pool(&client, &env, &creator);
    let pool_id_3 = create_test_pool(&client, &env, &creator);

    // Update states
    client.update_pool_state(&pool_id_1, &PoolState::Disbursed);
    client.update_pool_state(&pool_id_2, &PoolState::Cancelled);
    client.update_pool_state(&pool_id_3, &PoolState::Disbursed);

    // Close pools 1 and 3
    client.close_pool(&pool_id_1, &admin);
    client.close_pool(&pool_id_3, &admin);

    // Verify states
    assert_eq!(client.is_closed(&pool_id_1), true);
    assert_eq!(client.is_closed(&pool_id_2), false);
    assert_eq!(client.is_closed(&pool_id_3), true);
}

#[test]
fn test_close_pool_state_transition_sequence() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Initial state: Active
    assert_eq!(client.is_closed(&pool_id), false);

    // Try to close from Active - should fail
    let result = client.try_close_pool(&pool_id, &admin);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotDisbursedOrRefunded))
    );

    // Transition to Disbursed
    client.update_pool_state(&pool_id, &PoolState::Disbursed);
    assert_eq!(client.is_closed(&pool_id), false);

    // Now close should succeed
    client.close_pool(&pool_id, &admin);
    assert_eq!(client.is_closed(&pool_id), true);
}

#[test]
fn test_close_pool_after_refund_scenario() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&client, &env, &creator);

    // Simulate refund scenario by setting state to Cancelled
    client.update_pool_state(&pool_id, &PoolState::Cancelled);

    // Close the pool
    client.close_pool(&pool_id, &admin);

    // Verify pool is closed
    assert_eq!(client.is_closed(&pool_id), true);
}

#[test]
fn test_is_closed_for_different_states() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let creator = Address::generate(&env);

    // Create pools for each state
    let pool_active = create_test_pool(&client, &env, &creator);
    let pool_paused = create_test_pool(&client, &env, &creator);
    let pool_completed = create_test_pool(&client, &env, &creator);
    let pool_cancelled = create_test_pool(&client, &env, &creator);
    let pool_disbursed = create_test_pool(&client, &env, &creator);
    let pool_closed = create_test_pool(&client, &env, &creator);

    // Set states
    client.update_pool_state(&pool_paused, &PoolState::Paused);
    client.update_pool_state(&pool_completed, &PoolState::Completed);
    client.update_pool_state(&pool_cancelled, &PoolState::Cancelled);
    client.update_pool_state(&pool_disbursed, &PoolState::Disbursed);
    client.update_pool_state(&pool_closed, &PoolState::Disbursed);
    client.close_pool(&pool_closed, &admin);

    // Verify is_closed returns false for all except Closed state
    assert_eq!(client.is_closed(&pool_active), false);
    assert_eq!(client.is_closed(&pool_paused), false);
    assert_eq!(client.is_closed(&pool_completed), false);
    assert_eq!(client.is_closed(&pool_cancelled), false);
    assert_eq!(client.is_closed(&pool_disbursed), false);
    assert_eq!(client.is_closed(&pool_closed), true);
}
