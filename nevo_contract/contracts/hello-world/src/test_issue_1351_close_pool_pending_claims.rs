#![cfg(test)]
//! Tests for issue #1351 — `close_pool` interaction with pending claims.
//!
//! Covers: (1) closing requires the pool sponsor's authorization, (2) a closed
//! pool cannot be closed again, (3) approved-but-unclaimed allocations keep
//! their approval and recorded claims through closure while further claims
//! are rejected, (4) donations are rejected after closure, and (5) `get_pool`
//! and the stored state report the pool as Closed.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, String,
};

/// Issue #1678: a `Cancelled` pool must not be closable while it still holds
/// donor funds, and donors must always be able to recover their funds from a
/// `Cancelled` pool even when no refund deadline was ever set.
struct CancelSetup<'a> {
    contract_id: Address,
    client: ContractClient<'a>,
    sponsor: Address,
    pool_id: u32,
    token: Address,
    donors: [Address; 2],
}

fn setup_cancelled(env: &Env) -> CancelSetup<'_> {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);
    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[7u8; 32]));

    let sponsor = Address::generate(env);
    let pool_id = client.create_pool_for_school(
        &sponsor,
        &String::from_str(env, "Cancelled Pool"),
        &String::from_str(env, "close_pool on cancelled pool"),
        &10_000i128,
        &school,
        &200_000u64,
    );

    let token = env
        .register_stellar_asset_contract_v2(Address::generate(env))
        .address();
    let asset = StellarAssetClient::new(env, &token);

    let donors = [Address::generate(env), Address::generate(env)];
    for (donor, amount) in donors.iter().zip([400i128, 600i128]) {
        asset.mint(donor, &amount);
        client.donate_with_token(&pool_id, donor, &amount, &token);
    }

    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    CancelSetup {
        contract_id,
        client,
        sponsor,
        pool_id,
        token,
        donors,
    }
}

// (1678) A Cancelled pool that still holds donor funds cannot be closed.
#[test]
fn test_cancelled_pool_with_funds_cannot_close() {
    let env = Env::default();
    let s = setup_cancelled(&env);

    assert_eq!(s.client.get_total_raised(&s.pool_id), 1_000);
    assert_eq!(
        s.client.try_close_pool(&s.pool_id),
        Err(Ok(ContractError::PoolHasFunds))
    );
    assert!(!s.client.get_pool(&s.pool_id).4);
    assert_eq!(
        stored_state(&env, &s.contract_id, s.pool_id),
        PoolState::Cancelled
    );
}

// (1678) Donors can recover their funds from a Cancelled pool even though no
// refund deadline was ever configured.
#[test]
fn test_cancelled_pool_refunds_available_without_deadline() {
    let env = Env::default();
    let s = setup_cancelled(&env);
    let token_client = token::Client::new(&env, &s.token);

    for (donor, amount) in s.donors.iter().zip([400i128, 600i128]) {
        assert_eq!(token_client.balance(donor), 0);
        s.client.refund_donation(&s.pool_id, donor);
        assert_eq!(token_client.balance(donor), amount);
    }

    assert_eq!(s.client.get_total_raised(&s.pool_id), 0);
    assert_eq!(token_client.balance(&s.contract_id), 0);
}

// (1678) Once every donation has been refunded the Cancelled pool can close.
#[test]
fn test_cancelled_pool_closes_after_all_refunds() {
    let env = Env::default();
    let s = setup_cancelled(&env);

    for donor in s.donors.iter() {
        s.client.refund_donation(&s.pool_id, donor);
    }

    s.client.close_pool(&s.pool_id);

    assert!(s.client.get_pool(&s.pool_id).4);
    assert_eq!(
        stored_state(&env, &s.contract_id, s.pool_id),
        PoolState::Closed
    );
}

struct Setup<'a> {
    contract_id: Address,
    client: ContractClient<'a>,
    admin: Address,
    school: Address,
    sponsor: Address,
    pool_id: u32,
    claimed_student: Address,
    pending_student: Address,
    token: Address,
}

/// A school-linked pool with 1_000 raised and two approved students: one who
/// has claimed 300 and one with an unclaimed allocation. The pool has then
/// been marked Disbursed, ready to close.
fn setup(env: &Env) -> Setup<'_> {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);
    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[5u8; 32]));

    let sponsor = Address::generate(env);
    let pool_id = client.create_pool_for_school(
        &sponsor,
        &String::from_str(env, "Closing Pool"),
        &String::from_str(env, "close_pool with pending claims"),
        &10_000i128,
        &school,
        &200_000u64,
    );

    let claimed_student = Address::generate(env);
    let pending_student = Address::generate(env);
    for student in [&claimed_student, &pending_student] {
        client.apply_to_pool(&pool_id, student, &String::from_str(env, "application"));
        client.approve_application(&pool_id, &school, student, &true);
    }

    client.donate(&pool_id, &Address::generate(env), &1_000i128);

    let token = env
        .register_stellar_asset_contract_v2(Address::generate(env))
        .address();
    StellarAssetClient::new(env, &token).mint(&contract_id, &1_000i128);
    client.claim_funds(&claimed_student, &pool_id, &300i128, &token);

    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    Setup {
        contract_id,
        client,
        admin,
        school,
        sponsor,
        pool_id,
        claimed_student,
        pending_student,
        token,
    }
}

