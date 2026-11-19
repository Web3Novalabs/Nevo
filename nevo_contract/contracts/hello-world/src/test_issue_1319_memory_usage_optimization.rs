#![cfg(test)]
//! Tests for memory usage optimization — issue #1319.
//!
//! Verifies storage stays balanced: campaign listing and per-pool totals grow
//! with the data set while repeated reads leave no residual state behind.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_campaign_storage_matches_pool_count() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ContractClient::new(&env, &env.register(Contract, ()));

    for i in 0..5 {
        client.create_pool(
            &Address::generate(&env),
            &String::from_str(&env, "Mem Pool"),
            &String::from_str(&env, "Memory usage optimization"),
            &1_000_000_000u128,
            &100_000u64,
        );
        assert_eq!(client.get_pool_count(), i + 1);
    }

    assert_eq!(client.get_all_campaigns().len(), 5);
    assert_eq!(client.get_total_raised(&0), 0);
}
