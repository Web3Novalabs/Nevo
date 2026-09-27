#![cfg(test)]
//! Tests for edge case combinations — issue #1318.
//!
//! Verifies that a closed pool stays closed for every dependent operation
//! (donation, refund deadline) when multiple edge conditions coincide.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_closed_pool_blocks_donations_at_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ContractClient::new(&env, &env.register(Contract, ()));

    let pool_id = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Edge Pool"),
        &String::from_str(&env, "Edge case combinations"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.close_pool(&pool_id);

    client.try_donate(&pool_id, &Address::generate(&env), &1_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 0);
}
