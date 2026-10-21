#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, String,
};

// ============= ISSUE #1279: POOL CLOSURE AUTHORIZATION TESTS =============

/// (1) Admin/Sponsor can close Disbursed pool.
#[test]
fn test_admin_can_close_disbursed_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Disbursed Auth Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Mock state transition to Disbursed
    env.mock_all_auths();
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Close pool with sponsor auth
    client
        .mock_auths(&[MockAuth {
            address: &sponsor,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "close_pool",
                args: (&pool_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .close_pool(&pool_id);

    assert!(client.is_closed(&pool_id));
}

/// (2) Admin/Sponsor can close Cancelled pool.
#[test]
fn test_admin_can_close_cancelled_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Cancelled Auth Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Mock state transition to Cancelled
    env.mock_all_auths();
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    // Close pool with sponsor auth
    client
        .mock_auths(&[MockAuth {
            address: &sponsor,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "close_pool",
                args: (&pool_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .close_pool(&pool_id);

    assert!(client.is_closed(&pool_id));
}

/// (3) Non-admin cannot close pool.
#[test]
#[should_panic]
fn test_non_admin_cannot_close_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let non_admin = Address::generate(&env);

    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Unauthorized Close Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    env.mock_all_auths();
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Mock auth for non_admin instead of sponsor; close_pool requires sponsor auth
    client
        .mock_auths(&[MockAuth {
            address: &non_admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "close_pool",
                args: (&pool_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .close_pool(&pool_id);
}

/// (4) Closing Active pool fails with PoolNotDisbursedOrRefunded error (#8).
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_closing_active_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Active Close Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Pool state is Active; closing active pool fails
    client.close_pool(&pool_id);
}

/// (5) Closing already closed pool fails with PoolAlreadyClosed error (#15).
#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_closing_already_closed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Double Close Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);
    assert!(client.is_closed(&pool_id));

    // Second call to close_pool fails with PoolAlreadyClosed (#15)
    client.close_pool(&pool_id);
}

/// (6) Pool closure event emitted when pool is closed.
#[test]
fn test_pool_closure_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Event Pool"),
        &String::from_str(&env, "Testing pool closure event emission"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    let events = env.events().all();
    let contract_events = events.filter_by_contract(&contract_id);

    assert!(
        !contract_events.events().is_empty(),
        "Expected at least one event from contract after pool closure"
    );
}
