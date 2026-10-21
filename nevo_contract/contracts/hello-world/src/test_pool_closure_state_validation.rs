#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ============= ISSUE #1280: POOL CLOSURE STATE VALIDATION TESTS =============

/// (1) Only Disbursed pools can be closed.
#[test]
fn test_close_pool_disbursed_state_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Disbursed Pool"),
        &String::from_str(&env, "Pool to be closed after disbursement"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Transition pool state to Disbursed
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Closing disbursed pool should succeed
    client.close_pool(&pool_id);

    // Verify pool is closed using is_closed function and get_pool tuple
    assert!(client.is_closed(&pool_id));
    assert!(client.get_pool(&pool_id).4);
}

/// (2) Only Cancelled pools can be closed.
#[test]
fn test_close_pool_cancelled_state_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cancelled Pool"),
        &String::from_str(&env, "Pool to be closed after cancellation"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Transition pool state to Cancelled
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    // Closing cancelled pool should succeed
    client.close_pool(&pool_id);

    // Verify pool is closed using is_closed function
    assert!(client.is_closed(&pool_id));
    assert!(client.get_pool(&pool_id).4);
}

/// (3a) Other states (Active) return PoolNotDisbursedOrRefunded error (#8).
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_close_pool_active_state_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Active Pool"),
        &String::from_str(&env, "Active pool should fail close_pool"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Pool state is Active by default. Attempting to close returns PoolNotDisbursedOrRefunded (#8)
    client.close_pool(&pool_id);
}

/// (3b) Other states (Paused) return PoolNotDisbursedOrRefunded error (#8).
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_close_pool_paused_state_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Paused Pool"),
        &String::from_str(&env, "Paused pool should fail close_pool"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Paused);
    client.close_pool(&pool_id);
}

/// (3c) Other states (Completed) return PoolNotDisbursedOrRefunded error (#8).
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_close_pool_completed_state_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Completed Pool"),
        &String::from_str(&env, "Completed pool should fail close_pool"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Completed);
    client.close_pool(&pool_id);
}

/// (4) Closed state persists correctly across multiple reads.
#[test]
fn test_closed_state_persists_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Persistence Pool"),
        &String::from_str(&env, "Testing closed state persistence"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    // Verify first read
    assert!(client.is_closed(&pool_id));
    assert!(client.get_pool(&pool_id).4);

    // Verify subsequent reads
    assert!(client.is_closed(&pool_id));
    let pool_info = client.get_pool(&pool_id);
    assert!(pool_info.4);
    assert_eq!(pool_info.0, pool_id);
    assert_eq!(pool_info.1, creator);
}

/// (5) is_closed function returns true when pool is closed and false initially.
#[test]
fn test_is_closed_function_returns_true() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "is_closed Test Pool"),
        &String::from_str(&env, "Testing is_closed contract function"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Before closure, is_closed returns false
    assert!(!client.is_closed(&pool_id));

    // Close pool after transitioning to Disbursed
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    // After closure, is_closed returns true
    assert_eq!(client.is_closed(&pool_id), true);
}
