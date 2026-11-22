#![cfg(test)]
//! Tests for set_pool_deadline and get_pool_deadline updates — issue #1377.
//!
//! Covers set_pool_deadline / get_pool_deadline functions:
//!   (1) Only authorized caller can update deadline.
//!   (2) get_pool_deadline reflects updated value immediately.
//!   (3) Cannot set a deadline in the past.
//!   (4) Extending vs shortening deadline both handled per spec.
//!   (5) Deadline change does not affect already-closed pools.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, String,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn setup_pool(env: &Env, client: &ContractClient) -> (u32, Address) {
    let sponsor = Address::generate(env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(env, "Deadline Test Pool"),
        &String::from_str(env, "Tests for set_pool_deadline and get_pool_deadline"),
        &1_000_000_000u128,
        &100_000u64,
    );
    (pool_id, sponsor)
}

// ── Test 1: Only authorized caller can update deadline ──────────────────────

#[test]
fn test_authorized_sponsor_can_update_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);
    let deadline = env.ledger().sequence() + 500;

    // Authorized sponsor successfully updates deadline
    client.set_pool_deadline(&pool_id, &deadline);
    assert_eq!(client.get_pool_deadline(&pool_id), deadline);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_unauthorized_caller_cannot_update_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);
    let non_sponsor = Address::generate(&env);
    let deadline = env.ledger().sequence() + 500;

    // Caller other than the pool sponsor must fail require_auth
    client
        .mock_auths(&[MockAuth {
            address: &non_sponsor,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_pool_deadline",
                args: (&pool_id, &deadline).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_pool_deadline(&pool_id, &deadline);
}

// ── Test 2: get_pool_deadline reflects updated value immediately ────────────

#[test]
fn test_get_pool_deadline_reflects_updated_value_immediately() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);

    // With no explicit refund deadline, the getter returns the derived refund
    // window: the funding deadline plus the grace period (ledger sequence base).
    let derived_deadline = client.get_pool_deadline(&pool_id);
    assert_eq!(
        derived_deadline,
        client.get_pool(&pool_id).3 + REFUND_GRACE_PERIOD_LEDGERS,
        "get_pool_deadline must derive the refund window from the funding deadline"
    );

    let first_deadline = env.ledger().sequence() + 1_000;
    client.set_pool_deadline(&pool_id, &first_deadline);
    assert_eq!(
        client.get_pool_deadline(&pool_id),
        first_deadline,
        "get_pool_deadline must reflect first updated value immediately"
    );

    let second_deadline = env.ledger().sequence() + 2_500;
    client.set_pool_deadline(&pool_id, &second_deadline);
    assert_eq!(
        client.get_pool_deadline(&pool_id),
        second_deadline,
        "get_pool_deadline must reflect subsequent update immediately"
    );
}

// ── Test 3: Cannot set a deadline in the past ───────────────────────────────

#[test]
#[should_panic(expected = "Deadline must be in the future")]
fn test_cannot_set_deadline_equal_to_current_sequence() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);
    let current_seq = env.ledger().sequence();

    // Sequence equal to current sequence must panic
    client.set_pool_deadline(&pool_id, &current_seq);
}

#[test]
#[should_panic(expected = "Deadline must be in the future")]
fn test_cannot_set_deadline_in_the_past() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);
    env.ledger().set_sequence_number(200);

    // Sequence strictly less than current sequence must panic
    let past_deadline = 199u32;
    client.set_pool_deadline(&pool_id, &past_deadline);
}

// ── Test 4: Extending vs shortening deadline both handled per spec ──────────

