#![cfg(test)]
//! Shows that `get_pool` never exposes `Pool.state` — issue #1470.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_pool(env: &Env) -> (Address, ContractClient<'_>, u32) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let pool_id = client.create_pool(
        &Address::generate(env),
        &String::from_str(env, "State Pool"),
        &String::from_str(env, "Description"),
        &1_000_000u128,
        &200_000u64,
    );
    (contract_id, client, pool_id)
}

fn stored_state(env: &Env, contract_id: &Address, pool_id: u32) -> PoolState {
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap()
            .state
    })
}

/// Every `PoolState` yields the same `get_pool` tuple, so the state cannot
/// be derived from `get_pool` alone.
#[test]
fn test_get_pool_tuple_is_identical_across_all_pool_states() {
    let env = Env::default();
    let (contract_id, client, pool_id) = setup_pool(&env);
    let baseline = client.get_pool(&pool_id);

    for state in [
        PoolState::Active,
        PoolState::Paused,
        PoolState::Completed,
        PoolState::Cancelled,
        PoolState::Disbursed,
    ] {
        client.set_pool_state(&pool_id, &state);
        assert_eq!(stored_state(&env, &contract_id, pool_id), state);
        assert_eq!(client.get_pool(&pool_id), baseline);
    }
}

/// `close_pool` only flips `is_closed` and never touches `state`, so the two
/// can disagree (closed but `Active`), and `get_pool` surfaces only
/// `is_closed`.
#[test]
fn test_is_closed_and_state_can_disagree_and_get_pool_shows_only_is_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let (contract_id, client, pool_id) = setup_pool(&env);

    // close_pool requires a Cancelled or Disbursed pool and leaves state as-is.
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    client.close_pool(&pool_id);
    assert_eq!(
        stored_state(&env, &contract_id, pool_id),
        PoolState::Cancelled
    );

    client.set_pool_state(&pool_id, &PoolState::Active);

    let (_, _, _, _, is_closed, _) = client.get_pool(&pool_id);
    assert!(is_closed);
    assert_eq!(stored_state(&env, &contract_id, pool_id), PoolState::Active);
}
