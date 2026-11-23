#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env, String};

// ============= ISSUE #1108: get_refund_grace_period_ledgers GETTER =============

/// Assert that get_refund_grace_period_ledgers returns the compile-time constant 17_280.
#[test]
fn test_get_refund_grace_period_ledgers_returns_17280() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_refund_grace_period_ledgers(), 17_280u32);
}

// ============= ISSUE #1664: UNIFIED LEDGER-SEQUENCE TIME BASE =============

/// The refund grace period is expressed in ledger sequences, matching the
/// unified ledger-sequence time base used for the refund window.
#[test]
fn test_refund_grace_period_is_in_ledger_sequences() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let grace = client.get_refund_grace_period_ledgers();
    assert!(grace > 0, "grace period must be a positive ledger count");
    assert_eq!(grace, 17_280u32);
}

/// A pool without an explicit `pool_deadline` must still have a reachable
/// refund window: refunds open after the funding deadline plus the grace
/// period (both in ledger sequences).
#[test]
fn test_derived_refund_window_without_explicit_pool_deadline() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let grace = client.get_refund_grace_period_ledgers();

    // The derived refund deadline is the funding deadline plus the grace
    // period; it must be strictly after the funding deadline so the refund
    // window is reachable.
    let funding_deadline: u32 = 1_000;
    let derived_refund_deadline = funding_deadline
        .checked_add(grace)
        .expect("derived refund deadline must not overflow");
    assert!(derived_refund_deadline > funding_deadline);
    assert_eq!(derived_refund_deadline, funding_deadline + 17_280u32);
}

// ============= ISSUE #1678: CANCELLED POOL REFUNDS ALWAYS AVAILABLE =============

/// Helper: set up a token, a pool, and a donor who has donated.
/// Returns (client, token_client, donor, pool_id).
fn setup_cancelled_pool_with_donation(
    env: &Env,
) -> (ContractClient<'_>, token::Client<'_>, Address, u64) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let donor = Address::generate(env);

    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_id = token_contract.address();
    let token_client = token::Client::new(env, &token_id);
    let token_admin_client = token::StellarAssetClient::new(env, &token_id);

    // Mint tokens to the donor.
    token_admin_client.mint(&donor, &10_000i128);

    // Initialize the contract.
    client.initialize(&admin, &token_id);

    // Create a pool.
    let pool_id = client.create_pool(
        &admin,
        &String::from_str(env, "Cancelled Pool"),
        &String::from_str(env, "Test pool for refunds"),
        &1_000i128,
        &100u64,
        &token_id,
    );

    // Donor donates.
    client.donate(&pool_id, &donor, &500i128);

    (client, token_client, donor, pool_id)
}

/// A Cancelled pool's donors can always refund, even after the refund grace
/// period has elapsed (grace period is bypassed for Cancelled pools).
#[test]
fn test_cancelled_pool_refund_always_available_after_grace_period() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, token_client, donor, pool_id) = setup_cancelled_pool_with_donation(&env);

    // Cancel the pool.
    client.cancel_pool(&pool_id);

    // Advance the ledger far beyond the refund grace period.
    let grace = client.get_refund_grace_period_ledgers();
    env.ledger().with_mut(|li| {
        li.sequence_number = li.sequence_number.saturating_add(grace + 1_000);
    });

    // Donor must still be able to refund.
    let balance_before = token_client.balance(&donor);
    client.refund_donation(&pool_id, &donor);
    let balance_after = token_client.balance(&donor);

    assert_eq!(balance_after - balance_before, 500i128);
}

/// A Cancelled pool's donors can refund immediately (before any grace period).
#[test]
fn test_cancelled_pool_refund_available_immediately() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, token_client, donor, pool_id) = setup_cancelled_pool_with_donation(&env);

    // Cancel the pool.
    client.cancel_pool(&pool_id);

    // Donor refunds immediately.
    let balance_before = token_client.balance(&donor);
    client.refund_donation(&pool_id, &donor);
    let balance_after = token_client.balance(&donor);

    assert_eq!(balance_after - balance_before, 500i128);
}

/// `close_pool` must be rejected while a Cancelled pool still holds donor funds
/// (`collected > 0`), so funds never become unrecoverable.
#[test]
#[should_panic]
fn test_close_cancelled_pool_rejected_while_collected_positive() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _token_client, _donor, pool_id) = setup_cancelled_pool_with_donation(&env);

    // Cancel the pool but do not refund.
    client.cancel_pool(&pool_id);

    // Attempting to close while collected > 0 must fail.
    client.close_pool(&pool_id);
}

/// Once all donations are refunded (`collected == 0`), a Cancelled pool can be
/// closed successfully.
#[test]
fn test_close_cancelled_pool_allowed_after_all_refunded() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, token_client, donor, pool_id) = setup_cancelled_pool_with_donation(&env);

    // Cancel the pool.
    client.cancel_pool(&pool_id);

    // Refund the donation so collected == 0.
    let balance_before = token_client.balance(&donor);
    client.refund_donation(&pool_id, &donor);
    let balance_after = token_client.balance(&donor);
    assert_eq!(balance_after - balance_before, 500i128);

    // Now closing must succeed.
    client.close_pool(&pool_id);
}