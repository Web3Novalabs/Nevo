#![cfg(test)]
//! Tests for `donate_with_token` multi-token support — issue #1349.
//!
//! `donate_with_token` is the token-backed path: it requires donor auth, moves
//! tokens from the donor into the contract, and records the same accounting as
//! `donate`. A pool can have a token bound to it with `set_pool_token`, and from
//! then on a different token is rejected.
//!
//! Scenarios:
//!   1. A supported non-native token is accepted and recorded.
//!   2. A token other than the one bound to the pool is rejected
//!      (`TokenTransferFailed`).
//!   3. An address that is not a token contract at all is rejected, whether or
//!      not a token is bound to the pool.
//!   4. The transfer moves tokens from the donor to the contract — both sides
//!      are checked, not just the pool total.
//!   5. The pool total is the token-denominated amount, and native (`donate`)
//!      plus token donations add up consistently, with the donor counted once
//!      across both paths.

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::Client as TokenClient, token::StellarAssetClient, Address, Env,
    String,
};

const GOAL: u128 = 1_000_000_000;

fn make_pool(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Multi Token"),
        &String::from_str(env, "donate_with_token tests"),
        &GOAL,
        &100_000u64,
    )
}

/// A Stellar Asset Contract with `amount` minted to `recipient`.
fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(env, &token.address()).mint(recipient, &amount);
    token.address()
}

fn setup(env: &Env) -> (ContractClient<'_>, u32) {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let pool_id = make_pool(env, &client);
    (client, pool_id)
}

// ── 1. A supported non-native token is accepted ───────────────────────────

/// A donation in a bound SEP-41 token is accepted and recorded.
#[test]
fn test_donate_with_token_accepts_bound_token() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &donor);

    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &donor, &token, &400i128);

    assert_eq!(client.get_total_raised(&pool_id), 400u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);
}

// ── 2. Wrong token for the pool ───────────────────────────────────────────

/// Once a token is bound to a pool, a donation in a different token is
/// rejected rather than silently mixed into the same total.
#[test]
#[should_panic(expected = "TokenTransferFailed")]
fn test_donate_with_token_rejects_unbound_token() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);
    let bound = create_token(&env, 1_000i128, &donor);
    let other = create_token(&env, 1_000i128, &donor);

    client.set_pool_token(&pool_id, &bound);
    client.donate_with_token(&pool_id, &donor, &other, &100i128);
}

// ── 3. Not a token at all ─────────────────────────────────────────────────

/// An address that is not a token contract cannot be donated through: the
/// transfer call fails, and the pool total is untouched afterwards.
#[test]
fn test_donate_with_token_rejects_non_token_address() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);
    let not_a_token = Address::generate(&env);

    // No token bound to the pool yet, so the guard does not fire — the failure
    // has to come from the token client itself.
    let result = client.try_donate_with_token(&pool_id, &donor, &not_a_token, &100i128);

    assert!(
        result.is_err(),
        "a plain address must not be accepted as a token contract"
    );
    assert_eq!(
        client.get_total_raised(&pool_id),
        0u128,
        "a failed transfer must not be recorded as a donation"
    );
    assert_eq!(client.get_donor_count(&pool_id), 0u32);
}

// ── 4. Both sides of the transfer ─────────────────────────────────────────

/// The donation moves tokens out of the donor and into the contract, so both
/// balances are asserted rather than just the pool's accounting.
#[test]
fn test_donate_with_token_moves_tokens_both_ways() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &donor);

    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &donor, &token, &250i128);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&donor), 750i128);
    assert_eq!(
        token_client.balance(&client.address),
        250i128,
        "the contract must hold the donated tokens"
    );
    assert_eq!(client.get_total_raised(&pool_id), 250u128);
}

// ── 5. Native plus token donations ────────────────────────────────────────

/// The total is the token-denominated amount, and native (`donate`) and
/// token-backed contributions add up in the same running total. The donor is
/// counted once even though two different entry points recorded them.
#[test]
fn test_native_and_token_donations_accumulate_consistently() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &donor);

    client.set_pool_token(&pool_id, &token);

    client.donate(&pool_id, &donor, &500u128);
    assert_eq!(client.get_total_raised(&pool_id), 500u128);

    client.donate_with_token(&pool_id, &donor, &token, &300i128);
    assert_eq!(
        client.get_total_raised(&pool_id),
        800u128,
        "token donations must add to the same total as native ones"
    );
    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "the same donor must not be counted twice across the two entry points"
    );

    let other = Address::generate(&env);
    let second_token = create_token(&env, 1_000i128, &other);
    client.set_pool_token(&pool_id, &second_token);
    client.donate_with_token(&pool_id, &other, &second_token, &100i128);

    assert_eq!(client.get_total_raised(&pool_id), 900u128);
    assert_eq!(client.get_donor_count(&pool_id), 2u32);
}
