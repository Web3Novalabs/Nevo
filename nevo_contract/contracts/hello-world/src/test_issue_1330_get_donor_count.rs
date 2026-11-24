#![cfg(test)]
//! Issue #1330: `get_donor_count` accuracy across repeated donations.
//!
//! Scenarios:
//!   1. Returns 0 for a newly created pool.
//!   2. Increments on the first donation from a new donor.
//!   3. Does not increment on a repeat donation from an existing donor.
//!   4. Reports the correct count for many distinct donors.
//!   5. Counts are isolated per pool_id.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn create_pool(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Donor Count Pool"),
        &String::from_str(env, "Measures unique donor aggregation"),
        &10_000_000_000u128,
        &100_000u64,
    )
}

/// (1): a brand-new pool reports zero donors.
#[test]
fn test_new_pool_has_zero_donors() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = create_pool(&env, &client);
    assert_eq!(client.get_donor_count(&pool_id), 0u32);
}

/// (2): the first donation from a donor increments the count to one.
#[test]
fn test_first_donation_increments_donor_count() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = create_pool(&env, &client);
    let donor = Address::generate(&env);
    client.donate(&pool_id, &donor, &100_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);
}

/// (3): repeat donations from the same donor do not increment the count.
#[test]
fn test_repeat_donation_does_not_increment_donor_count() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = create_pool(&env, &client);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &100_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);

    client.donate(&pool_id, &donor, &50_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);

    client.donate(&pool_id, &donor, &25_000_000u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "repeat donations from the same donor must not increment the count"
    );
}

/// (4): many distinct donors produce the matching count.
#[test]
fn test_donor_count_with_many_distinct_donors() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = create_pool(&env, &client);
    let distinct_donors = 10u32;
    for _ in 0..distinct_donors {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &100_000_000u128);
    }
    assert_eq!(client.get_donor_count(&pool_id), distinct_donors);
}

/// (5): donor counts are tracked independently per pool. The same address
/// donating to two pools is counted once in each, and one pool's count never
/// leaks into another.
#[test]
fn test_donor_count_isolated_per_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_a = create_pool(&env, &client);
    let pool_b = create_pool(&env, &client);

    let donor_a1 = Address::generate(&env);
    let donor_a2 = Address::generate(&env);
    let donor_b1 = Address::generate(&env);

    // Pool A receives two distinct donors.
    client.donate(&pool_a, &donor_a1, &100_000_000u128);
    client.donate(&pool_a, &donor_a2, &100_000_000u128);
    assert_eq!(client.get_donor_count(&pool_a), 2u32);
    assert_eq!(client.get_donor_count(&pool_b), 0u32);

    // Pool B receives one donor; pool A's count is unaffected.
    client.donate(&pool_b, &donor_b1, &100_000_000u128);
    assert_eq!(client.get_donor_count(&pool_b), 1u32);
    assert_eq!(client.get_donor_count(&pool_a), 2u32);

    // The same donor donating to pool B is a new donor for pool B and must not
    // change pool A's count.
    client.donate(&pool_b, &donor_a1, &100_000_000u128);
    assert_eq!(client.get_donor_count(&pool_b), 2u32);
    assert_eq!(client.get_donor_count(&pool_a), 2u32);
}
