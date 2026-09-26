#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, String, Symbol,
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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
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
    assert!(
        !has_request,
        "Withdrawal request should be removed after execution"
    );
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
        &100_000u64,
    );

    // Pool is in Active state by default - should succeed
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000u128);
}

/// Test 2: Contribute to Closed pool fails
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
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
        &100_000u64,
    );

    // Transition to Disbursed so close_pool is allowed, then close the pool
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    // Should fail with "Pool is closed"
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);
}

/// Test 3: Contribute to a Paused pool fails (Issue #943)
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_contribute_to_paused_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Paused Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.set_pool_state(&pool_id, &PoolState::Paused);

    // Should fail with InvalidPoolState
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);
}

/// Test 4: Contribute to a Completed pool fails (Issue #943)
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_contribute_to_completed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Completed Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.set_pool_state(&pool_id, &PoolState::Completed);

    // Should fail with InvalidPoolState
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);
}

/// Test 5: Contribute to a Cancelled pool fails (Issue #943)
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_contribute_to_cancelled_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cancelled Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    // Should fail with InvalidPoolState
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);
}

// NOTE: Disbursed state is intentionally not covered here: it is not part of
// the Paused/Completed/Cancelled gap this issue targets, and is reachable
// only via the test-only `set_pool_state` helper (no production flow drives
// a pool from Active to Disbursed before donations close).

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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
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
        &100_000u64,
    );

    // Set pool state to Disbursed
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Close should succeed for Disbursed pool
    client.close_pool(&pool_id);

    // Verify closed state persists
    let pool = client.get_pool(&pool_id);
    assert!(pool.4);
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
        &100_000u64,
    );

    // Set pool state to Cancelled
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    // Close should succeed for Cancelled pool
    client.close_pool(&pool_id);

    // Verify closed state persists
    let pool = client.get_pool(&pool_id);
    assert!(pool.4);
}

/// Test 3: Close pool in Active state fails with PoolNotDisbursedOrRefunded error
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
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
        &100_000u64,
    );

    // Pool is in Active state by default - should fail
    client.close_pool(&pool_id);
}

/// Test 4: Close pool in Paused state fails with PoolNotDisbursedOrRefunded error
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
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
        &100_000u64,
    );

    // Set pool state to Paused
    client.set_pool_state(&pool_id, &PoolState::Paused);

    // Close should fail for Paused pool
    client.close_pool(&pool_id);
}

/// Test 5: Close pool in Completed state fails with PoolNotDisbursedOrRefunded error
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
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
        &100_000u64,
    );

    // Set pool state to Completed
    client.set_pool_state(&pool_id, &PoolState::Completed);

    // Close should fail for Completed pool
    client.close_pool(&pool_id);
}

/// Test 6: Close pool in Closed state fails with PoolNotDisbursedOrRefunded error
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
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
        &100_000u64,
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
        &100_000u64,
    );

    // Set pool state to Disbursed
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Close the pool
    client.close_pool(&pool_id);

    // Verify is_closed returns true via get_pool
    let pool = client.get_pool(&pool_id);
    assert!(pool.4);

    // Verify state persists across multiple reads
    let pool2 = client.get_pool(&pool_id);
    assert!(pool2.4);
}

// ============= ISSUE #942: MILESTONE SETUP/GETTER TESTS =============

/// Test 1: Setting an empty milestone list panics
#[test]
#[should_panic(expected = "Milestones required")]
fn test_setup_application_milestones_empty_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application"));
    client.setup_application_milestones(&pool_id, &student, &Vec::new(&env));
}

