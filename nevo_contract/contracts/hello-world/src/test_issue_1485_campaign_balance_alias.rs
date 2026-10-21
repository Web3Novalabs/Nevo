#![cfg(test)]
//! Regression test for the `get_campaign_balance` / `get_total_raised` pair —
//! issue #1485.
//!
//! `get_campaign_balance` is a thin alias for `get_total_raised`. The two are
//! tested separately today, so nothing would catch one of them drifting away
//! from the other. This test calls both getters on the same pool after several
//! donations — native (`donate`) and token-backed (`donate_with_token`) — and
//! requires identical results.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Register a Stellar asset contract, mint `amount` to `holder`, and return the
/// token address.
fn create_token(env: &Env, amount: i128, holder: &Address) -> Address {
    let asset_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(asset_admin);
    soroban_sdk::token::StellarAssetClient::new(env, &token.address()).mint(holder, &amount);
    token.address()
}

/// After a mix of native and token-backed donations, both getters must report
/// the same total, and that total must be the sum of the donations.
#[test]
fn test_campaign_balance_matches_total_raised_after_donations() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Alias Drift Pool"),
        &String::from_str(&env, "Pool used to compare the two balance getters"),
        &100_000_000_000u128,
        &100_000u64,
    );

    let native_amounts: [u128; 2] = [100_000_000u128, 250_000_000u128];
    let token_donor = Address::generate(&env);
    let token_amount: i128 = 75_000_000;
    let donation_token = create_token(&env, token_amount * 2, &token_donor);

    let mut expected_total: u128 = 0;

    for amount in native_amounts {
        client.donate(&pool_id, &Address::generate(&env), &amount);
        expected_total += amount;
    }

    // A second token-backed donation from the same donor, so the donation path
    // has to accumulate rather than overwrite.
    client.donate_with_token(&pool_id, &token_donor, &donation_token, &token_amount);
    expected_total += token_amount as u128;
    client.donate_with_token(&pool_id, &token_donor, &donation_token, &token_amount);
    expected_total += token_amount as u128;

    assert!(
        expected_total > 0,
        "the pool must hold donations, otherwise the comparison is trivial"
    );
    assert_eq!(client.get_donor_count(&pool_id), 3u32);

    let campaign_balance = client.get_campaign_balance(&pool_id);
    let total_raised = client.get_total_raised(&pool_id);

    assert_eq!(
        campaign_balance, total_raised,
        "get_campaign_balance ({}) and get_total_raised ({}) must not drift apart",
        campaign_balance, total_raised
    );
    assert_eq!(
        campaign_balance, expected_total,
        "both getters must report the sum of every donation"
    );
    assert_eq!(
        client.get_campaign_goal(&pool_id),
        100_000_000_000u128,
        "the goal must be unaffected by donations"
    );
}
