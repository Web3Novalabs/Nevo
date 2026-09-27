#![cfg(test)]
//! Tests for concurrent operation safety — issue #1317.
//!
//! Verifies atomic donation updates: sequential donations from several donors
//! accumulate without lost updates, and donor count stays in sync.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_sequential_donations_are_atomic() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ContractClient::new(&env, &env.register(Contract, ()));

    let pool_id = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Concurrent Pool"),
        &String::from_str(&env, "Concurrent operation safety"),
        &1_000_000_000u128,
        &100_000u64,
    );
    for _ in 0..3 {
        client.donate(&pool_id, &Address::generate(&env), &1_000u128);
    }

    assert_eq!(client.get_pool(&pool_id).3, 3_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 3);
}