fn close_invoke<'a>(contract_id: &'a Address, pool_id: u32, env: &Env) -> MockAuthInvoke<'a> {
    MockAuthInvoke {
        contract: contract_id,
        fn_name: "close_pool",
        args: (pool_id,).into_val(env),
        sub_invokes: &[],
    }
}

fn stored_state(env: &Env, contract_id: &Address, pool_id: u32) -> PoolState {
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap()
            .state
    })
}

// (1) The sponsor's authorization is what close_pool requires.
#[test]
fn test_sponsor_can_close_pool() {
    let env = Env::default();
    let s = setup(&env);
    let invoke = close_invoke(&s.contract_id, s.pool_id, &env);

    s.client
        .mock_auths(&[MockAuth {
            address: &s.sponsor,
            invoke: &invoke,
        }])
        .close_pool(&s.pool_id);

    assert!(s.client.get_pool(&s.pool_id).4);
}

// (1) Neither the platform admin, the linked school nor an outsider can close
// the pool on their own authorization.
#[test]
fn test_non_sponsor_addresses_cannot_close_pool() {
    let env = Env::default();
    let s = setup(&env);
    let invoke = close_invoke(&s.contract_id, s.pool_id, &env);
    let outsider = Address::generate(&env);

    for caller in [&s.admin, &s.school, &outsider] {
        assert!(
            s.client
                .mock_auths(&[MockAuth {
                    address: caller,
                    invoke: &invoke,
                }])
                .try_close_pool(&s.pool_id)
                .is_err(),
            "close_pool must not succeed without the sponsor's authorization"
        );
    }

    assert!(!s.client.get_pool(&s.pool_id).4);
    assert_eq!(
        stored_state(&env, &s.contract_id, s.pool_id),
        PoolState::Disbursed
    );
}

// (2) A pool that is already closed cannot be closed again.
#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_cannot_close_already_closed_pool() {
    let env = Env::default();
    let s = setup(&env);

    s.client.close_pool(&s.pool_id);
    s.client.close_pool(&s.pool_id);
}

// (3) Approved allocations and recorded claims survive closure unchanged.
#[test]
fn test_approved_allocations_and_claims_persist_through_closure() {
    let env = Env::default();
    let s = setup(&env);
    let approved = String::from_str(&env, "Approved");

    s.client.close_pool(&s.pool_id);

    for student in [&s.claimed_student, &s.pending_student] {
        assert_eq!(
            s.client.get_application_status(&s.pool_id, student),
            approved
        );
        assert!(s.client.has_applied(&s.pool_id, student));
    }
    assert_eq!(
        s.client.get_claimed_amount(&s.pool_id, &s.claimed_student),
        300
    );
    assert_eq!(
        s.client.get_claimed_amount(&s.pool_id, &s.pending_student),
        0
    );
    assert_eq!(s.client.get_total_raised(&s.pool_id), 1_000);
}

// (3) Once closed, the pending student's unclaimed allocation can no longer be
// drawn (PoolIsClosed), and nothing is transferred.
#[test]
fn test_pending_claim_after_closure_is_rejected_without_transfer() {
    let env = Env::default();
    let s = setup(&env);
    let token_client = token::Client::new(&env, &s.token);
    let contract_balance = token_client.balance(&s.contract_id);

    s.client.close_pool(&s.pool_id);

    assert_eq!(
        s.client
            .try_claim_funds(&s.pending_student, &s.pool_id, &100i128, &s.token),
        Err(Ok(ContractError::PoolIsClosed))
    );
    assert_eq!(token_client.balance(&s.pending_student), 0);
    assert_eq!(token_client.balance(&s.contract_id), contract_balance);
    assert_eq!(
        s.client.get_claimed_amount(&s.pool_id, &s.pending_student),
        0
    );
}

// (4) New donations are rejected after closure.
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_donation_rejected_after_closure() {
    let env = Env::default();
    let s = setup(&env);

    s.client.close_pool(&s.pool_id);
    s.client
        .donate(&s.pool_id, &Address::generate(&env), &100i128);
}

// (5) get_pool and the stored state reflect the closure; collected is unchanged.
#[test]
fn test_get_pool_reflects_closed_pool() {
    let env = Env::default();
    let s = setup(&env);

    let before = s.client.get_pool(&s.pool_id);
    assert!(!before.4);

    s.client.close_pool(&s.pool_id);

    let after = s.client.get_pool(&s.pool_id);
    assert!(after.4, "is_closed must be true after close_pool");
    assert_eq!(after.0, s.pool_id);
    assert_eq!(after.1, s.sponsor);
    assert_eq!(after.2, before.2);
    assert_eq!(after.3, before.3);
    assert_eq!(after.5, before.5);
    assert_eq!(
        stored_state(&env, &s.contract_id, s.pool_id),
        PoolState::Closed
    );
}