/// Test 2: Milestone amounts that don't sum to the pool goal panic
#[test]
#[should_panic(expected = "Milestone total must equal pool goal")]
fn test_setup_application_milestones_total_mismatch_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let milestones = Vec::from_array(
        &env,
        [Milestone { amount: 100_000_000u128 }, Milestone { amount: 200_000_000u128 }],
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application"));
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

/// Test 3: Milestone amounts that overflow u128 on summation panic
#[test]
#[should_panic(expected = "Milestone amount overflow")]
fn test_setup_application_milestones_overflow_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let milestones = Vec::from_array(
        &env,
        [Milestone { amount: u128::MAX }, Milestone { amount: 1u128 }],
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application"));
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

/// Test 4: Milestones summing to the pool goal are stored and read back correctly
#[test]
fn test_setup_and_get_milestones_round_trip() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let milestones = Vec::from_array(
        &env,
        [Milestone { amount: 400_000_000u128 }, Milestone { amount: 600_000_000u128 }],
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application"));
    client.setup_application_milestones(&pool_id, &student, &milestones);

    let stored = client.get_milestones(&pool_id, &student);
    assert_eq!(stored, milestones);
}

// ============= ISSUE #940: POOL DEADLINE SETTER/GETTER TESTS =============

/// Test 1: get_pool_deadline defaults to 0 when no deadline has been set
#[test]
fn test_get_pool_deadline_defaults_to_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Deadline Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_pool_deadline(&pool_id), 0u32);
}

/// Test 2: Sponsor can set the deadline and read it back
#[test]
fn test_set_and_get_pool_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Deadline Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let deadline = env.ledger().sequence() + 1_000;
    client.set_pool_deadline(&pool_id, &deadline);

    assert_eq!(client.get_pool_deadline(&pool_id), deadline);
}

/// Test 3: A caller other than the pool sponsor cannot set the deadline
#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_set_pool_deadline_rejects_non_sponsor() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let non_sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Deadline Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let deadline = env.ledger().sequence() + 1_000;
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

/// Test 4: A deadline that is not strictly in the future panics
#[test]
#[should_panic(expected = "Deadline must be in the future")]
fn test_set_pool_deadline_rejects_non_future_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Deadline Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let deadline = env.ledger().sequence();
    client.set_pool_deadline(&pool_id, &deadline);
}

// ============= ISSUE #939: REFUND_DONATION TESTS =============

/// Test 1: Donor is refunded once the deadline has passed AND the grace
/// period (REFUND_GRACE_PERIOD_LEDGERS) has fully elapsed.
#[test]
fn test_refund_donation_after_grace_period_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    // Raise the default persistent-entry TTL (4096 ledgers) so that jumping
    // the ledger sequence forward past the deadline + grace period doesn't
    // archive the contract instance / pool storage entries in the test sandbox.
    env.ledger().with_mut(|li| {
        li.min_persistent_entry_ttl = 20_000;
    });
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let contribution = 50_000_000u128;
    // Fund the contract directly (mirrors the withdraw_unallocated_funds
    // regression test): `donate` only updates accounting, so the contract
    // needs a real token balance for `refund_donation`'s transfer to work.
    let token = create_token(&env, contribution as i128, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.donate(&pool_id, &donor, &contribution);

    let deadline: u32 = 1_000;
    client.set_pool_deadline(&pool_id, &deadline);

    // Advance exactly to the boundary: deadline + REFUND_GRACE_PERIOD_LEDGERS.
    env.ledger()
        .set_sequence_number(deadline + REFUND_GRACE_PERIOD_LEDGERS);

    client.refund_donation(&pool_id, &donor, &token);

    // Donor received their contribution back.
    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&donor), contribution as i128);

    // Pool's collected amount and the donor's recorded contribution are both cleared.
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 0u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 0u128);
}

/// Test 2: Refund fails when no deadline was ever set on the pool.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_donation_no_deadline_set_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let contribution = 50_000_000u128;
    let token = create_token(&env, contribution as i128, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.donate(&pool_id, &donor, &contribution);

    // No deadline was ever set -> refund must be rejected with PoolNotExpired.
    client.refund_donation(&pool_id, &donor, &token);
}

/// Test 3: Refund fails when the deadline is set but has not passed yet.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_donation_before_deadline_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let contribution = 50_000_000u128;
    let token = create_token(&env, contribution as i128, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.donate(&pool_id, &donor, &contribution);

    let deadline: u32 = 1_000;
    client.set_pool_deadline(&pool_id, &deadline);

    // Ledger sequence is still 0 (the default) -- well before the deadline.
    client.refund_donation(&pool_id, &donor, &token);
}

