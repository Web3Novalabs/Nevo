//! Tests for pool contribution metrics tracking — issue #1267
//!
//! Covers:
//!   1. First contribution increments contributor_count to 1.
//!   2. Same contributor's second contribution keeps count at 1.
//!   3. Different contributor increments count to 2.
//!   4. Total_raised updates correctly with multiple contributions.
//!   5. Last_donation_at timestamp updates properly.

#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// (1) First contribution increments contributor_count to 1.
#[test]
fn test_first_contribution_increments_contributor_count_to_one() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool 1"),
        &String::from_str(&env, "Testing first contribution metric"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Initial contributor count must be 0
    assert_eq!(
        client.get_donor_count(&pool_id),
        0u32,
        "A freshly created pool must have 0 contributors"
    );

    // First contribution made
    client.donate(&pool_id, &donor, &50_000_000u128);

    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "First contribution must increment contributor_count to 1"
    );
}

/// (2) Same contributor's second contribution keeps count at 1.
#[test]
fn test_same_contributor_second_contribution_keeps_count_at_one() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool 2"),
        &String::from_str(&env, "Testing repeat contribution metric"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor, &20_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);

    // Same contributor contributes a second time
    client.donate(&pool_id, &donor, &30_000_000u128);

    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "Second contribution by same contributor must keep contributor_count at 1"
    );

    // Also verify individual contribution accumulated correctly
    assert_eq!(
        client.get_contribution(&pool_id, &donor),
        50_000_000u128,
        "Individual contribution should equal sum of donations"
    );
}

/// (3) Different contributor increments count to 2.
#[test]
fn test_different_contributor_increments_count_to_two() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_1 = Address::generate(&env);
    let donor_2 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool 3"),
        &String::from_str(&env, "Testing multiple contributor count"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor_1, &40_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);

    // A different contributor donates
    client.donate(&pool_id, &donor_2, &60_000_000u128);

    assert_eq!(
        client.get_donor_count(&pool_id),
        2u32,
        "Different contributor contribution must increment contributor_count to 2"
    );
}

/// (4) Total_raised updates correctly with multiple contributions.
#[test]
fn test_total_raised_updates_correctly_with_multiple_contributions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_1 = Address::generate(&env);
    let donor_2 = Address::generate(&env);
    let donor_3 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Total Raised Pool"),
        &String::from_str(&env, "Testing total raised calculation"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(
        client.get_total_raised(&pool_id),
        0u128,
        "Fresh pool must start with 0 total raised"
    );

    // Contribution 1
    let amount_1 = 100_000_000u128;
    client.donate(&pool_id, &donor_1, &amount_1);
    assert_eq!(client.get_total_raised(&pool_id), amount_1);

    // Contribution 2 (same donor)
    let amount_2 = 150_000_000u128;
    client.donate(&pool_id, &donor_1, &amount_2);
    assert_eq!(client.get_total_raised(&pool_id), amount_1 + amount_2);

    // Contribution 3 (second donor)
    let amount_3 = 75_000_000u128;
    client.donate(&pool_id, &donor_2, &amount_3);
    assert_eq!(
        client.get_total_raised(&pool_id),
        amount_1 + amount_2 + amount_3
    );

    // Contribution 4 (third donor)
    let amount_4 = 25_000_000u128;
    client.donate(&pool_id, &donor_3, &amount_4);
    let expected_total = amount_1 + amount_2 + amount_3 + amount_4;
    assert_eq!(client.get_total_raised(&pool_id), expected_total);

    // Cross-verify with get_pool().3 (collected)
    let pool = client.get_pool(&pool_id);
    assert_eq!(
        pool.3, expected_total,
        "Pool collected balance must match get_total_raised"
    );
}

/// (5) Last_donation_at timestamp updates properly.
#[test]
fn test_last_donation_at_timestamp_updates_properly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_1 = Address::generate(&env);
    let donor_2 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Timestamp Pool"),
        &String::from_str(&env, "Testing last_donation_at"),
        &1_000_000_000u128,
        &200_000u64,
    );

    // Initial last_donation_at must be 0
    assert_eq!(
        client.get_last_donation_at(&pool_id),
        0u64,
        "A pool with no donations must have last_donation_at == 0"
    );

    // First donation at timestamp 10_000
    let ts_1 = 10_000u64;
    env.ledger().set_timestamp(ts_1);
    client.donate(&pool_id, &donor_1, &100_000_000u128);

    assert_eq!(
        client.get_last_donation_at(&pool_id),
        ts_1,
        "last_donation_at must match timestamp of first donation"
    );

    // Second donation at timestamp 25_000
    let ts_2 = 25_000u64;
    env.ledger().set_timestamp(ts_2);
    client.donate(&pool_id, &donor_2, &50_000_000u128);

    assert_eq!(
        client.get_last_donation_at(&pool_id),
        ts_2,
        "last_donation_at must update to the new ledger timestamp"
    );

    // Third donation at timestamp 40_000 (same donor_1)
    let ts_3 = 40_000u64;
    env.ledger().set_timestamp(ts_3);
    client.donate(&pool_id, &donor_1, &20_000_000u128);

    assert_eq!(
        client.get_last_donation_at(&pool_id),
        ts_3,
        "last_donation_at must update on repeat donation by existing contributor"
    );
}

/// Token-based contributions (`donate_with_token`) also update all metrics properly.
#[test]
fn test_metrics_tracked_consistently_via_token_donations() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_a = Address::generate(&env);
    let donor_b = Address::generate(&env);

    let amount_a: i128 = 200_000_000;
    let amount_b: i128 = 300_000_000;
    let token = create_token(&env, amount_a + amount_b, &donor_a);

    // Also mint to donor_b
    let sac = StellarAssetClient::new(&env, &token);
    sac.mint(&donor_b, &amount_b);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Token Metrics Pool"),
        &String::from_str(&env, "Testing metrics with token donations"),
        &1_000_000_000u128,
        &200_000u64,
    );

    let t1 = 5_000u64;
    env.ledger().set_timestamp(t1);
    client.donate_with_token(&pool_id, &donor_a, &token, &100_000_000i128);

    assert_eq!(client.get_donor_count(&pool_id), 1u32);
    assert_eq!(client.get_total_raised(&pool_id), 100_000_000u128);
    assert_eq!(client.get_last_donation_at(&pool_id), t1);

    // Repeat contribution by donor_a
    let t2 = 8_000u64;
    env.ledger().set_timestamp(t2);
    client.donate_with_token(&pool_id, &donor_a, &token, &100_000_000i128);

    assert_eq!(client.get_donor_count(&pool_id), 1u32);
    assert_eq!(client.get_total_raised(&pool_id), 200_000_000u128);
    assert_eq!(client.get_last_donation_at(&pool_id), t2);

    // Contribution by donor_b
    let t3 = 12_000u64;
    env.ledger().set_timestamp(t3);
    client.donate_with_token(&pool_id, &donor_b, &token, &amount_b);

    assert_eq!(client.get_donor_count(&pool_id), 2u32);
    assert_eq!(client.get_total_raised(&pool_id), 500_000_000u128);
    assert_eq!(client.get_last_donation_at(&pool_id), t3);
}
