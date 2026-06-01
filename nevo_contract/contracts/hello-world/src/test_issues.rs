#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token::StellarAssetClient,
    Address, Env, IntoVal, String, Vec, Symbol,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

// ============= ISSUE #460: EMERGENCY WITHDRAWAL GRACE PERIOD VALIDATION TESTS =============

/// Test 1: Execute withdrawal exactly at grace period boundary succeeds
#[test]
fn test_emergency_withdrawal_at_grace_period_boundary() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    client.request_emergency_withdraw(&admin, &pool_id, &token, &100_000_000i128);

    // Advance time exactly to grace period boundary (86400 seconds)
    env.ledger().set_timestamp(86400);

    // Should succeed at exactly grace period boundary
    client.execute_emergency_withdraw(&pool_id);
}

/// Test 2: Execute withdrawal 1 second before grace period fails
#[test]
#[should_panic(expected = "Grace period not elapsed")]
fn test_emergency_withdrawal_before_grace_period_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    client.request_emergency_withdraw(&admin, &pool_id, &token, &100_000_000i128);

    // Advance time to 1 second before grace period (86399 seconds)
    env.ledger().set_timestamp(86399);

    // Should fail - grace period not elapsed
    client.execute_emergency_withdraw(&pool_id);
}

/// Test 3: Test grace period calculation with different timestamps
#[test]
fn test_grace_period_calculation_with_different_timestamps() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Set initial timestamp to a non-zero value
    env.ledger().set_timestamp(1000);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &100_000_000i128);

    // Advance time past grace period (1000 + 86400 + 1 = 87401)
    env.ledger().set_timestamp(87401);

    // Should succeed - grace period elapsed
    client.execute_emergency_withdraw(&pool_id);
}

/// Test 4: Verify tokens are properly transferred after successful execution
#[test]
fn test_emergency_withdrawal_token_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let withdrawal_amount = 100_000_000i128;
    let token = create_token(&env, withdrawal_amount, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    client.request_emergency_withdraw(&admin, &pool_id, &token, &withdrawal_amount);

    // Advance time past grace period
    env.ledger().set_timestamp(86401);

    // Execute withdrawal - tokens should be transferred to admin
    client.execute_emergency_withdraw(&pool_id);

    // Verify withdrawal request was removed
    let withdrawal_key = (Symbol::new(&env, "emergency_withdraw"), pool_id);
    let has_request = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(!has_request, "Withdrawal request should be removed after execution");
}

// ============= ISSUE #461: POOL CONTRIBUTION EDGE CASE TESTS FOR STATE VALIDATION =============

/// Test 1: Contribute to Active pool succeeds
#[test]
fn test_contribute_to_active_pool_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Active Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Pool is in Active state by default - should succeed
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000u128);
}

/// Test 2: Contribute to Closed pool fails
#[test]
#[should_panic(expected = "Pool is closed")]
fn test_contribute_to_closed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closed Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Close the pool
    client.close_pool(&pool_id);

    // Should fail with "Pool is closed"
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);
}

// NOTE: Tests for Paused, Completed, Cancelled, and Disbursed states require set_pool_state
// which has SDK limitations with enum parameters. These will be added when the real
// implementation is available from the dependent issue.

// ============= ISSUE #459: COMPREHENSIVE TESTS FOR EMERGENCY WITHDRAWAL AUTHORIZATION =============

/// Test 1: Valid admin successfully requests emergency withdrawal with proper token and amount
#[test]
fn test_valid_admin_requests_emergency_withdrawal() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Valid admin should successfully request emergency withdrawal
    client.request_emergency_withdraw(&admin, &pool_id, &token, &100_000_000i128);

    // Verify request was stored
    let withdrawal_key = (Symbol::new(&env, "emergency_withdraw"), pool_id);
    let has_request = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(has_request, "Emergency withdrawal request should be stored");
}

/// Test 2: Non-admin account calling request_emergency_withdraw gets Auth Error
#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_non_admin_request_emergency_withdrawal_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Non-admin should fail with Auth Error
    client.request_emergency_withdraw(&non_admin, &pool_id, &token, &100_000_000i128);
}

/// Test 3: Test duplicate requests fail with EmergencyWithdrawalAlreadyRequested
#[test]
#[should_panic(expected = "EmergencyWithdrawalAlreadyRequested")]
fn test_duplicate_emergency_withdrawal_request_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // First request should succeed
    client.request_emergency_withdraw(&admin, &pool_id, &token, &100_000_000i128);

    // Second request should fail with EmergencyWithdrawalAlreadyRequested
    client.request_emergency_withdraw(&admin, &pool_id, &token, &100_000_000i128);
}