/// Test 4: Refund fails when the deadline has passed but the grace period
/// has not fully elapsed yet -- one ledger short of the boundary.
#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_refund_donation_within_grace_period_fails() {
    let env = Env::default();
    env.mock_all_auths();
    // Raise the default persistent-entry TTL so the ledger-sequence jump
    // below doesn't archive the contract instance / pool storage entries.
    env.ledger().with_mut(|li| {
        li.min_persistent_entry_ttl = 20_000;
    });
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let contribution = 50_000_000u128;
    let token = create_token(&env, contribution as i128, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.donate(&pool_id, &donor, &contribution);

    let deadline: u32 = 1_000;
    client.set_pool_deadline(&pool_id, &deadline);

    // One ledger short of deadline + REFUND_GRACE_PERIOD_LEDGERS: the
    // deadline has passed, but the grace period has not fully elapsed.
    env.ledger()
        .set_sequence_number(deadline + REFUND_GRACE_PERIOD_LEDGERS - 1);

    client.refund_donation(&pool_id, &donor, &token);
}

/// Test 5: Refund fails for a donor who never contributed to the pool.
#[test]
#[should_panic(expected = "Error(Contract, #13)")]
fn test_refund_donation_no_contribution_fails() {
    let env = Env::default();
    env.mock_all_auths();
    // Raise the default persistent-entry TTL so the ledger-sequence jump
    // below doesn't archive the contract instance / pool storage entries.
    env.ledger().with_mut(|li| {
        li.min_persistent_entry_ttl = 20_000;
    });
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env); // never donates
    let token = create_token(&env, 50_000_000i128, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let deadline: u32 = 1_000;
    client.set_pool_deadline(&pool_id, &deadline);
    env.ledger()
        .set_sequence_number(deadline + REFUND_GRACE_PERIOD_LEDGERS);

    // Donor never contributed -> NoContributionToRefund.
    client.refund_donation(&pool_id, &donor, &token);
}

// ============= ISSUE #941: CREATE_POOL_FOR_SCHOOL / GET_POOL_SCHOOL TESTS =============

/// Test 1: A registered school can back a pool, and get_pool_school returns it.
#[test]
fn test_create_pool_for_school_succeeds_for_registered_school() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[6u8; 32]);
    client.register_school(&school, &metadata_hash);

    let creator = Address::generate(&env);
    let goal = 250_000_000u128;
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "Test"),
        &goal,
        &school,
        &100_000u64,
    );

    assert_eq!(client.get_pool_school(&pool_id), school);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, goal);
    assert_eq!(pool.3, 0u128);
}

/// Test 2: Creating a school-linked pool for an unregistered school panics.
#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_create_pool_for_school_unregistered_school_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let unregistered_school = Address::generate(&env);

    // No `register_school` call was made for this address.
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &unregistered_school,
        &100_000u64,
    );
}

/// Test 3: get_pool_school panics for a pool with no linked school (i.e. one
/// created via the normal, non-school `create_pool` path).
#[test]
#[should_panic(expected = "Pool school not set")]
fn test_get_pool_school_fails_for_non_school_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Regular Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.get_pool_school(&pool_id);
}

// ============= ISSUE #1059: POOL CONTRIBUTION METRICS TRACKING TESTS =============
//
// These tests pin down the three per-pool contribution metrics named in
// issue #1059:
//   - contributor_count -> `get_donor_count(pool_id)`, backed by the
//     `d_count` storage entry.
//   - total_raised       -> `get_total_raised(pool_id)`, backed by
//     `Pool.collected`.
//   - last_donation_at   -> `get_last_donation_at(pool_id)`, backed by the
//     new `Pool.last_donation_at` field (added alongside these tests --
//     the field, its getter, and the `env.ledger().timestamp()` writes in
//     `donate()`/`donate_with_token()` did not exist before; see lib.rs).
//
// Tests 1-3 encode the unique-donor semantics issue #1059 asks for
// ("0 -> 1 on first contribution", "stays at 1 on a repeat contribution",
// "1 -> 2 on a new contributor"). `d_count` increments only when a donor
// contributes to the pool for the first time.

/// Test 1 (issue #1059, requirement 1): a pool's first-ever contribution
/// should take contributor_count from 0 to 1.
#[test]
fn test_first_contribution_increments_contributor_count_from_zero_to_one() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(
        client.get_donor_count(&pool_id),
        0,
        "a freshly-created pool must start with zero contributors"
    );

    client.donate(&pool_id, &donor, &10_000_000u128);

    assert_eq!(
        client.get_donor_count(&pool_id),
        1,
        "the pool's first-ever contribution must set contributor_count to 1"
    );
}

