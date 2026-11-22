#![cfg(test)]
//! Tests for issue #1371 — `request_emergency_withdraw` duplicate request handling.
//!
//! Covers: (1) a second request while one is pending is rejected and leaves the
//! first untouched, (2) only the stored admin may request, (3) a request larger
//! than the pool's token balance cannot be executed, (4) a pending request is
//! cleared only by execution after the grace period, after which a new request
//! is accepted, and (5) requests are isolated per `pool_id`.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
};

const POOL_BALANCE: i128 = 10_000;

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

    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(env, &token).mint(&contract_id, &POOL_BALANCE);

    env.ledger().set_timestamp(1_000);

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
        &String::from_str(env, "Duplicate request handling"),
        &1_000_000u128,
        &1_000_000u64,
    )
}

fn pending_request(
    env: &Env,
    contract_id: &Address,
    pool_id: u32,
) -> Option<EmergencyWithdrawalRequest> {
    let key = (Symbol::new(env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
    env.as_contract(contract_id, || env.storage().persistent().get(&key))
}

// (1) A second request while one is pending is rejected.
#[test]
#[should_panic(expected = "EmergencyWithdrawalAlreadyRequested")]
fn test_second_request_while_pending_is_rejected() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);
    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &2_000i128);
}

// (1) The rejected duplicate does not replace or modify the pending request.
#[test]
fn test_rejected_duplicate_leaves_first_request_unchanged() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);
    let first = pending_request(&env, &s.contract_id, pool_id).expect("request stored");

    env.ledger().set_timestamp(5_000);
    let other_token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    assert!(s
        .client
        .try_request_emergency_withdraw(&s.admin, &pool_id, &other_token, &2_000i128)
        .is_err());

    let after = pending_request(&env, &s.contract_id, pool_id).expect("request still stored");
    assert_eq!(after, first);
    assert_eq!(after.amount, 1_000);
    assert_eq!(after.token_address, s.token);
    assert_eq!(after.request_timestamp, 1_000);
}

// (2) A non-admin address cannot request an emergency withdrawal.
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_request_from_non_admin_fails() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    let attacker = Address::generate(&env);
    s.client
        .request_emergency_withdraw(&attacker, &pool_id, &s.token, &1_000i128);
}

// (2) A failed non-admin request stores nothing, so the admin can still request.
#[test]
fn test_failed_non_admin_request_does_not_block_admin() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    let attacker = Address::generate(&env);
    assert!(s
        .client
        .try_request_emergency_withdraw(&attacker, &pool_id, &s.token, &1_000i128)
        .is_err());
    assert!(pending_request(&env, &s.contract_id, pool_id).is_none());

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);
    assert_eq!(
        pending_request(&env, &s.contract_id, pool_id)
            .unwrap()
            .requested_by,
        s.admin
    );
}

// (2) Requests fail when no admin has been configured.
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_request_without_configured_admin_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let pool_id = create_pool(&env, &client);
    let token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    client.request_emergency_withdraw(&Address::generate(&env), &pool_id, &token, &1_000i128);
}

// (3) A request larger than the pool's token balance cannot be executed, and
// the pending request is left in place.
#[test]
fn test_request_exceeding_pool_balance_cannot_be_executed() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    let excessive = POOL_BALANCE + 1;
    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &excessive);

    env.ledger().set_timestamp(1_000 + GRACE_PERIOD_SECS);
    assert!(s.client.try_execute_emergency_withdraw(&pool_id).is_err());

    let token_client = token::Client::new(&env, &s.token);
    assert_eq!(token_client.balance(&s.contract_id), POOL_BALANCE);
    assert_eq!(token_client.balance(&s.admin), 0);
    assert_eq!(
        pending_request(&env, &s.contract_id, pool_id)
            .unwrap()
            .amount,
        excessive
    );
}

// (4) A pending request cannot be executed early; it stays pending (and keeps
// blocking new requests) until the grace period has elapsed.
#[test]
fn test_stale_request_remains_pending_until_grace_period_elapses() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);

    env.ledger().set_timestamp(1_000 + GRACE_PERIOD_SECS - 1);
    assert!(s.client.try_execute_emergency_withdraw(&pool_id).is_err());
    assert!(pending_request(&env, &s.contract_id, pool_id).is_some());
    assert!(s
        .client
        .try_request_emergency_withdraw(&s.admin, &pool_id, &s.token, &500i128)
        .is_err());
}

// (4) Executing the request clears it, after which a fresh request is accepted.
#[test]
fn test_new_request_accepted_after_previous_one_is_executed() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &1_000i128);
    env.ledger().set_timestamp(1_000 + GRACE_PERIOD_SECS);
    s.client.execute_emergency_withdraw(&pool_id);
    assert!(pending_request(&env, &s.contract_id, pool_id).is_none());

    let second_timestamp = 1_000 + GRACE_PERIOD_SECS + 10;
    env.ledger().set_timestamp(second_timestamp);
    s.client
        .request_emergency_withdraw(&s.admin, &pool_id, &s.token, &2_000i128);

    let request = pending_request(&env, &s.contract_id, pool_id).unwrap();
    assert_eq!(request.amount, 2_000);
    assert_eq!(request.request_timestamp, second_timestamp);
}

// (5) A pending request on one pool does not block or affect another pool.
#[test]
fn test_requests_are_isolated_per_pool() {
    let env = Env::default();
    let s = setup(&env);
    let pool_a = create_pool(&env, &s.client);
    let pool_b = create_pool(&env, &s.client);

    s.client
        .request_emergency_withdraw(&s.admin, &pool_a, &s.token, &1_000i128);
    assert!(pending_request(&env, &s.contract_id, pool_b).is_none());

    env.ledger().set_timestamp(2_000);
    s.client
        .request_emergency_withdraw(&s.admin, &pool_b, &s.token, &3_000i128);

    let a = pending_request(&env, &s.contract_id, pool_a).unwrap();
    let b = pending_request(&env, &s.contract_id, pool_b).unwrap();
    assert_eq!(
        (a.pool_id, a.amount, a.request_timestamp),
        (pool_a, 1_000, 1_000)
    );
    assert_eq!(
        (b.pool_id, b.amount, b.request_timestamp),
        (pool_b, 3_000, 2_000)
    );

    // Executing pool A's request leaves pool B's pending request intact.
    env.ledger().set_timestamp(1_000 + GRACE_PERIOD_SECS);
    s.client.execute_emergency_withdraw(&pool_a);
    assert!(pending_request(&env, &s.contract_id, pool_a).is_none());
    assert_eq!(pending_request(&env, &s.contract_id, pool_b).unwrap(), b);
    // Pool B is still within its grace period.
    assert!(s.client.try_execute_emergency_withdraw(&pool_b).is_err());
}
