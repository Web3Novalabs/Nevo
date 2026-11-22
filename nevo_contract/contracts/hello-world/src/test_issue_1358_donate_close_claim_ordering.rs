#![cfg(test)]
//! Integration tests for issue #1358 — `donate` -> `close_pool` -> `claim_funds` ordering.
//!
//! Walks one school-linked pool through donations, an approved student's
//! claims and closure, asserting `get_total_raised` and the events published
//! by each call along the way.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, String, Val,
};

const GOAL: u128 = 1_000;

struct Setup<'a> {
    contract_id: Address,
    client: ContractClient<'a>,
    pool_id: u32,
    sponsor: Address,
    student: Address,
    token: Address,
}

fn setup(env: &Env) -> Setup<'_> {
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
        &String::from_str(env, "Ordering Pool"),
        &String::from_str(env, "donate -> close_pool -> claim_funds"),
        &GOAL,
        &school,
        &200_000u64,
    );

    let student = Address::generate(env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(env, "application"));
    client.approve_application(&pool_id, &school, &student, &true);

    // Back the pool's accounting with real tokens held by the contract.
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(env, &token).mint(&contract_id, &(GOAL as i128));

    Setup {
        contract_id,
        client,
        pool_id,
        sponsor,
        student,
        token,
    }
}

/// Assert the most recent invocation published exactly one event from the
/// pool contract, on `(topic, pool_id)` and carrying `data`.
fn assert_single_event(env: &Env, contract_id: &Address, topic: Symbol, pool_id: u32, data: Val) {
    let expected: Vec<(Address, Vec<Val>, Val)> = soroban_sdk::vec![
        env,
        (
            contract_id.clone(),
            soroban_sdk::vec![env, topic.into_val(env), pool_id.into_val(env)],
            data,
        ),
    ];
    assert_eq!(env.events().all().filter_by_contract(contract_id), expected);
}

// (1) Donations are accepted while the pool is Active, and (4) get_total_raised
// tracks every donation.
#[test]
fn test_donations_accepted_while_active_and_total_raised_tracks_them() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    assert_eq!(s.client.get_total_raised(&s.pool_id), 0);

    s.client.donate(&s.pool_id, &donor, &300u128);
    assert_single_event(
        &env,
        &s.contract_id,
        DONATION_MADE,
        s.pool_id,
        (donor.clone(), 300u128, 300u128).into_val(&env),
    );
    assert_eq!(s.client.get_total_raised(&s.pool_id), 300);

    s.client.donate(&s.pool_id, &donor, &200u128);
    assert_eq!(s.client.get_total_raised(&s.pool_id), 500);
    assert!(!s.client.get_pool(&s.pool_id).4, "pool must still be open");
}

// (2)-(5) donate -> claim_funds -> close_pool in order: claims succeed while the
// pool is open, closure succeeds after disbursement, totals stay accurate and
// each call publishes its event in sequence.
#[test]
fn test_donate_claim_close_sequence_keeps_totals_and_event_order() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    // Step 1: donation while Active.
    s.client.donate(&s.pool_id, &donor, &600u128);
    assert_single_event(
        &env,
        &s.contract_id,
        DONATION_MADE,
        s.pool_id,
        (donor.clone(), 600u128, 600u128).into_val(&env),
    );
    assert_eq!(s.client.get_total_raised(&s.pool_id), 600);

    // Step 2: the approved student claims against the collected funds.
    s.client
        .claim_funds(&s.student, &s.pool_id, &250i128, &s.token);
    assert_single_event(
        &env,
        &s.contract_id,
        FUNDS_CLAIMED,
        s.pool_id,
        (s.student.clone(), 250i128, 250i128).into_val(&env),
    );
    // Claims are tracked separately and never reduce the amount raised.
    assert_eq!(s.client.get_total_raised(&s.pool_id), 600);
    assert_eq!(s.client.get_claimed_amount(&s.pool_id, &s.student), 250);

    // Step 3: funds are disbursed, then the sponsor closes the pool.
    s.client.set_pool_state(&s.pool_id, &PoolState::Disbursed);
    assert_single_event(
        &env,
        &s.contract_id,
        POOL_STATE_SET,
        s.pool_id,
        (PoolState::Active, PoolState::Disbursed).into_val(&env),
    );

    s.client.close_pool(&s.pool_id);
    assert_single_event(
        &env,
        &s.contract_id,
        POOL_CLOSED,
        s.pool_id,
        (s.sponsor.clone(), 600u128).into_val(&env),
    );

    // Closure is visible via get_pool, and totals/claims are unchanged.
    let pool = s.client.get_pool(&s.pool_id);
    assert!(pool.4, "pool must report closed");
    assert_eq!(pool.3, 600);
    assert_eq!(s.client.get_total_raised(&s.pool_id), 600);
    assert_eq!(s.client.get_claimed_amount(&s.pool_id, &s.student), 250);
}

// (3) An approved student can keep claiming after the pool reaches its goal
// (Completed), up to the point it is closed.
#[test]
fn test_claim_succeeds_after_goal_reached_before_closure() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    s.client.donate(&s.pool_id, &donor, &GOAL);
    assert_eq!(s.client.get_total_raised(&s.pool_id), GOAL);

    s.client
        .claim_funds(&s.student, &s.pool_id, &400i128, &s.token);
    s.client
        .claim_funds(&s.student, &s.pool_id, &600i128, &s.token);

    assert_eq!(s.client.get_claimed_amount(&s.pool_id, &s.student), 1_000);
    assert_eq!(s.client.get_total_raised(&s.pool_id), GOAL);
}

// (2) close_pool blocks further donations.
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_close_pool_blocks_further_donations() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    s.client.donate(&s.pool_id, &donor, &500u128);
    s.client.set_pool_state(&s.pool_id, &PoolState::Disbursed);
    s.client.close_pool(&s.pool_id);

    // PoolIsClosed
    s.client.donate(&s.pool_id, &donor, &100u128);
}

// (3) Once closed, the pool accepts no further claims (PoolIsClosed); claims
// made before closure are preserved, as asserted in the sequence test above.
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_claim_after_close_is_rejected() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    s.client.donate(&s.pool_id, &donor, &500u128);
    s.client
        .claim_funds(&s.student, &s.pool_id, &100i128, &s.token);
    s.client.set_pool_state(&s.pool_id, &PoolState::Disbursed);
    s.client.close_pool(&s.pool_id);

    s.client
        .claim_funds(&s.student, &s.pool_id, &100i128, &s.token);
}

// (4) get_total_raised is unchanged by a rejected post-closure donation.
#[test]
fn test_total_raised_unchanged_by_rejected_donation_after_close() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    s.client.donate(&s.pool_id, &donor, &500u128);
    s.client.set_pool_state(&s.pool_id, &PoolState::Disbursed);
    s.client.close_pool(&s.pool_id);

    assert!(s.client.try_donate(&s.pool_id, &donor, &100u128).is_err());
    assert_eq!(s.client.get_total_raised(&s.pool_id), 500);
}