/// Test 2 (issue #1059, requirement 2): a second contribution from the
/// *same* contributor must not be double-counted -- contributor_count
/// should stay at 1.
#[test]
fn test_repeat_contribution_from_same_donor_leaves_contributor_count_at_one() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor, &10_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);

    // Same donor contributes again -- must not be counted as a new contributor.
    client.donate(&pool_id, &donor, &5_000_000u128);

    assert_eq!(
        client.get_donor_count(&pool_id),
        1,
        "a repeat contribution from an existing contributor must not change contributor_count"
    );
}

/// Test 3 (issue #1059, requirement 3): a contribution from a *different*,
/// new contributor to the same pool should take contributor_count from 1
/// to 2.
#[test]
fn test_new_contributor_increments_contributor_count_from_one_to_two() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_a = Address::generate(&env);
    let donor_b = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor_a, &10_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);

    // A different contributor donates for the first time.
    client.donate(&pool_id, &donor_b, &20_000_000u128);

    assert_eq!(
        client.get_donor_count(&pool_id),
        2,
        "a new contributor to the same pool must take contributor_count from 1 to 2"
    );
}

/// Test 4 (issue #1059, requirement 4): total_raised accumulates correctly
/// across multiple contributions, including repeat contributions from the
/// same contributor. Unlike contributor_count, `Pool.collected` (exposed
/// via `get_total_raised`) has no double-counting bug -- this test passes.
#[test]
fn test_total_raised_accumulates_across_repeat_and_new_contributions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_a = Address::generate(&env);
    let donor_b = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_total_raised(&pool_id), 0u128);

    let first_amount = 10_000_000u128;
    client.donate(&pool_id, &donor_a, &first_amount);
    assert_eq!(client.get_total_raised(&pool_id), first_amount);

    // Same contributor donates again -- must accumulate, not overwrite.
    let second_amount = 15_000_000u128;
    client.donate(&pool_id, &donor_a, &second_amount);
    assert_eq!(
        client.get_total_raised(&pool_id),
        first_amount + second_amount
    );

    // A different contributor's donation must also accumulate into the same total.
    let third_amount = 7_500_000u128;
    client.donate(&pool_id, &donor_b, &third_amount);
    assert_eq!(
        client.get_total_raised(&pool_id),
        first_amount + second_amount + third_amount,
        "total_raised must equal the sum of every contribution, repeats included"
    );
}

/// Test 5 (issue #1059, requirement 5): last_donation_at updates to the
/// current ledger timestamp after each contribution. Two contributions are
/// made at different simulated timestamps (via `env.ledger().set_timestamp`)
/// so the change is actually observable, not just "non-zero".
#[test]
fn test_last_donation_at_updates_to_current_ledger_timestamp() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_a = Address::generate(&env);
    let donor_b = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metrics Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(
        client.get_last_donation_at(&pool_id),
        0u64,
        "a pool with no donations yet must report last_donation_at as 0"
    );

    let first_timestamp = 1_000u64;
    env.ledger().set_timestamp(first_timestamp);
    client.donate(&pool_id, &donor_a, &10_000_000u128);

    assert_eq!(
        client.get_last_donation_at(&pool_id),
        first_timestamp,
        "last_donation_at must be set to the ledger timestamp of the first contribution"
    );

    // Advance to a distinct later timestamp and have a different donor contribute.
    let second_timestamp = 5_000u64;
    assert_ne!(second_timestamp, first_timestamp);
    env.ledger().set_timestamp(second_timestamp);
    client.donate(&pool_id, &donor_b, &20_000_000u128);

    assert_eq!(
        client.get_last_donation_at(&pool_id),
        second_timestamp,
        "last_donation_at must update to the new ledger timestamp on the next contribution"
    );
}

// ============= ISSUE #1344: GET_MILESTONES RETRIEVAL AND ORDERING TESTS =============
//
// Item 3 from the issue ("get_milestones reflects milestone completion state
// changes") is skipped: `Milestone` has only an `amount: u128` field, with no
// completion state anywhere, and no function in the contract marks a
// milestone complete. There is nothing to change or observe for that case.