#[test]
fn test_extending_and_shortening_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    env.ledger().set_sequence_number(50);
    let (pool_id, _) = setup_pool(&env, &client);

    // Initial setup: set deadline to ledger 500
    let initial_deadline = 500u32;
    client.set_pool_deadline(&pool_id, &initial_deadline);
    assert_eq!(client.get_pool_deadline(&pool_id), initial_deadline);

    // Extending deadline to ledger 1500 (> 500)
    let extended_deadline = 1_500u32;
    client.set_pool_deadline(&pool_id, &extended_deadline);
    assert_eq!(
        client.get_pool_deadline(&pool_id),
        extended_deadline,
        "Extending the deadline must succeed and be reflected in get_pool_deadline"
    );

    // Shortening deadline to ledger 200 (< 1500, but > current sequence 50)
    let shortened_deadline = 200u32;
    client.set_pool_deadline(&pool_id, &shortened_deadline);
    assert_eq!(
        client.get_pool_deadline(&pool_id),
        shortened_deadline,
        "Shortening the deadline must succeed and be reflected in get_pool_deadline"
    );
}

// ── Test 5: Deadline change does not affect already-closed pools ────────────

#[test]
fn test_deadline_change_does_not_affect_already_closed_pools() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);

    // Transition pool state to Disbursed and close the pool
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    // Ensure pool is closed
    assert!(client.get_pool(&pool_id).4);

    // Update deadline on the closed pool
    let new_deadline = env.ledger().sequence() + 5_000;
    client.set_pool_deadline(&pool_id, &new_deadline);

    // Verify deadline was updated in storage
    assert_eq!(client.get_pool_deadline(&pool_id), new_deadline);

    // Invariant: Pool remains closed and unaffected
    assert!(
        client.get_pool(&pool_id).4,
        "get_pool tuple is_closed flag must remain true"
    );

    // Invariant: Donations to closed pool remain rejected
    let donor = Address::generate(&env);
    let donate_err = client.try_donate(&pool_id, &donor, &1_000u128);
    assert_eq!(
        donate_err,
        Err(Ok(ContractError::PoolIsClosed)),
        "Donating to closed pool must remain rejected with PoolIsClosed"
    );

    // Invariant: Closing an already closed pool remains rejected
    let close_err = client.try_close_pool(&pool_id);
    assert_eq!(
        close_err,
        Err(Ok(ContractError::PoolAlreadyClosed)),
        "Closing an already closed pool must remain rejected with PoolAlreadyClosed"
    );
}

// ── Test 6: set_pool_deadline bounds deadlines to the maximum horizon ──────

#[test]
fn test_set_pool_deadline_accepts_deadline_at_max_horizon_boundary() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);

    let deadline = env
        .ledger()
        .sequence()
        .saturating_add(MAX_POOL_DEADLINE_LEDGERS);
    client.set_pool_deadline(&pool_id, &deadline);
    assert_eq!(client.get_pool_deadline(&pool_id), deadline);
}

#[test]
#[should_panic(expected = "Deadline too far in the future")]
fn test_set_pool_deadline_rejects_deadline_beyond_max_horizon() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);

    let deadline = env
        .ledger()
        .sequence()
        .saturating_add(MAX_POOL_DEADLINE_LEDGERS)
        + 1;
    client.set_pool_deadline(&pool_id, &deadline);
}

// ── Test 7: set_pool_deadline rejects deadlines before the funding deadline ─

#[test]
fn test_set_pool_deadline_rejects_deadline_before_funding_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = setup_pool(&env, &client);

    // The funding deadline (application_deadline) is the lower bound for the
    // refund window; a refund deadline before it would be contradictory.
    let funding_deadline = client.get_pool(&pool_id).3;
    let contradictory_deadline = funding_deadline.saturating_sub(1);

    let err = client.try_set_pool_deadline(&pool_id, &contradictory_deadline);
    assert_eq!(
        err,
        Err(Ok(ContractError::DeadlineBeforeFundingDeadline)),
        "set_pool_deadline must reject a deadline earlier than the funding deadline"
    );

    // The stored/derived deadline must be unchanged after the rejected call.
    assert_eq!(
        client.get_pool_deadline(&pool_id),
        funding_deadline + REFUND_GRACE_PERIOD_LEDGERS,
        "rejected update must not alter the derived refund window"
    );
}
