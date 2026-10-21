#![cfg(test)]
//! Tests for documentation accuracy — issue #1323.
//!
//! Verifies documented behaviour: `create_pool` documents the returned pool id
//! and `get_pool` the (id, creator, goal, raised, is_closed, deadline) tuple.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_documented_get_pool_tuple_shape() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ContractClient::new(&env, &env.register(Contract, ()));

    let pool_id = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Docs Pool"),
        &String::from_str(&env, "Documentation accuracy"),
        &1_000_000_000u128,
        &100_000u64,
    );
    let (id, _creator, goal, _raised, is_closed, _deadline) = client.get_pool(&pool_id);

    assert_eq!(id, pool_id);
    assert_eq!(goal, 1_000_000_000u128);
    assert!(!is_closed);
}
