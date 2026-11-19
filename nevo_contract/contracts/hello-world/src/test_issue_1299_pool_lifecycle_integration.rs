#![cfg(test)]
//! Integration tests for issue #1299 — complete pool lifecycle.
//!
//! Unlike the scattered unit-level tests elsewhere in this crate (creation,
//! contribution, state validation, refund, closure each tested in isolation),
//! this file walks a *single* pool through a realistic end-to-end sequence:
//! creation with metadata -> multiple contributions (including a repeat
//! contributor) -> a pause/resume state transition -> a real, fully executed
//! refund -> cancellation -> closure. Intermediate state (metrics, balances,
//! contribution records) is asserted after every step, not just at the end.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger, LedgerInfo},
    token::StellarAssetClient,
    Address, Env, String,
};

// REFUND_GRACE_PERIOD_LEDGERS = 17_280 (see lib.rs)
const GRACE: u32 = 17_280;

// A TTL large enough that no persistent entry expires/archives during this
// test, even after the ledger sequence is advanced past the refund grace
// period. Mirrors the pattern already proven in the refund-deadline tests.
const LARGE_TTL: u32 = 10_000_000;

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// Set the ledger sequence and timestamp, keeping a very large TTL ceiling so
/// persistent entries never expire even after a large sequence jump.
fn set_ledger_state(env: &Env, seq: u32, timestamp: u64) {
    env.ledger().set(LedgerInfo {
        sequence_number: seq,
        timestamp,
        protocol_version: env.ledger().protocol_version(),
        network_id: Default::default(),
        base_reserve: 5_000_000,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: LARGE_TTL,
        max_entry_ttl: LARGE_TTL,
    });
}