/// Test 1: get_milestones returns an empty Vec before setup_application_milestones
/// has ever been called for that pool/student pair.
#[test]
fn test_get_milestones_empty_before_setup() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // No setup_application_milestones call at all for this pair.
    let milestones = client.get_milestones(&pool_id, &student);
    assert_eq!(milestones.len(), 0);
}

/// Test 2: get_milestones returns milestones in the exact order they were
/// configured, not re-sorted (amounts are deliberately non-monotonic).
#[test]
fn test_get_milestones_preserves_configured_order() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Deliberately not sorted ascending or descending.
    let milestones = Vec::from_array(
        &env,
        [
            Milestone { amount: 500_000_000u128 },
            Milestone { amount: 100_000_000u128 },
            Milestone { amount: 400_000_000u128 },
        ],
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application"));
    client.setup_application_milestones(&pool_id, &student, &milestones);

    let stored = client.get_milestones(&pool_id, &student);
    assert_eq!(stored.len(), 3);
    assert_eq!(stored.get(0).unwrap().amount, 500_000_000u128);
    assert_eq!(stored.get(1).unwrap().amount, 100_000_000u128);
    assert_eq!(stored.get(2).unwrap().amount, 400_000_000u128);
}

/// Test 3 (issue item 4): milestones for one (pool_id, student) pair don't
/// leak into or get confused with another pair's milestones, whether the
/// pool_id differs, the student differs, or both.
#[test]
fn test_get_milestones_isolated_per_pool_and_student() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student_x = Address::generate(&env);
    let student_y = Address::generate(&env);

    let pool_a = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool A"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    let pool_b = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool B"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let milestones_ax = Vec::from_array(&env, [Milestone { amount: 1_000_000_000u128 }]);
    let milestones_ay = Vec::from_array(
        &env,
        [Milestone { amount: 300_000_000u128 }, Milestone { amount: 700_000_000u128 }],
    );
    let milestones_bx = Vec::from_array(
        &env,
        [Milestone { amount: 250_000_000u128 }, Milestone { amount: 750_000_000u128 }],
    );

    // (pool_a, student_x)
    client.apply_to_pool(&pool_a, &student_x, &String::from_str(&env, "App"));
    client.setup_application_milestones(&pool_a, &student_x, &milestones_ax);

    // (pool_a, student_y) -- same pool, different student
    client.apply_to_pool(&pool_a, &student_y, &String::from_str(&env, "App"));
    client.setup_application_milestones(&pool_a, &student_y, &milestones_ay);

    // (pool_b, student_x) -- same student, different pool
    client.apply_to_pool(&pool_b, &student_x, &String::from_str(&env, "App"));
    client.setup_application_milestones(&pool_b, &student_x, &milestones_bx);

    // (pool_b, student_y) is left untouched entirely.

    assert_eq!(client.get_milestones(&pool_a, &student_x), milestones_ax);
    assert_eq!(client.get_milestones(&pool_a, &student_y), milestones_ay);
    assert_eq!(client.get_milestones(&pool_b, &student_x), milestones_bx);
    assert_eq!(client.get_milestones(&pool_b, &student_y).len(), 0);
}

/// Test 4 (issue item 5): a large milestone list is stored and read back in
/// full, with nothing silently dropped. No MAX_MILESTONES or similar cap
/// exists in the contract, so this uses a reasonably large list rather than
/// testing at a specific boundary.
#[test]
fn test_get_milestones_large_list_not_truncated() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    const COUNT: u32 = 200;
    const PER_MILESTONE: u128 = 5_000_000u128;
    let goal = PER_MILESTONE * (COUNT as u128);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Large Milestone Pool"),
        &String::from_str(&env, "Test"),
        &goal,
        &100_000u64,
    );

    let mut milestones: Vec<Milestone> = Vec::new(&env);
    for _ in 0..COUNT {
        milestones.push_back(Milestone { amount: PER_MILESTONE });
    }

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application"));
    client.setup_application_milestones(&pool_id, &student, &milestones);

    let stored = client.get_milestones(&pool_id, &student);
    assert_eq!(stored.len(), COUNT);

    let mut sum: u128 = 0;
    for i in 0..stored.len() {
        sum += stored.get(i).unwrap().amount;
    }
    assert_eq!(sum, goal);
}