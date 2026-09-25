#![cfg(test)]

// ============= ISSUE #1264: EMERGENCY WITHDRAWAL GRACE PERIOD TESTS =============

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger as _}, token::StellarAssetClient, Address, Env, String};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn setup_pending_withdrawal(
    env: &Env,
    request_timestamp: u64,
) -> (ContractClient<'_>, u32, Address, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);

    let creator = Address::generate(env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(env, "Emergency Pool"),
        &String::from_str(env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    env.ledger().set_timestamp(request_timestamp);
    let token = create_token(env, 500_000_000i128, &contract_id);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &500_000_000i128);

    (client, pool_id, admin, token)
}

/// Executing exactly at the grace period boundary succeeds — the check is
/// `time_elapsed < GRACE_PERIOD_SECS`, so equality is not a rejection.
#[test]
fn test_execute_at_grace_period_boundary_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, _, _) = setup_pending_withdrawal(&env, 0);

    env.ledger().set_timestamp(GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool_id);
}

/// Executing one second before the grace period elapses fails.
#[test]
#[should_panic(expected = "Grace period not elapsed")]
fn test_execute_one_second_before_grace_period_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, _, _) = setup_pending_withdrawal(&env, 0);

    env.ledger().set_timestamp(GRACE_PERIOD_SECS - 1);
    client.execute_emergency_withdraw(&pool_id);
}

/// Grace period elapsed is computed relative to the request's own
/// timestamp, not an absolute clock value — a request made at a nonzero
/// base timestamp is still gated by the same `GRACE_PERIOD_SECS` offset.
#[test]
fn test_grace_period_calculation_relative_to_nonzero_request_timestamp() {
    let env = Env::default();
    env.mock_all_auths();
    let base_timestamp: u64 = 1_000_000;
    let (client, pool_id, _, _) = setup_pending_withdrawal(&env, base_timestamp);

    // One second before the boundary relative to the request's own
    // timestamp must still fail.
    env.ledger()
        .set_timestamp(base_timestamp + GRACE_PERIOD_SECS - 1);
    let res = client.try_execute_emergency_withdraw(&pool_id);
    assert!(res.is_err(), "Grace period must be measured from request_timestamp");

    // At the boundary relative to the request's own timestamp it succeeds.
    env.ledger().set_timestamp(base_timestamp + GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool_id);
}

/// A successful execution transfers exactly the requested token and amount
/// from the contract to the requesting admin.
#[test]
fn test_execute_emergency_withdraw_transfers_tokens_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, admin, token) = setup_pending_withdrawal(&env, 0);
    let contract_id = client.address.clone();

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&contract_id), 500_000_000i128);
    assert_eq!(token_client.balance(&admin), 0i128);

    env.ledger().set_timestamp(GRACE_PERIOD_SECS + 1);
    client.execute_emergency_withdraw(&pool_id);

    assert_eq!(token_client.balance(&admin), 500_000_000i128);
    assert_eq!(token_client.balance(&contract_id), 0i128);
}
