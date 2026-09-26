#![cfg(test)]

//! Issue #944: token transfer failure paths against the real `Contract` API.
//!
//! These cover what happens when a transfer cannot be satisfied — insufficient
//! donor balance, an empty contract balance, no surplus, no accrued fees — and
//! assert that a failed transfer leaves contract state untouched.

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// A failed transfer must not move the pool's accounting forward.
#[test]
fn test_failed_transfer_does_not_corrupt_pool_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let collected_before = client.get_pool(&pool_id).3;

    // `try_*` returns the error instead of unwinding, so state can be inspected.
    let result = client.try_donate_with_token(&pool_id, &donor, &token, &1_000i128);
    assert!(result.is_err(), "Overdrawn donation must fail");

    assert_eq!(
        client.get_pool(&pool_id).3,
        collected_before,
        "Pool collected total must be unchanged after a failed transfer"
    );
    assert_eq!(
        client.get_contribution(&pool_id, &donor),
        0u128,
        "Donor contribution must not be recorded for a failed transfer"
    );
    assert_eq!(
        token::Client::new(&env, &token).balance(&donor),
        100i128,
        "Donor balance must be intact after a failed transfer"
    );
}

/// A zero or negative donation is rejected before any transfer is attempted.
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_negative_donation_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.donate_with_token(&pool_id, &donor, &token, &-1i128);
}

/// Claiming fees when none have accrued fails with `NoUnclaimedFees`.
#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_claim_protocol_fees_with_no_accrued_fees_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &contract_id);

    client.set_admin(&admin);
    client.claim_protocol_fees(&admin, &token);
}

/// Withdrawing unallocated funds from an empty pool fails with no surplus.
#[test]
#[should_panic(expected = "No surplus to withdraw")]
fn test_withdraw_unallocated_funds_with_no_surplus_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Empty Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.withdraw_unallocated_funds(&pool_id, &token);
}

/// A student cannot claim more than the pool has collected.
#[test]
#[should_panic(expected = "Overdraw attempt")]
fn test_claim_beyond_collected_funds_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let student = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.donate_with_token(&pool_id, &donor, &token, &1_000i128);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Pool holds 1_000; claiming 5_000 must be rejected.
    client.claim_funds(&student, &pool_id, &5_000i128, &token);
}