/// Full pool lifecycle: create -> multiple contributions (incl. a repeat
/// contributor) -> pause/resume -> more contributions -> cancel -> refund ->
/// close. Every meaningful step asserts pool metrics, per-donor records,
/// and (for the refund) token balances and event emission, not just the
/// final state.
#[test]
fn test_pool_lifecycle_full_integration_with_refund() {
    let env = Env::default();
    env.mock_all_auths();
    set_ledger_state(&env, 100, 1_000);

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    let title = String::from_str(&env, "Integration Lifecycle Pool");
    let description = String::from_str(&env, "Full lifecycle: create, contribute, transition, refund, close");
    let goal = 10_000_000_000u128;
    let application_deadline = 500_000u64;

    // Mint tokens to both donors up front. Using one token consistently
    // across all donate_with_token / refund_donation calls on this pool
    // (the contract enforces this once a pool's token is fixed by usage).
    let donor1_mint: i128 = 2_000_000_000;
    let donor2_mint: i128 = 3_000_000_000;
    let token = create_token(&env, donor1_mint, &donor1);
    StellarAssetClient::new(&env, &token).mint(&donor2, &donor2_mint);
    let token_client = token::Client::new(&env, &token);

    // ── Step 1: create pool with metadata ──────────────────────────────
    let pool_id = client.create_pool(&creator, &title, &description, &goal, &application_deadline);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, goal);
    assert_eq!(pool.3, 0u128, "collected starts at zero");
    assert_eq!(pool.4, false, "pool starts open");
    assert_eq!(pool.5, application_deadline);

    let (stored_title, stored_description) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, title);
    assert_eq!(stored_description, description);

    assert_eq!(client.get_total_raised(&pool_id), 0u128);
    assert_eq!(client.get_donor_count(&pool_id), 0u32);
    assert_eq!(client.get_contribution(&pool_id, &donor1), 0u128);
    assert_eq!(client.get_last_donation_at(&pool_id), 0u64);
    assert_eq!(client.get_pool_deadline(&pool_id), 0u32);

    // ── Step 2: first contribution (donor1) ─────────────────────────────
    let a1: i128 = 1_000_000_000;
    set_ledger_state(&env, 100, 1_000);
    client.donate_with_token(&pool_id, &donor1, &token, &a1);

    assert_eq!(client.get_total_raised(&pool_id), a1 as u128);
    assert_eq!(client.get_contribution(&pool_id, &donor1), a1 as u128);
    assert_eq!(client.get_donor_count(&pool_id), 1, "first-ever contributor");
    assert_eq!(client.get_last_donation_at(&pool_id), 1_000);
    assert_eq!(token_client.balance(&donor1), donor1_mint - a1);

    // ── Step 3: second contribution, new contributor (donor2) ───────────
    let a2: i128 = 2_000_000_000;
    set_ledger_state(&env, 100, 2_000);
    client.donate_with_token(&pool_id, &donor2, &token, &a2);

    assert_eq!(client.get_total_raised(&pool_id), (a1 + a2) as u128);
    assert_eq!(client.get_contribution(&pool_id, &donor2), a2 as u128);
    assert_eq!(client.get_donor_count(&pool_id), 2, "second unique contributor");
    assert_eq!(client.get_last_donation_at(&pool_id), 2_000, "advances with each contribution");
    assert_eq!(token_client.balance(&donor2), donor2_mint - a2);

    // ── Step 4: repeat contribution from donor1 ─────────────────────────
    let a3: i128 = 500_000_000;
    set_ledger_state(&env, 100, 3_000);
    client.donate_with_token(&pool_id, &donor1, &token, &a3);

    assert_eq!(client.get_total_raised(&pool_id), (a1 + a2 + a3) as u128);
    assert_eq!(client.get_contribution(&pool_id, &donor1), (a1 + a3) as u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        2,
        "repeat contributor must not be double-counted"
    );
    assert_eq!(client.get_last_donation_at(&pool_id), 3_000);
    assert_eq!(token_client.balance(&donor1), donor1_mint - a1 - a3);

    // ── Step 5: state transition — pause, then resume ────────────────────
    client.set_pool_state(&pool_id, &PoolState::Paused);
    // Nothing should change purely from pausing.
    assert_eq!(client.get_total_raised(&pool_id), (a1 + a2 + a3) as u128);
    assert_eq!(client.get_donor_count(&pool_id), 2);

    client.set_pool_state(&pool_id, &PoolState::Active);

    // ── Step 6: contribution after resuming (donor2 again) ──────────────
    let a4: i128 = 300_000_000;
    set_ledger_state(&env, 100, 4_000);
    client.donate_with_token(&pool_id, &donor2, &token, &a4);

    let total_before_refund = (a1 + a2 + a3 + a4) as u128;
    assert_eq!(client.get_total_raised(&pool_id), total_before_refund);
    assert_eq!(client.get_contribution(&pool_id, &donor2), (a2 + a4) as u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        2,
        "still only two unique contributors after resuming"
    );
    assert_eq!(client.get_last_donation_at(&pool_id), 4_000);
    assert_eq!(token_client.balance(&donor2), donor2_mint - a2 - a4);

    // ── Step 7: cancel the pool and set a refund deadline ────────────────
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    let deadline: u32 = 200; // must be > current ledger sequence (100)
    client.set_pool_deadline(&pool_id, &deadline);
    assert_eq!(client.get_pool_deadline(&pool_id), deadline);

    // ── Step 8: advance past deadline + grace period, then refund donor1 ─
    set_ledger_state(&env, deadline + GRACE, 5_000);

    let donor1_balance_before_refund = token_client.balance(&donor1);

    let refund_amount = a1 + a3;
    client.refund_donation(&pool_id, &donor1, &token);

    // env.events().all() reflects only the most recent contract invocation,
    // so the refund event must be checked immediately, before any further
    // client/token calls (even read-only ones) overwrite that snapshot.
    let refund_events = env.events().all().filter_by_contract(&contract_id);
    assert_eq!(
        refund_events.events().len(),
        1,
        "refund_donation must emit exactly one event from the pool contract"
    );

    // Donor1 receives their full contribution back.
    assert_eq!(
        token_client.balance(&donor1),
        donor1_balance_before_refund + refund_amount
    );
    // A clean round trip: donor1 ends up exactly back at their original mint,
    // since every stroop they ever contributed (a1 + a3) is refunded.
    assert_eq!(token_client.balance(&donor1), donor1_mint);

    // Donor1's contribution record is zeroed.
    assert_eq!(client.get_contribution(&pool_id, &donor1), 0u128);

    // Pool metrics decrease by exactly the refunded amount.
    let total_after_refund = total_before_refund - (refund_amount as u128);
    assert_eq!(client.get_total_raised(&pool_id), total_after_refund);

    // Donor count is unaffected by a refund (donor1 is still a known donor,
    // just with a zero balance) — refund_donation never touches "d_count".
    assert_eq!(client.get_donor_count(&pool_id), 2);

    // last_donation_at tracks *contributions*, not refunds, so it must not
    // have moved from the last real donation at t = 4_000.
    assert_eq!(client.get_last_donation_at(&pool_id), 4_000);

    // Donor2's contribution is untouched by donor1's refund.
    assert_eq!(client.get_contribution(&pool_id, &donor2), (a2 + a4) as u128);

    // ── Step 9: close the pool (valid from Cancelled) ────────────────────
    client.close_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true, "pool is closed");
    assert_eq!(pool.3, total_after_refund, "closure does not alter collected");

    // Final sanity: donor2's (never-refunded) contribution and the donor
    // count both persist correctly through closure.
    assert_eq!(client.get_contribution(&pool_id, &donor2), (a2 + a4) as u128);
    assert_eq!(client.get_donor_count(&pool_id), 2);
}
