#![cfg(test)]
//! Bounds behaviour of `get_application_by_index` against the per-pool
//! application counter — issue #1472.
//!
//! `apply_to_pool` increments the counter before storing, so applications
//! are 1-indexed and keyed by `(prefix, pool_id, index)`.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn create_pool(env: &Env, client: &ContractClient<'_>) -> u32 {
    client.create_pool(
        &Address::generate(env),
        &String::from_str(env, "Application Pool"),
        &String::from_str(env, "Description"),
        &1_000_000u128,
        &200_000u64,
    )
}

fn apply(env: &Env, client: &ContractClient<'_>, pool_id: u32, data: &str) -> Address {
    let student = Address::generate(env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(env, data));
    student
}

#[test]
fn test_index_zero_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ContractClient::new(&env, &env.register(Contract, ()));
    let pool_id = create_pool(&env, &client);
    let student = apply(&env, &client, pool_id, "first");

    assert_eq!(client.get_application_by_index(&pool_id, &0), None);
    assert_eq!(
        client.get_application_by_index(&pool_id, &1),
        Some((1, student, String::from_str(&env, "first")))
    );
}

#[test]
fn test_index_above_app_count_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ContractClient::new(&env, &env.register(Contract, ()));
    let pool_id = create_pool(&env, &client);
    apply(&env, &client, pool_id, "first");
    apply(&env, &client, pool_id, "second");

    assert!(client.get_application_by_index(&pool_id, &2).is_some());
    assert_eq!(client.get_application_by_index(&pool_id, &3), None);
    assert_eq!(client.get_application_by_index(&pool_id, &u32::MAX), None);
}

#[test]
fn test_indexes_do_not_collide_across_pools() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ContractClient::new(&env, &env.register(Contract, ()));
    let pool_a = create_pool(&env, &client);
    let pool_b = create_pool(&env, &client);

    let a1 = apply(&env, &client, pool_a, "a1");
    let a2 = apply(&env, &client, pool_a, "a2");
    let b1 = apply(&env, &client, pool_b, "b1");

    assert_eq!(
        client.get_application_by_index(&pool_a, &1),
        Some((1, a1, String::from_str(&env, "a1")))
    );
    assert_eq!(
        client.get_application_by_index(&pool_a, &2),
        Some((2, a2, String::from_str(&env, "a2")))
    );
    assert_eq!(
        client.get_application_by_index(&pool_b, &1),
        Some((1, b1, String::from_str(&env, "b1")))
    );
    // Index 2 is valid in pool A's range but does not exist in pool B.
    assert_eq!(client.get_application_by_index(&pool_b, &2), None);
}
