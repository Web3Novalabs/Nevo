#![cfg(test)]
//! Tests for donor count tracking accuracy — issue #1271.
//!
//! Covers:
//!   1. New campaign has 0 donors.
//!   2. First donation increments to 1.
//!   3. Same donor's multiple donations keeps count at 1.
//!   4. Different donors increment count correctly.
//!   5. Nonexistent campaign returns error.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn create_campaign(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Donor Tracking Pool"),
        &String::from_str(env, "Validating unique donor count metrics"),
        &10_000_000_000u128,
        &100_000u64,
    )
}

/// (1) New campaign has 0 donors.
#[test]
fn test_new_campaign_has_zero_donors() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = create_campaign(&env, &client);

    assert_eq!(
        client.get_donor_count(&pool_id),
        0u32,
        "New campaign must report 0 donors"
    );
}

/// (2) First donation increments to 1.
#[test]
fn test_first_donation_increments_donor_count_to_one() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = create_campaign(&env, &client);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &100_000_000u128);

    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "First donation must increment donor count to 1"
    );
}

/// (3) Same donor's multiple donations keeps count at 1.
#[test]
fn test_same_donor_multiple_donations_keeps_count_at_one() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = create_campaign(&env, &client);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &100_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);

    client.donate(&pool_id, &donor, &50_000_000u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "Second donation from same donor must keep donor count at 1"
    );

    client.donate(&pool_id, &donor, &25_000_000u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "Third donation from same donor must keep donor count at 1"
    );
}

/// (4) Different donors increment count correctly.
#[test]
fn test_different_donors_increment_count_correctly() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = create_campaign(&env, &client);

    let donor_1 = Address::generate(&env);
    let donor_2 = Address::generate(&env);
    let donor_3 = Address::generate(&env);

    client.donate(&pool_id, &donor_1, &100_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);

    client.donate(&pool_id, &donor_2, &200_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 2u32);

    client.donate(&pool_id, &donor_3, &300_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 3u32);

    // Repeat donation from donor_2 does not increment count
    client.donate(&pool_id, &donor_2, &50_000_000u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        3u32,
        "Repeat donation must not increase unique donor count"
    );
}

/// (5) Nonexistent campaign returns error.
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_nonexistent_campaign_donor_count_returns_error() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.get_donor_count(&9999u32);
}

#[test]
fn test_nonexistent_campaign_donor_count_try_returns_error() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let res = client.try_get_donor_count(&9999u32);
    assert_eq!(
        res,
        Err(Ok(ContractError::PoolNotFound)),
        "Nonexistent campaign must return PoolNotFound error"
    );
}
