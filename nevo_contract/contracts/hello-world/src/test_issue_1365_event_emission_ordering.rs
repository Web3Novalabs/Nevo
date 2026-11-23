#![cfg(test)]
//! Event emission consistency across multi-step flows — issue #1365.
//!
//! Covers (1) event order within and across calls, (2) every state-mutating
//! call emitting an event, (3) payloads matching final stored state, (4) no
//! duplicate events for a single action and (5) events on the admin/school
//! privileged paths.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, BytesN, Env, IntoVal, String, Val,
};

const GOAL: i128 = 1_000;
const APPLICATION_DEADLINE: u64 = 200_000;

struct Setup<'a> {
    contract_id: Address,
    client: ContractClient<'a>,
    admin: Address,
    school: Address,
    sponsor: Address,
    pool_id: u32,
    token: Address,
}

fn setup(env: &Env) -> Setup<'_> {
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.min_persistent_entry_ttl = 20_000;
    });
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);

    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[1u8; 32]));

    let sponsor = Address::generate(env);
    let pool_id = client.create_pool_for_school(
        &sponsor,
        &String::from_str(env, "Event Pool"),
        &String::from_str(env, "event emission ordering"),
        &GOAL,
        &school,
        &APPLICATION_DEADLINE,
    );

    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    Setup {
        contract_id,
        client,
        admin,
        school,
        sponsor,
        pool_id,
        token,
    }
}

/// Number of events the pool contract published in the most recent invocation.
fn event_count(env: &Env, contract_id: &Address) -> usize {
    env.events()
        .all()
        .filter_by_contract(contract_id)
        .events()
        .len()
}

/// Assert the most recent invocation published exactly one event from the
/// pool contract, on `topics` and carrying `data`.
fn assert_single_event(env: &Env, contract_id: &Address, topics: Vec<Val>, data: Val) {
    let expected: Vec<(Address, Vec<Val>, Val)> =
        soroban_sdk::vec![env, (contract_id.clone(), topics, data)];
    assert_eq!(env.events().all().filter_by_contract(contract_id), expected);
}

// (1) Within a single call, events are published in the order the
// operations run: the creation fee is charged before the pool is created.
#[test]
fn test_create_pool_with_fee_emits_fee_paid_before_pool_created() {
    let env = Env::default();
    let s = setup(&env);
    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Fee Pool");
    let description = String::from_str(&env, "fee then pool");

    s.client.set_creation_fee(&s.admin, &50i128);
    StellarAssetClient::new(&env, &s.token).mint(&creator, &50i128);

    let pool_id = s.client.create_pool_with_fee(
        &creator,
        &title,
        &description,
        &GOAL,
        &APPLICATION_DEADLINE,
        &s.token,
    );

    let expected: Vec<(Address, Vec<Val>, Val)> = soroban_sdk::vec![
        &env,
        (
            s.contract_id.clone(),
            soroban_sdk::vec![&env, FEE_PAID.into_val(&env)],
            (creator.clone(), 50i128).into_val(&env),
        ),
        (
            s.contract_id.clone(),
            soroban_sdk::vec![&env, POOL_CREATED.into_val(&env), pool_id.into_val(&env)],
            (creator.clone(), GOAL, title, description).into_val(&env),
        ),
    ];
    assert_eq!(
        env.events().all().filter_by_contract(&s.contract_id),
        expected,
        "fee_paid must be published before pool_crtd"
    );
}

