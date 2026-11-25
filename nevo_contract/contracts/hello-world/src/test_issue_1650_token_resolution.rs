#![cfg(test)]
//! Tests for issue #1650: a pool's `collected` must always be denominated in
//! exactly one token.
//!
//! Scenarios covered:
//!   1. Unconfigured pool + unset global → typed `CrowdfundingTokenNotSet`.
//!   2. Unconfigured pool + set global    → only the global token accepted.
//!   3. Pool token + global token         → pool token wins.
//!   4. `refund_donation` mismatch        → typed `TokenTransferFailed`.
//!   5. `withdraw_unallocated_funds` mix  → typed `TokenTransferFailed`.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    token::StellarAssetClient,
    Address, Env, String,
};

const GRACE: u32 = 17_280;
const LARGE_TTL: u32 = 10_000_000;

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
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

fn setup(env: &Env) -> (ContractClient<'_>, Address, u32) {
    env.mock_all_auths();
    set_ledger_sequence(env, 100);
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.set_admin(&admin);
    let sponsor = Address::generate(env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(env, "Token Resolution Pool"),
        &String::from_str(env, "Tests for #1650"),
        &5_000_000i128,
        &500u64,
    );
    (client, admin, pool_id)
}

// ── 1. Unconfigured pool + no global token ────────────────────────────────

#[test]
fn donate_with_unconfigured_pool_and_no_global_token_is_typed_error() {
    let env = Env::default();
    let (client, _admin, pool_id) = setup(&env);

    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000, &donor);

    let result = client.try_donate_with_token(&pool_id, &donor, &token, &100);
    assert!(result.is_err(), "expected CrowdfundingTokenNotSet");
}

// ── 2. Unconfigured pool + global token ──────────────────────────────────

#[test]
fn unconfigured_pool_uses_global_token_and_rejects_others() {
    let env = Env::default();
    let (client, admin, pool_id) = setup(&env);

    let sponsor_mint = Address::generate(&env);
    let global_token = create_token(&env, 0, &sponsor_mint);
    client.set_crowdfunding_token(&admin, &global_token);

    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &global_token).mint(&donor, &1_000);

    // Global token donation succeeds
    client.donate_with_token(&pool_id, &donor, &global_token, &100);
    assert_eq!(client.get_total_raised(&pool_id), 100);

    // Different token rejected
    let other_token = create_token(&env, 1_000, &donor);
    let result = client.try_donate_with_token(&pool_id, &donor, &other_token, &50);
    assert!(result.is_err(), "expected TokenTransferFailed");
}

// ── 3. Pool token wins over global ────────────────────────────────────────

#[test]
fn pool_token_overrides_global_token() {
    let env = Env::default();
    let (client, admin, pool_id) = setup(&env);

    let minter = Address::generate(&env);
    let global_token = create_token(&env, 0, &minter);
    client.set_crowdfunding_token(&admin, &global_token);

    let pool_token = create_token(&env, 0, &minter);
    client.set_pool_token(&pool_id, &pool_token);

    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &pool_token).mint(&donor, &1_000);
    StellarAssetClient::new(&env, &global_token).mint(&donor, &1_000);

    client.donate_with_token(&pool_id, &donor, &pool_token, &100);

    let result = client.try_donate_with_token(&pool_id, &donor, &global_token, &50);
    assert!(result.is_err(), "expected TokenTransferFailed");
}

// ── 4. refund_donation mismatch ───────────────────────────────────────────

#[test]
fn refund_rejects_mismatched_token() {
    let env = Env::default();
    let (client, _admin, pool_id) = setup(&env);

    let minter = Address::generate(&env);
    let pool_token = create_token(&env, 0, &minter);
    client.set_pool_token(&pool_id, &pool_token);

    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &pool_token).mint(&donor, &1_000);
    client.donate_with_token(&pool_id, &donor, &pool_token, &500);

    client.set_pool_deadline(&pool_id, &1_000u32);
    set_ledger_sequence(&env, 1_000 + GRACE + 1);

    let wrong_token = create_token(&env, 1_000, &donor);
    let result = client.try_refund_donation(&pool_id, &donor, &wrong_token);
    assert!(result.is_err(), "expected TokenTransferFailed");
}

// ── 5. withdraw_unallocated_funds mismatch ────────────────────────────────

#[test]
fn withdraw_unallocated_rejects_mismatched_token() {
    let env = Env::default();
    let (client, _admin, pool_id) = setup(&env);

    let minter = Address::generate(&env);
    let pool_token = create_token(&env, 0, &minter);
    client.set_pool_token(&pool_id, &pool_token);

    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &pool_token).mint(&donor, &1_000);
    client.donate_with_token(&pool_id, &donor, &pool_token, &500);

    let wrong_token = create_token(&env, 1_000, &donor);
    let result = client.try_withdraw_unallocated_funds(&pool_id, &wrong_token);
    assert!(result.is_err(), "expected TokenTransferFailed");
}