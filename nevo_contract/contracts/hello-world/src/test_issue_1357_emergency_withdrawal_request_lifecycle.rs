#![cfg(test)]
//! Tests for issue #1357 — `EmergencyWithdrawalRequest` lifecycle.
//!
//! Covers: (1) the request is created by `request_emergency_withdraw`,
//! (2) its fields (requester, timestamp, amount) are accurate at creation,
//! (3) it is removed after `execute_emergency_withdraw`, (4) a second active
//! request for the same pool is rejected, and (5) it persists unchanged across
//! contract calls until it is resolved.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

const POOL_BALANCE: i128 = 10_000;
const REQUEST_TIMESTAMP: u64 = 1_000;

struct Setup<'a> {
    contract_id: Address,
    client: ContractClient<'a>,
    admin: Address,
    token: Address,
}

fn setup(env: &Env) -> Setup<'_> {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);
    // `request_emergency_withdraw` reads the admin from persistent storage.
    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&Symbol::new(env, ADMIN_KEY), &admin);
    });

    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(env, &token).mint(&contract_id, &POOL_BALANCE);

    env.ledger().set_timestamp(REQUEST_TIMESTAMP);

    Setup {
        contract_id,
        client,
        admin,
        token,
    }
}

fn create_pool(env: &Env, client: &ContractClient<'_>) -> u32 {
    client.create_pool(
        &Address::generate(env),
        &String::from_str(env, "Emergency Pool"),
        &String::from_str(env, "Emergency withdrawal request lifecycle"),
        &1_000_000i128,
        &1_000_000u64,
    )
}

fn stored_request(
    env: &Env,
    contract_id: &Address,
    pool_id: u32,
) -> Option<EmergencyWithdrawalRequest> {
    let key = (Symbol::new(env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
    env.as_contract(contract_id, || env.storage().persistent().get(&key))
}

// (1) No request exists before `request_emergency_withdraw` is called.
#[test]
fn test_no_request_before_request_emergency_withdraw() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    assert!(stored_request(&env, &s.contract_id, pool_id).is_none());
    assert!(s.client.try_get_emergency_withdrawal(&pool_id).is_err());
}

// (1) `request_emergency_withdraw` stores a request for the pool.
#[test]
fn test_request_is_created_on_request_emergency_withdraw() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);

    let request = stored_request(&env, &s.contract_id, pool_id).expect("request stored");
    assert_eq!(request, s.client.get_emergency_withdrawal(&pool_id));
}

// (2) Requester, timestamp, amount (and pool / token) are accurate at creation.
#[test]
fn test_request_fields_are_accurate_at_creation() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &2_500i128);

    let request = s.client.get_emergency_withdrawal(&pool_id);
    assert_eq!(
        request,
        EmergencyWithdrawalRequest {
            pool_id,
            token_address: s.token.clone(),
            amount: 2_500,
            request_timestamp: REQUEST_TIMESTAMP,
            requested_by: s.admin.clone(),
        }
    );
}

// (2) The timestamp reflects the ledger time of the request call itself.
#[test]
fn test_request_timestamp_matches_ledger_time_at_request() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    env.ledger().set_timestamp(REQUEST_TIMESTAMP + 777);
    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);

    assert_eq!(
        s.client.get_emergency_withdrawal(&pool_id).request_timestamp,
        REQUEST_TIMESTAMP + 777
    );
}

// (3) The request is removed after `execute_emergency_withdraw`.
#[test]
fn test_request_is_cleared_after_execute_emergency_withdraw() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);

    env.ledger()
        .set_timestamp(REQUEST_TIMESTAMP + GRACE_PERIOD_SECS);
    s.client.execute_emergency_withdraw(&pool_id);

    assert!(stored_request(&env, &s.contract_id, pool_id).is_none());
    assert!(s.client.try_get_emergency_withdrawal(&pool_id).is_err());
    assert_eq!(TokenClient::new(&env, &s.token).balance(&s.admin), 1_000);
}

// (3) Once cleared, the same request cannot be executed a second time.
#[test]
#[should_panic(expected = "Error(Contract, #21)")]
fn test_cleared_request_cannot_be_executed_again() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);

    env.ledger()
        .set_timestamp(REQUEST_TIMESTAMP + GRACE_PERIOD_SECS);
    s.client.execute_emergency_withdraw(&pool_id);
    s.client.execute_emergency_withdraw(&pool_id);
}

// (4) A second simultaneous request for the same pool is rejected.
#[test]
#[should_panic(expected = "EmergencyWithdrawalAlreadyRequested")]
fn test_second_active_request_for_same_pool_is_rejected() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);
    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &500i128);
}

// (4) A new request is accepted once the previous one has been resolved.
#[test]
fn test_new_request_allowed_after_previous_is_resolved() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);
    env.ledger()
        .set_timestamp(REQUEST_TIMESTAMP + GRACE_PERIOD_SECS);
    s.client.execute_emergency_withdraw(&pool_id);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &500i128);

    let request = s.client.get_emergency_withdrawal(&pool_id);
    assert_eq!(request.amount, 500);
    assert_eq!(request.request_timestamp, REQUEST_TIMESTAMP + GRACE_PERIOD_SECS);
}

// (5) The request persists unchanged across unrelated contract calls and
// time advancing, until it is resolved.
#[test]
fn test_request_persists_across_calls_until_resolved() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);
    let other_pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);
    let original = s.client.get_emergency_withdrawal(&pool_id);

    // Unrelated calls and time advancing within the grace period.
    env.ledger()
        .set_timestamp(REQUEST_TIMESTAMP + GRACE_PERIOD_SECS / 2);
    s.client
        .request_emergency_withdraw(&s.admin, &other_pool_id, &s.token, &200i128);
    let _ = s.client.get_pool(&pool_id);
    assert_eq!(s.client.get_emergency_withdrawal(&pool_id), original);

    // A premature execution attempt fails and leaves the request in place.
    assert!(s.client.try_execute_emergency_withdraw(&pool_id).is_err());
    assert_eq!(s.client.get_emergency_withdrawal(&pool_id), original);
    assert_eq!(
        stored_request(&env, &s.contract_id, pool_id),
        Some(original.clone())
    );

    // Resolving it removes only this pool's request.
    env.ledger()
        .set_timestamp(REQUEST_TIMESTAMP + GRACE_PERIOD_SECS);
    s.client.execute_emergency_withdraw(&pool_id);
    assert!(stored_request(&env, &s.contract_id, pool_id).is_none());
    assert_eq!(
        s.client.get_emergency_withdrawal(&other_pool_id).amount,
        200
    );
}