// (1) donate_with_token moves the tokens before recording the contribution,
// so the token transfer event precedes the pool's contrib event.
#[test]
fn test_donate_with_token_transfer_event_precedes_contribution_event() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &s.token).mint(&donor, &300i128);

    s.client
        .donate_with_token(&s.pool_id, &donor, &s.token, &300i128);

    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, CONTRIBUTION.into_val(&env), s.pool_id.into_val(&env)],
        (donor.clone(), 300i128, 300i128, true).into_val(&env),
    );

    let all = env.events().all();
    let transfer = all.filter_by_contract(&s.token).events()[0].clone();
    let contribution = all.filter_by_contract(&s.contract_id).events()[0].clone();
    let transfer_pos = all.events().iter().position(|e| *e == transfer).unwrap();
    let contribution_pos = all.events().iter().position(|e| *e == contribution).unwrap();
    assert!(
        transfer_pos < contribution_pos,
        "token transfer must be emitted before the contrib event"
    );
}

// (1) Across a multi-step flow, each call publishes its own event in the
// order the calls are made.
#[test]
fn test_apply_approve_donate_disburse_close_flow_emits_events_in_order() {
    let env = Env::default();
    let s = setup(&env);
    let student = Address::generate(&env);
    let donor = Address::generate(&env);

    s.client
        .apply_to_pool(&s.pool_id, &student, &String::from_str(&env, "application"));
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, APPLICATION_SUBMITTED.into_val(&env), s.pool_id.into_val(&env)],
        (student.clone(), 1u32, false).into_val(&env),
    );

    s.client
        .approve_application(&s.pool_id, &s.school, &student, &true);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, APP_APPROVED.into_val(&env), s.pool_id.into_val(&env)],
        (student.clone(), true).into_val(&env),
    );

    s.client.donate(&s.pool_id, &donor, &400i128);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, DONATION_MADE.into_val(&env), s.pool_id.into_val(&env)],
        (donor.clone(), 400i128, 400i128).into_val(&env),
    );

    s.client.set_pool_state(&s.pool_id, &PoolState::Disbursed);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, POOL_STATE_SET.into_val(&env), s.pool_id.into_val(&env)],
        (PoolState::Active, PoolState::Disbursed).into_val(&env),
    );

    s.client.close_pool(&s.pool_id);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, POOL_CLOSED.into_val(&env), s.pool_id.into_val(&env)],
        (s.sponsor.clone(), 400i128).into_val(&env),
    );
}

// (2) Each state-mutating call in a pool's lifecycle emits at least one event.
#[test]
fn test_each_state_mutating_call_emits_at_least_one_event() {
    let env = Env::default();
    let s = setup(&env);
    let student = Address::generate(&env);
    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &s.token).mint(&donor, &100i128);

    s.client.set_admin(&s.admin);
    assert!(event_count(&env, &s.contract_id) >= 1, "set_admin");

    let other_school = Address::generate(&env);
    s.client
        .register_school(&other_school, &BytesN::from_array(&env, &[2u8; 32]));
    assert!(event_count(&env, &s.contract_id) >= 1, "register_school");

    s.client.create_pool(
        &s.sponsor,
        &String::from_str(&env, "Second Pool"),
        &String::from_str(&env, "second"),
        &GOAL,
        &APPLICATION_DEADLINE,
    );
    assert!(event_count(&env, &s.contract_id) >= 1, "create_pool");

    s.client
        .apply_to_pool(&s.pool_id, &student, &String::from_str(&env, "application"));
    assert!(event_count(&env, &s.contract_id) >= 1, "apply_to_pool");

    s.client
        .approve_application(&s.pool_id, &s.school, &student, &true);
    assert!(event_count(&env, &s.contract_id) >= 1, "approve_application");

    s.client.donate(&s.pool_id, &donor, &200i128);
    assert!(event_count(&env, &s.contract_id) >= 1, "donate");

    s.client
        .donate_with_token(&s.pool_id, &donor, &s.token, &100i128);
    assert!(event_count(&env, &s.contract_id) >= 1, "donate_with_token");

    let deadline = env.ledger().sequence() + 100;
    s.client.set_pool_deadline(&s.pool_id, &deadline);
    assert!(event_count(&env, &s.contract_id) >= 1, "set_pool_deadline");

    s.client.set_creation_fee(&s.admin, &10i128);
    assert!(event_count(&env, &s.contract_id) >= 1, "set_creation_fee");

    s.client.set_pool_state(&s.pool_id, &PoolState::Disbursed);
    assert!(event_count(&env, &s.contract_id) >= 1, "set_pool_state");

    s.client.close_pool(&s.pool_id);
    assert!(event_count(&env, &s.contract_id) >= 1, "close_pool");
}

