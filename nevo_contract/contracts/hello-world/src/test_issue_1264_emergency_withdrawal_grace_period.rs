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
#[should_panic(expected = "Error(Contract, #21)")]
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

/// Multiple emergency withdrawal requests can be active simultaneously on
/// different pools, each with their own independent grace period clock.
#[test]
fn test_multiple_concurrent_emergency_withdrawals_on_different_pools() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    // Create two pools
    let pool1 = client.create_pool(
        &creator1,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "First Pool"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let pool2 = client.create_pool(
        &creator2,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Second Pool"),
        &2_000_000_000u128,
        &100_000u64,
    );

    // Request emergency withdrawal on pool1 at timestamp 1000
    env.ledger().set_timestamp(1000);
    let token1 = create_token(&env, 500_000_000i128, &contract_id);
    client.request_emergency_withdraw(&admin, &pool1, &token1, &300_000_000i128);

    // Request emergency withdrawal on pool2 at timestamp 2000 (1000 seconds later)
    env.ledger().set_timestamp(2000);
    let token2 = create_token(&env, 800_000_000i128, &contract_id);
    client.request_emergency_withdraw(&admin, &pool2, &token2, &600_000_000i128);

    // At timestamp 1000 + GRACE_PERIOD_SECS, only pool1 can be executed
    env.ledger().set_timestamp(1000 + GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool1);

    // pool2 cannot be executed yet (needs to wait until 2000 + GRACE_PERIOD_SECS)
    let res = client.try_execute_emergency_withdraw(&pool2);
    assert!(res.is_err(), "Pool2 grace period should not have elapsed yet");

    // Advance to pool2's grace period boundary
    env.ledger().set_timestamp(2000 + GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool2);
}

/// A pending emergency withdrawal request is removed from storage upon
/// successful execution and cannot be executed a second time.
#[test]
#[should_panic(expected = "Error(Contract, #21)")]
fn test_cannot_execute_emergency_withdrawal_twice() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, _, _) = setup_pending_withdrawal(&env, 0);

    env.ledger().set_timestamp(GRACE_PERIOD_SECS + 1);
    client.execute_emergency_withdraw(&pool_id);

    // Attempt to execute again should fail with EmergencyWithdrawalNotRequested
    client.execute_emergency_withdraw(&pool_id);
}

/// Emergency withdrawal can be requested with a zero amount, though the
/// execute will succeed after the grace period — the contract does not
/// enforce a minimum withdrawal amount in the request phase.
#[test]
fn test_emergency_withdrawal_with_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Zero Amount Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    env.ledger().set_timestamp(1000);
    let token = create_token(&env, 1_000_000i128, &contract_id);

    // Request with zero amount
    client.request_emergency_withdraw(&admin, &pool_id, &token, &0i128);

    // Verify the request was stored
    let request = client.get_emergency_withdrawal(&pool_id);
    assert_eq!(request.amount, 0i128);

    // Execute after grace period
    env.ledger().set_timestamp(1000 + GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool_id);

    // Execution succeeded — no tokens transferred but no error either
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&admin), 0i128);
}

/// Emergency withdrawal request timestamp is stored and used for grace
/// period calculation even when the request is made very close to the
/// maximum u64 timestamp boundary (no overflow in subtraction).
#[test]
fn test_emergency_withdrawal_with_high_timestamp_no_overflow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "High Timestamp Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &200_000u64,
    );

    // Set a very high timestamp
    let high_timestamp: u64 = u64::MAX - GRACE_PERIOD_SECS - 10_000;
    env.ledger().set_timestamp(high_timestamp);

    let token = create_token(&env, 500_000_000i128, &contract_id);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &500_000_000i128);

    // Before grace period elapses
    env.ledger().set_timestamp(high_timestamp + GRACE_PERIOD_SECS - 1);
    let res = client.try_execute_emergency_withdraw(&pool_id);
    assert!(res.is_err(), "Grace period should not have elapsed");

    // At grace period boundary
    env.ledger().set_timestamp(high_timestamp + GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool_id);
}

/// After a successful emergency withdrawal execution, a new emergency
/// withdrawal can be requested on the same pool (they are independent events).
#[test]
fn test_sequential_emergency_withdrawals_on_same_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Sequential Withdrawal Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // First withdrawal request
    env.ledger().set_timestamp(1000);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &400_000_000i128);

    // Execute first withdrawal
    env.ledger().set_timestamp(1000 + GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool_id);

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&admin), 400_000_000i128);

    // Second withdrawal request (new emergency situation)
    env.ledger().set_timestamp(2000 + GRACE_PERIOD_SECS);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &300_000_000i128);

    // Execute second withdrawal
    env.ledger().set_timestamp(2000 + (2 * GRACE_PERIOD_SECS));
    client.execute_emergency_withdraw(&pool_id);

    assert_eq!(token_client.balance(&admin), 700_000_000i128);
    assert_eq!(token_client.balance(&contract_id), 300_000_000i128);
}
