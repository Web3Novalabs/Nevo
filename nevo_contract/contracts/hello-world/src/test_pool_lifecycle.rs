#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

// ============= ISSUE #1091: INTEGRATION TESTS FOR POOL LIFECYCLE =============

/// Test 1: Create pool with metadata and verify all fields
#[test]
fn test_pool_lifecycle_create_with_metadata() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Scholarship Pool");
    let description = String::from_str(&env, "University scholarship funding");
    let goal = 10_000_000_000u128;
    let deadline = 300_000u64;

    let pool_id = client.create_pool(&creator, &title, &description, &goal, &deadline);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, goal);
    assert_eq!(pool.3, 0u128);
    assert_eq!(pool.4, false);
    assert_eq!(pool.5, deadline);

    let (stored_title, stored_desc) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, title);
    assert_eq!(stored_desc, description);
}

/// Test 2: Multiple contributions to the same pool
#[test]
fn test_pool_lifecycle_multiple_contributions() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Community Pool"),
        &String::from_str(&env, "Multiple contributors"),
        &5_000_000_000u128,
        &200_000u64,
    );

    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);

    client.donate(&pool_id, &donor1, &1_000_000_000u128);
    client.donate(&pool_id, &donor2, &2_000_000_000u128);
    client.donate(&pool_id, &donor3, &500_000_000u128);

    assert_eq!(client.get_total_raised(&pool_id), 3_500_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor1), 1_000_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor2), 2_000_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor3), 500_000_000u128);
}

/// Test 3: State transitions - Active -> Paused -> Active -> Disbursed -> Closed
#[test]
fn test_pool_lifecycle_state_transitions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "State Transition Pool"),
        &String::from_str(&env, "Testing state machine"),
        &1_000_000_000u128,
        &200_000u64,
    );

    // Active: can donate
    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);

    // Transition to Paused
    client.set_pool_state(&pool_id, &PoolState::Paused);

    // Transition back to Active
    client.set_pool_state(&pool_id, &PoolState::Active);

    // Can donate again when Active
    client.donate(&pool_id, &Address::generate(&env), &200_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 300_000_000u128);

    // Transition to Disbursed
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Can close from Disbursed
    client.close_pool(&pool_id);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
}

/// Test 4: Refund preconditions - verify deadline storage and validation
///
/// NOTE: Full refund execution (refund_donation) requires advancing the ledger
/// past REFUND_GRACE_PERIOD_LEDGERS (17280), which the Soroban SDK test
/// environment cannot handle without archiving contract instance entries.
/// This test validates the deadline-setting and storage mechanics.
#[test]
fn test_pool_lifecycle_refund_preconditions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Refund Pool"),
        &String::from_str(&env, "Testing refund preconditions"),
        &1_000_000_000u128,
        &200_000u64,
    );

    // Initially no deadline set
    assert_eq!(client.get_pool_deadline(&pool_id), 0u32);

    // Set refund deadline
    client.set_pool_deadline(&pool_id, &50u32);
    assert_eq!(client.get_pool_deadline(&pool_id), 50u32);

    // Mint tokens and donate
    let donor = Address::generate(&env);
    let token = create_token(&env, 500_000_000i128, &donor);
    client.donate_with_token(&pool_id, &donor, &token, &500_000_000i128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 500_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 500_000_000u128);

    // Deadline is stored correctly
    assert_eq!(client.get_pool_deadline(&pool_id), 50u32);
}

/// Test 5: Pool closure after disbursement
#[test]
fn test_pool_lifecycle_closure_after_disbursement() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Disbursement Pool"),
        &String::from_str(&env, "Close after disbursement"),
        &1_000_000_000u128,
        &200_000u64,
    );

    client.donate(&pool_id, &Address::generate(&env), &500_000_000u128);
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);

    // Verify closed state persists across reads
    let pool2 = client.get_pool(&pool_id);
    assert_eq!(pool2.4, true);
}

/// Test 6: Pool closure after cancellation
#[test]
fn test_pool_lifecycle_closure_after_cancellation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cancelled Pool"),
        &String::from_str(&env, "Close after cancellation"),
        &1_000_000_000u128,
        &200_000u64,
    );

    client.donate(&pool_id, &Address::generate(&env), &200_000_000u128);
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    client.close_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
}

/// Test 7: Full lifecycle with token donations and state transitions
#[test]
fn test_pool_lifecycle_full_with_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Full Token Pool"),
        &String::from_str(&env, "Complete lifecycle"),
        &1_000_000_000u128,
        &200_000u64,
    );

    // Mint tokens to donors
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let token = create_token(&env, 2_000_000_000i128, &donor1);
    let sac = StellarAssetClient::new(&env, &token);
    sac.mint(&donor2, &1_000_000_000i128);

    // Donate with tokens
    client.donate_with_token(&pool_id, &donor1, &token, &600_000_000i128);
    client.donate_with_token(&pool_id, &donor2, &token, &400_000_000i128);

    assert_eq!(client.get_total_raised(&pool_id), 1_000_000_000u128);

    // Verify contributions are tracked
    assert_eq!(client.get_contribution(&pool_id, &donor1), 600_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor2), 400_000_000u128);

    // Transition to Disbursed and close
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
    assert_eq!(pool.3, 1_000_000_000u128);
}

/// Test 8: State change tracking across operations
#[test]
fn test_pool_lifecycle_state_change_tracking() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Tracking Pool"),
        &String::from_str(&env, "Track all changes"),
        &2_000_000_000u128,
        &200_000u64,
    );

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 0u128);
    assert_eq!(pool.4, false);

    client.donate(&pool_id, &Address::generate(&env), &1_000_000_000u128);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 1_000_000_000u128);
    assert_eq!(pool.4, false);

    client.donate(&pool_id, &Address::generate(&env), &500_000_000u128);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 1_500_000_000u128);
}