// (3) A donation event reports the cumulative collected amount after the
// write, not the pre-donation total or the donation alone.
#[test]
fn test_donation_payload_reports_final_collected_total() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    s.client.donate(&s.pool_id, &donor, &300i128);
    s.client.donate(&s.pool_id, &donor, &200i128);

    let total = s.client.get_total_raised(&s.pool_id);
    assert_eq!(total, 500);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, DONATION_MADE.into_val(&env), s.pool_id.into_val(&env)],
        (donor.clone(), 200i128, total).into_val(&env),
    );
}

// (3) The donation that reaches the goal reports the final total, and the
// stored pool has moved to Completed.
#[test]
fn test_goal_reaching_donation_payload_matches_completed_pool() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    s.client.donate(&s.pool_id, &donor, &GOAL);

    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, DONATION_MADE.into_val(&env), s.pool_id.into_val(&env)],
        (donor.clone(), GOAL, GOAL).into_val(&env),
    );
    assert_eq!(s.client.get_total_raised(&s.pool_id), GOAL);
    assert_eq!(s.client.get_pool_state(&s.pool_id), PoolState::Completed);
}

// (3) State-change and closure payloads match the pool as stored afterwards.
#[test]
fn test_state_and_close_payloads_match_stored_pool() {
    let env = Env::default();
    let s = setup(&env);

    s.client.set_pool_state(&s.pool_id, &PoolState::Cancelled);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, POOL_STATE_SET.into_val(&env), s.pool_id.into_val(&env)],
        (PoolState::Active, PoolState::Cancelled).into_val(&env),
    );
    assert_eq!(s.client.get_pool_state(&s.pool_id), PoolState::Cancelled);

    s.client.close_pool(&s.pool_id);
    let pool = s.client.get_pool(&s.pool_id);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, POOL_CLOSED.into_val(&env), s.pool_id.into_val(&env)],
        (pool.1.clone(), pool.3).into_val(&env),
    );
    assert!(pool.4, "pool must report closed");
    assert_eq!(s.client.get_pool_state(&s.pool_id), PoolState::Closed);
}

// (3) A refund event carries the refunded contribution, and the stored totals
// and balances reflect the refund once the call returns.
#[test]
fn test_refund_payload_matches_post_refund_state() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);
    StellarAssetClient::new(&env, &s.token).mint(&donor, &400i128);

    s.client
        .donate_with_token(&s.pool_id, &donor, &s.token, &400i128);

    let deadline = env.ledger().sequence() + 10;
    s.client.set_pool_deadline(&s.pool_id, &deadline);
    env.ledger().with_mut(|li| {
        li.sequence_number = deadline + REFUND_GRACE_PERIOD_LEDGERS;
    });

    s.client.refund_donation(&s.pool_id, &donor, &s.token);

    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, DONATION_REFUND.into_val(&env), s.pool_id.into_val(&env)],
        (donor.clone(), 400i128).into_val(&env),
    );
    assert_eq!(s.client.get_total_raised(&s.pool_id), 0);
    assert_eq!(TokenClient::new(&env, &s.token).balance(&donor), 400);
}

