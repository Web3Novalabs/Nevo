#![cfg(test)]
//! Tests for duplicate donor donation aggregation — issue #1368.
//!
//! Scenarios:
//!   1. The same donor donating multiple times aggregates in `get_contribution`.
//!   2. `get_donor_count` does not double-count repeat donors.
//!   3. `get_total_raised` sums every donation regardless of donor repetition.
//!   4. Aggregation holds when one donor mixes `donate` and `donate_with_token`.
//!   5. Refunding a multi-donation donor zeroes their full aggregated contribution.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

// REFUND_GRACE_PERIOD_LEDGERS = 17_280 (from lib.rs)
const GRACE: u32 = 17_280;
const LARGE_TTL: u32 = 10_000_000;
// High enough that no test donation completes the pool.
const GOAL: u128 = 1_000_000_000_000;

fn setup(env: &Env) -> (ContractClient<'_>, u32) {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let pool_id = client.create_pool(
        &Address::generate(env),
        &String::from_str(env, "Repeat Donor Pool"),
        &String::from_str(env, "Testing repeat donor aggregation"),
        &GOAL,
        &2_000_000u64,
    );
    (client, pool_id)
}

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let token = env.register_stellar_asset_contract_v2(Address::generate(env));
    StellarAssetClient::new(env, &token.address()).mint(recipient, &amount);
    token.address()
}

fn set_ledger_sequence(env: &Env, seq: u32) {
    env.ledger().set(LedgerInfo {
        sequence_number: seq,
        timestamp: env.ledger().timestamp(),
        protocol_version: env.ledger().protocol_version(),
        network_id: Default::default(),
        base_reserve: 5_000_000,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: LARGE_TTL,
        max_entry_ttl: LARGE_TTL,
    });
}

// ── 1: contribution aggregates ───────────────────────────────────────────────

#[test]
fn test_repeat_donations_aggregate_in_get_contribution() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);

    for amount in [100u128, 250, 650] {
        client.donate(&pool_id, &donor, &amount);
    }

    assert_eq!(client.get_contribution(&pool_id, &donor), 1_000);
}

#[test]
fn test_repeat_donations_do_not_leak_into_other_donors() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let repeat = Address::generate(&env);
    let single = Address::generate(&env);

    client.donate(&pool_id, &repeat, &100u128);
    client.donate(&pool_id, &single, &40u128);
    client.donate(&pool_id, &repeat, &200u128);

    assert_eq!(client.get_contribution(&pool_id, &repeat), 300);
    assert_eq!(client.get_contribution(&pool_id, &single), 40);
}

// ── 2: donor count ───────────────────────────────────────────────────────────

#[test]
fn test_repeat_donor_is_counted_once() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);

    for _ in 0..5 {
        client.donate(&pool_id, &donor, &10u128);
    }

    assert_eq!(client.get_donor_count(&pool_id), 1);
}

#[test]
fn test_donor_count_counts_each_distinct_repeat_donor_once() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    for _ in 0..3 {
        client.donate(&pool_id, &alice, &10u128);
        client.donate(&pool_id, &bob, &20u128);
    }

    assert_eq!(client.get_donor_count(&pool_id), 2);
}

// ── 3: total raised ──────────────────────────────────────────────────────────

#[test]
fn test_total_raised_sums_all_donations_regardless_of_repetition() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.donate(&pool_id, &alice, &100u128);
    client.donate(&pool_id, &alice, &100u128);
    client.donate(&pool_id, &bob, &50u128);
    client.donate(&pool_id, &alice, &25u128);

    assert_eq!(client.get_total_raised(&pool_id), 275);
    assert_eq!(
        client.get_total_raised(&pool_id),
        client.get_contribution(&pool_id, &alice) + client.get_contribution(&pool_id, &bob)
    );
}

// ── 4: native + token donations from the same donor ──────────────────────────

#[test]
fn test_mixed_donate_and_donate_with_token_aggregate() {
    let env = Env::default();
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000, &donor);

    client.donate(&pool_id, &donor, &300u128);
    client.donate_with_token(&pool_id, &donor, &token, &200i128);
    client.donate(&pool_id, &donor, &100u128);
    client.donate_with_token(&pool_id, &donor, &token, &400i128);

    assert_eq!(client.get_contribution(&pool_id, &donor), 1_000);
    assert_eq!(client.get_donor_count(&pool_id), 1);
    assert_eq!(client.get_total_raised(&pool_id), 1_000);
}

// ── 5: refund zeroes the full aggregate ──────────────────────────────────────

#[test]
fn test_refund_zeroes_full_aggregated_contribution() {
    let env = Env::default();
    set_ledger_sequence(&env, 100);
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);
    let other = Address::generate(&env);
    let token = create_token(&env, 1_000, &donor);
    StellarAssetClient::new(&env, &token).mint(&other, &500);

    client.donate_with_token(&pool_id, &donor, &token, &300i128);
    client.donate_with_token(&pool_id, &other, &token, &500i128);
    client.donate_with_token(&pool_id, &donor, &token, &200i128);
    client.donate_with_token(&pool_id, &donor, &token, &500i128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 1_000);

    client.set_pool_deadline(&pool_id, &1_000u32);
    set_ledger_sequence(&env, 1_000 + GRACE);
    client.refund_donation(&pool_id, &donor, &token);

    assert_eq!(client.get_contribution(&pool_id, &donor), 0);
    // Only the other donor's contribution remains.
    assert_eq!(client.get_total_raised(&pool_id), 500);
    assert_eq!(client.get_contribution(&pool_id, &other), 500);
    // The whole aggregate went back to the donor in one transfer.
    assert_eq!(TokenClient::new(&env, &token).balance(&donor), 1_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_second_refund_after_aggregate_refund_fails() {
    let env = Env::default();
    set_ledger_sequence(&env, 100);
    let (client, pool_id) = setup(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000, &donor);

    client.donate_with_token(&pool_id, &donor, &token, &400i128);
    client.donate_with_token(&pool_id, &donor, &token, &600i128);

    client.set_pool_deadline(&pool_id, &1_000u32);
    set_ledger_sequence(&env, 1_000 + GRACE);
    client.refund_donation(&pool_id, &donor, &token);

    // Nothing is left to refund.
    client.refund_donation(&pool_id, &donor, &token);
}
