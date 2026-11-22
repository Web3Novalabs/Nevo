#![cfg(test)]
//! Tests for `PoolState` enum completeness — issue #1355.
//!
//! Scenarios:
//!   1. Every declared state is reachable through the public API.
//!   2. The default state on pool creation is `Active`.
//!   3. No orphan/unreachable variants exist (the exhaustive match below stops
//!      compiling if a variant is added without a test path).
//!   4. Terminal states cannot transition further where disallowed.
//!   5. `get_pool` reports state consistently with internal storage.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

const GOAL: u128 = 1_000_000;

fn setup(env: &Env) -> (Address, ContractClient<'_>, u32) {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let pool_id = client.create_pool(
        &Address::generate(env),
        &String::from_str(env, "State Pool"),
        &String::from_str(env, "PoolState completeness"),
        &GOAL,
        &200_000u64,
    );
    (contract_id, client, pool_id)
}

fn stored_pool(env: &Env, contract_id: &Address, pool_id: u32) -> Pool {
    env.as_contract(contract_id, || {
        env.storage().persistent().get::<_, Pool>(&pool_id).unwrap()
    })
}

/// Index of every variant. Adding a `PoolState` variant makes this match
/// non-exhaustive, so the new state has to be wired into these tests.
fn variant_index(state: &PoolState) -> usize {
    match state {
        PoolState::Active => 0,
        PoolState::Paused => 1,
        PoolState::Completed => 2,
        PoolState::Cancelled => 3,
        PoolState::Disbursed => 4,
        PoolState::Closed => 5,
    }
}

const VARIANT_COUNT: usize = 6;

/// Drive a fresh pool into `target` using only the public API.
fn reach(env: &Env, target: PoolState) -> PoolState {
    let (contract_id, client, pool_id) = setup(env);
    match target {
        PoolState::Active => {}
        PoolState::Completed => {
            client.donate(&pool_id, &Address::generate(env), &GOAL);
        }
        PoolState::Paused | PoolState::Cancelled | PoolState::Disbursed => {
            client.set_pool_state(&pool_id, &target);
        }
        PoolState::Closed => {
            client.set_pool_state(&pool_id, &PoolState::Cancelled);
            client.close_pool(&pool_id);
        }
    }
    stored_pool(env, &contract_id, pool_id).state
}

// ── 1 & 3: every variant is reachable, none is orphaned ──────────────────────

#[test]
fn test_every_pool_state_is_reachable_through_public_api() {
    let targets = [
        PoolState::Active,
        PoolState::Paused,
        PoolState::Completed,
        PoolState::Cancelled,
        PoolState::Disbursed,
        PoolState::Closed,
    ];
    let mut reached = [false; VARIANT_COUNT];

    for target in targets {
        let env = Env::default();
        let state = reach(&env, target.clone());
        assert_eq!(state, target, "pool did not reach {:?}", target);
        reached[variant_index(&state)] = true;
    }

    assert!(
        reached.iter().all(|r| *r),
        "every PoolState variant must be reachable, reached: {:?}",
        reached
    );
}

#[test]
fn test_closed_is_reachable_from_disbursed() {
    let env = Env::default();
    let (contract_id, client, pool_id) = setup(&env);

    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    let pool = stored_pool(&env, &contract_id, pool_id);
    assert_eq!(pool.state, PoolState::Closed);
    assert!(pool.is_closed);
}

// ── 2: default state ─────────────────────────────────────────────────────────

#[test]
fn test_new_pool_defaults_to_active() {
    let env = Env::default();
    let (contract_id, _client, pool_id) = setup(&env);

    let pool = stored_pool(&env, &contract_id, pool_id);
    assert_eq!(pool.state, PoolState::Active);
    assert!(!pool.is_closed);
}

#[test]
fn test_partial_donation_keeps_pool_active() {
    let env = Env::default();
    let (contract_id, client, pool_id) = setup(&env);

    client.donate(&pool_id, &Address::generate(&env), &(GOAL - 1));

    assert_eq!(
        stored_pool(&env, &contract_id, pool_id).state,
        PoolState::Active
    );
}

// ── 4: terminal states ───────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_closed_pool_rejects_donations() {
    let env = Env::default();
    let (_contract_id, client, pool_id) = setup(&env);
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    client.close_pool(&pool_id);

    client.donate(&pool_id, &Address::generate(&env), &1u128);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_closed_pool_cannot_be_closed_again() {
    let env = Env::default();
    let (_contract_id, client, pool_id) = setup(&env);
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    client.close_pool(&pool_id);

    client.close_pool(&pool_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_completed_pool_rejects_further_donations() {
    let env = Env::default();
    let (_contract_id, client, pool_id) = setup(&env);
    client.donate(&pool_id, &Address::generate(&env), &GOAL);

    client.donate(&pool_id, &Address::generate(&env), &1u128);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_active_pool_cannot_be_closed() {
    let env = Env::default();
    let (_contract_id, client, pool_id) = setup(&env);

    client.close_pool(&pool_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_completed_pool_cannot_be_closed_before_disbursement() {
    let env = Env::default();
    let (_contract_id, client, pool_id) = setup(&env);
    client.donate(&pool_id, &Address::generate(&env), &GOAL);

    client.close_pool(&pool_id);
}

#[test]
fn test_non_active_states_reject_donations() {
    for state in [
        PoolState::Paused,
        PoolState::Completed,
        PoolState::Cancelled,
        PoolState::Disbursed,
    ] {
        let env = Env::default();
        let (_contract_id, client, pool_id) = setup(&env);
        client.set_pool_state(&pool_id, &state);

        let result = client.try_donate(&pool_id, &Address::generate(&env), &1u128);
        assert_eq!(
            result,
            Err(Ok(ContractError::InvalidPoolState)),
            "{:?} pool must reject donations",
            state
        );
    }
}

// ── 5: get_pool consistency ──────────────────────────────────────────────────

#[test]
fn test_get_pool_matches_storage_in_every_state() {
    for target in [
        PoolState::Active,
        PoolState::Paused,
        PoolState::Completed,
        PoolState::Cancelled,
        PoolState::Disbursed,
        PoolState::Closed,
    ] {
        let env = Env::default();
        let (contract_id, client, pool_id) = setup(&env);
        match target {
            PoolState::Active => {}
            PoolState::Completed => client.donate(&pool_id, &Address::generate(&env), &GOAL),
            PoolState::Closed => {
                client.set_pool_state(&pool_id, &PoolState::Cancelled);
                client.close_pool(&pool_id);
            }
            _ => client.set_pool_state(&pool_id, &target),
        }

        let stored = stored_pool(&env, &contract_id, pool_id);
        let (id, sponsor, goal, collected, is_closed, deadline) = client.get_pool(&pool_id);

        assert_eq!(stored.state, target);
        assert_eq!(id, pool_id);
        assert_eq!(sponsor, stored.sponsor);
        assert_eq!(goal, stored.goal);
        assert_eq!(collected, stored.collected);
        assert_eq!(deadline, stored.application_deadline);
        assert_eq!(is_closed, stored.is_closed);
        assert_eq!(
            is_closed,
            stored.state == PoolState::Closed,
            "get_pool's is_closed must agree with the stored {:?} state",
            stored.state
        );
    }
}
