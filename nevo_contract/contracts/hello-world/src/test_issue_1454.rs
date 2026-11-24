#![cfg(test)]
//! Issue #1454 — donate() records a donation attributed to any address
//! with no donor authorization.
//!
//! donate() (lib.rs ~line 904) never calls donor.require_auth(), so caller A
//! can call donate(pool_id, donor=B, amount) and inflate B's recorded
//! contribution and the pool's donor count without B's consent or any token
//! transfer.
//!
//! These tests document the current behaviour. Whether this is intentional
//! (pre-transfer bookkeeping) or a bug requiring auth is tracked in #1454.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Issue #1454, test 1.
///
/// Caller A calls donate(pool_id, donor=B, amount) with mock_all_auths.
/// The call succeeds without B ever authorising anything. The contribution
/// is recorded under B's address.
#[test]
fn test_donate_succeeds_without_donor_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_b = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1454 Pool"),
        &String::from_str(&env, "No auth required on donate"),
        &1_000_000i128,
        &1_000_000u64,
    );

    // donate() does no token transfer and requires no auth. It succeeds for
    // any (pool_id, donor, amount) triple while the pool is Active.
    client.donate(&pool_id, &donor_b, &5_000i128);

    // The accounting is recorded under donor_b even though donor_b never
    // authorised the call.
    assert_eq!(client.get_contribution(&pool_id, &donor_b), 5_000i128);
}

/// Issue #1454, test 2.
///
/// Caller A spoofs donor=B. The pool's donor count increments and B's
/// contribution is inflated without any action from B.
#[test]
fn test_spoofed_donate_inflates_donor_count() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_b = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1454 Spoof Pool"),
        &String::from_str(&env, "Spoofed donor inflates count"),
        &1_000_000i128,
        &1_000_000u64,
    );

    // Before the spoofed call, neither the count nor B's contribution exist.
    assert_eq!(client.get_donor_count(&pool_id), 0);
    assert_eq!(client.get_contribution(&pool_id, &donor_b), 0i128);

    // A (the caller, irrelevant under mock_all_auths) donates on behalf of B.
    client.donate(&pool_id, &donor_b, &10_000i128);

    // Bug: count is 1 and contribution is recorded for B with no auth.
    assert_eq!(client.get_donor_count(&pool_id), 1);
    assert_eq!(client.get_contribution(&pool_id, &donor_b), 10_000i128);
}

/// Issue #1454, test 3.
///
/// donate() records accounting without touching any token balance. The pool's
/// collected field increases and B's contribution is set, but no token
/// contract is involved and B's on-chain balance is unaffected.
#[test]
fn test_donate_with_no_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_b = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1454 No-Transfer Pool"),
        &String::from_str(&env, "Accounting with no token transfer"),
        &1_000_000i128,
        &1_000_000u64,
    );

    // Pool starts with zero collected.
    let pool_before = client.get_pool(&pool_id);
    assert_eq!(pool_before.3, 0i128);

    // No token setup required — donate() does not call any token contract.
    client.donate(&pool_id, &donor_b, &1_000i128);

    // Contribution is recorded for B.
    assert_eq!(client.get_contribution(&pool_id, &donor_b), 1_000i128);

    // Pool's collected field increased.
    let pool_after = client.get_pool(&pool_id);
    assert_eq!(pool_after.3, 1_000i128);
}
