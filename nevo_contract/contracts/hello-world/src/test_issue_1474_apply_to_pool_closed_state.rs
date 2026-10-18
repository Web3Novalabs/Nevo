#![cfg(test)]
//! Tests for `apply_to_pool` behavior on closed and cancelled pools - issue #1474.
//!
//! `apply_to_pool` loads the `Pool` record only to confirm it exists (`let _: Pool = ...`),
//! without checking `pool.is_closed` or `pool.state`.
//!
//! Covers:
//!   (1) Closing a pool via `close_pool` (after setting state to `Cancelled`/`Disbursed`)
//!       and confirming `apply_to_pool` currently still accepts and records the application.
//!   (2) Setting a pool state to `PoolState::Cancelled` via `set_pool_state` and confirming
//!       `apply_to_pool` currently still succeeds and increments `get_application_count`.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_apply_to_pool_on_closed_pool_currently_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closed Pool Application Test"),
        &String::from_str(&env, "Testing apply_to_pool after close_pool"),
        &10_000_000u128,
        &100_000u64,
    );

    // Transition pool to Disbursed so close_pool succeeds, then close it
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);
    assert!(client.is_pool_closed(&pool_id));

    // Because apply_to_pool discards the loaded Pool binding without checking
    // pool.is_closed, applying to the closed pool still succeeds.
    let app_data = String::from_str(&env, "Application submitted after close_pool");
    client.apply_to_pool(&pool_id, &student, &app_data);

    assert!(client.has_applied(&pool_id, &student));
    assert_eq!(client.get_application_count(&pool_id), 1u32);
}

#[test]
fn test_apply_to_pool_on_cancelled_pool_state_currently_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cancelled Pool Application Test"),
        &String::from_str(&env, "Testing apply_to_pool in Cancelled state"),
        &10_000_000u128,
        &100_000u64,
    );

    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    assert_eq!(client.get_pool_state(&pool_id), PoolState::Cancelled);

    // Unlike donate(), apply_to_pool does not check pool.state != PoolState::Active.
    let app_data = String::from_str(&env, "Application on Cancelled pool");
    client.apply_to_pool(&pool_id, &student, &app_data);

    assert!(client.has_applied(&pool_id, &student));
    assert_eq!(client.get_application_count(&pool_id), 1u32);
}