// (4) A single logical action never publishes duplicate events, including
// calls that delegate to other contract functions internally.
#[test]
fn test_single_actions_emit_exactly_one_event() {
    let env = Env::default();
    let s = setup(&env);
    let student = Address::generate(&env);
    let donor = Address::generate(&env);

    // create_pool_for_school delegates to create_pool.
    s.client.create_pool_for_school(
        &s.sponsor,
        &String::from_str(&env, "Another Pool"),
        &String::from_str(&env, "another"),
        &GOAL,
        &s.school,
        &APPLICATION_DEADLINE,
    );
    assert_eq!(event_count(&env, &s.contract_id), 1, "create_pool_for_school");

    // create_pool_with_fee with no fee configured only creates the pool.
    s.client.create_pool_with_fee(
        &s.sponsor,
        &String::from_str(&env, "Free Pool"),
        &String::from_str(&env, "no fee"),
        &GOAL,
        &APPLICATION_DEADLINE,
        &s.token,
    );
    assert_eq!(event_count(&env, &s.contract_id), 1, "create_pool_with_fee");

    s.client
        .apply_to_pool(&s.pool_id, &student, &String::from_str(&env, "application"));
    assert_eq!(event_count(&env, &s.contract_id), 1, "apply_to_pool");

    // approve_application delegates to set_application_status.
    s.client
        .approve_application(&s.pool_id, &s.school, &student, &true);
    assert_eq!(event_count(&env, &s.contract_id), 1, "approve_application");

    s.client.donate(&s.pool_id, &donor, &100i128);
    assert_eq!(event_count(&env, &s.contract_id), 1, "donate");

    s.client.set_pool_state(&s.pool_id, &PoolState::Disbursed);
    assert_eq!(event_count(&env, &s.contract_id), 1, "set_pool_state");

    s.client.close_pool(&s.pool_id);
    assert_eq!(event_count(&env, &s.contract_id), 1, "close_pool");
}

// (5) Admin-privileged paths publish their events.
#[test]
fn test_admin_privileged_paths_emit_events() {
    let env = Env::default();
    let s = setup(&env);

    s.client.set_admin(&s.admin);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, ADMIN_SET.into_val(&env)],
        s.admin.into_val(&env),
    );

    let school = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[3u8; 32]);
    s.client.register_school(&school, &hash);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, SCHOOL_REGISTERED.into_val(&env), school.into_val(&env)],
        hash.into_val(&env),
    );

    s.client.set_creation_fee(&s.admin, &50i128);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, FEE_UPDATED.into_val(&env)],
        50i128.into_val(&env),
    );

    let creator = Address::generate(&env);
    StellarAssetClient::new(&env, &s.token).mint(&creator, &50i128);
    s.client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Fee Pool"),
        &String::from_str(&env, "pays the fee"),
        &GOAL,
        &APPLICATION_DEADLINE,
        &s.token,
    );

    let claimed = s.client.claim_protocol_fees(&s.admin, &s.token);
    assert_eq!(claimed, 50);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, FEES_CLAIMED.into_val(&env), s.admin.into_val(&env)],
        (50i128,).into_val(&env),
    );
    assert_eq!(s.client.get_unclaimed_fees(), 0);
    assert_eq!(TokenClient::new(&env, &s.token).balance(&s.admin), 50);
}

// (5) School-privileged approvals publish an event for both approval and
// rejection.
#[test]
fn test_school_privileged_paths_emit_events() {
    let env = Env::default();
    let s = setup(&env);
    let approved = Address::generate(&env);
    let rejected = Address::generate(&env);

    s.client
        .apply_to_pool(&s.pool_id, &approved, &String::from_str(&env, "first"));
    s.client
        .apply_to_pool(&s.pool_id, &rejected, &String::from_str(&env, "second"));

    s.client
        .approve_application(&s.pool_id, &s.school, &approved, &true);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, APP_APPROVED.into_val(&env), s.pool_id.into_val(&env)],
        (approved.clone(), true).into_val(&env),
    );

    s.client
        .approve_application(&s.pool_id, &s.school, &rejected, &false);
    assert_single_event(
        &env,
        &s.contract_id,
        soroban_sdk::vec![&env, APP_APPROVED.into_val(&env), s.pool_id.into_val(&env)],
        (rejected.clone(), false).into_val(&env),
    );
}