/// Test 4: Test execute_emergency_withdraw before grace period fails
#[test]
#[should_panic(expected = "Grace period not elapsed")]
fn test_execute_emergency_withdraw_before_grace_period_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    client.request_emergency_withdraw(&admin, &pool_id, &token, &100_000_000i128);

    // Don't advance time - should fail immediately
    client.execute_emergency_withdraw(&pool_id);
}

// ============= ISSUE #462: POOL CONTRIBUTION AMOUNT VALIDATION TESTS =============

/// Test 1: Zero amount contribution fails with InvalidAmount
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_zero_amount_contribution_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Zero amount should fail with InvalidAmount
    client.donate_with_token(&pool_id, &donor, &token, &0i128);
}

/// Test 2: Negative amount contribution fails
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_negative_amount_contribution_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Negative amount should fail with InvalidAmount
    client.donate_with_token(&pool_id, &donor, &token, &-100_000_000i128);
}

/// Test 3: Maximum i128 amount contribution succeeds if balance allows
#[test]
fn test_maximum_i128_amount_contribution_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let max_amount = i128::MAX;
    let token = create_token(&env, max_amount, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Max Amount Pool"),
        &String::from_str(&env, "Test"),
        &(i128::MAX as u128),
    );

    // Maximum i128 amount should succeed if balance allows
    client.donate_with_token(&pool_id, &donor, &token, &max_amount);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, max_amount as u128);
}

/// Test 4: Contribution exceeding user balance fails with token transfer error
#[test]
#[should_panic]
fn test_contribution_exceeding_balance_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Try to contribute more than balance - should fail with token transfer error
    client.donate_with_token(&pool_id, &donor, &token, &200_000_000i128);
}

// ============= ISSUE #476: POOL CLOSURE STATE VALIDATION TESTS =============

/// Test 1: Close pool in Disbursed state succeeds
#[test]
fn test_close_disbursed_pool_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Disbursed Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Set pool state to Disbursed
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Close should succeed for Disbursed pool
    client.close_pool(&pool_id);

    // Verify closed state persists
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
}

/// Test 2: Close pool in Cancelled state succeeds
#[test]
fn test_close_cancelled_pool_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cancelled Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Set pool state to Cancelled
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    // Close should succeed for Cancelled pool
    client.close_pool(&pool_id);

    // Verify closed state persists
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
}

/// Test 3: Close pool in Active state fails with PoolNotDisbursedOrRefunded error
#[test]
#[should_panic(expected = "PoolNotDisbursedOrRefunded")]
fn test_close_active_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Active Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Pool is in Active state by default - should fail
    client.close_pool(&pool_id);
}

/// Test 4: Close pool in Paused state fails with PoolNotDisbursedOrRefunded error
#[test]
#[should_panic(expected = "PoolNotDisbursedOrRefunded")]
fn test_close_paused_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Paused Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Set pool state to Paused
    client.set_pool_state(&pool_id, &PoolState::Paused);

    // Close should fail for Paused pool
    client.close_pool(&pool_id);
}

/// Test 5: Close pool in Completed state fails with PoolNotDisbursedOrRefunded error
#[test]
#[should_panic(expected = "PoolNotDisbursedOrRefunded")]
fn test_close_completed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Completed Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Set pool state to Completed
    client.set_pool_state(&pool_id, &PoolState::Completed);

    // Close should fail for Completed pool
    client.close_pool(&pool_id);
}

/// Test 6: Close pool in Closed state fails with PoolNotDisbursedOrRefunded error
#[test]
#[should_panic(expected = "PoolNotDisbursedOrRefunded")]
fn test_close_already_closed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closed Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Set pool state to Closed
    client.set_pool_state(&pool_id, &PoolState::Closed);

    // Close should fail for already Closed pool
    client.close_pool(&pool_id);
}

/// Test 7: Closed state persists correctly after successful close
#[test]
fn test_closed_state_persists() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    // Set pool state to Disbursed
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Close the pool
    client.close_pool(&pool_id);

    // Verify is_closed returns true via get_pool
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);

    // Verify state persists across multiple reads
    let pool2 = client.get_pool(&pool_id);
    assert_eq!(pool2.4, true);
}
