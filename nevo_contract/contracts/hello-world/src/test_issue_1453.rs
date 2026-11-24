#![cfg(test)]
//! Issue #1453 — set_pool_state has no authorization check at all.
//!
//! set_pool_state (lib.rs ~2456) is marked as a mock testing helper but is a
//! public contract entrypoint with zero require_auth(). Any caller can
//! transition any pool to any PoolState, including Cancelled or Disbursed.
//!
//! These tests document the current behaviour. This needs either removal from
//! the production build or a real authorization gate before shipping.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Issue #1453, test 1.
///
/// An arbitrary address (not the pool sponsor, not the admin) calls
/// set_pool_state to force a pool into Cancelled. The call succeeds and
/// get_pool_state reflects the change.
#[test]
fn test_arbitrary_address_can_set_pool_state() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    // attacker is unrelated to the pool.
    let _attacker = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1453 Pool"),
        &String::from_str(&env, "No auth gate on set_pool_state"),
        &1_000_000i128,
        &1_000_000u64,
    );

    // Pool starts Active.
    assert_eq!(client.get_pool_state(&pool_id), PoolState::Active);

    // Any caller can force the state to Cancelled with no auth required.
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    // The state is now Cancelled.
    assert_eq!(client.get_pool_state(&pool_id), PoolState::Cancelled);
}

/// Issue #1453, test 2.
///
/// A pool forced into Cancelled by an arbitrary caller can no longer accept
/// donations. donate() panics with InvalidPoolState, demonstrating that the
/// unauthorized state change has real operational impact.
#[test]
#[should_panic(expected = "Error(Contract, #")]
fn test_set_pool_state_affects_donate_behavior() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1453 Donate Pool"),
        &String::from_str(&env, "State forced to Cancelled blocks donate"),
        &1_000_000i128,
        &1_000_000u64,
    );

    // Force pool into Cancelled without any authorization.
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    // donate() now rejects with InvalidPoolState.
    client.donate(&pool_id, &donor, &1_000i128);
}

/// Issue #1453, test 3.
///
/// An arbitrary caller forces a pool into Disbursed. Subsequent donate calls
/// also panic, confirming the unguarded state setter has the same operational
/// consequences as the legitimate lifecycle transition.
#[test]
#[should_panic(expected = "Error(Contract, #")]
fn test_set_pool_state_affects_close_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1453 Disbursed Pool"),
        &String::from_str(&env, "Any address can force Disbursed"),
        &1_000_000i128,
        &1_000_000u64,
    );

    // Any address can force the pool into Disbursed with no auth check.
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    assert_eq!(client.get_pool_state(&pool_id), PoolState::Disbursed);

    // The pool is now in Disbursed state; donate() panics with
    // InvalidPoolState, confirming the forced state has real effect.
    client.donate(&pool_id, &donor, &500i128);
}
