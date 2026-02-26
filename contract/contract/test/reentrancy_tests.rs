#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String,
};

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

// ── shared setup ──────────────────────────────────────────────────────────────

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token_id, &0);

    (client, admin, token_id)
}

/// Create a pool that expires at timestamp 1_001 (created_at=1_000, duration=1).
fn make_pool(env: &Env, client: &CrowdfundingContractClient<'_>, admin: &Address) -> u64 {
    env.ledger().with_mut(|li| li.timestamp = 1_000);

    let config = PoolConfig {
        name: String::from_str(env, "Reentrancy Test Pool"),
        description: String::from_str(env, "pool for reentrancy tests"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 1, // deadline = created_at + duration = 1_001
        created_at: 1_000,
    };

    client.create_pool(admin, &config)
}

/// Advance ledger past the 7-day grace period so refunds are eligible.
/// deadline = 1_001, grace = 604_800 → open at timestamp 605_802
fn advance_past_grace(env: &Env) {
    env.ledger()
        .with_mut(|li| li.timestamp = 1_001 + 604_800 + 1);
}

// ── Test 1: lock is released cleanly after every call ────────────────────────
//
// Proof: if the reentrancy lock were left set after a call, the second call
// would return Unauthorized instead of NoContributionToRefund.

#[test]
fn test_lock_released_after_each_call() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);
    let pool_id = make_pool(&env, &client, &admin);
    advance_past_grace(&env);

    let contributor = Address::generate(&env);

    // First call: lock is clear → reaches business logic → NoContributionToRefund
    let first = client.try_refund(&pool_id, &contributor);
    assert_eq!(
        first,
        Err(Ok(CrowdfundingError::NoContributionToRefund)),
        "expected business-logic error on first call"
    );

    // Second call: if the lock were stuck, we'd get Unauthorized here
    let second = client.try_refund(&pool_id, &contributor);
    assert_eq!(
        second,
        Err(Ok(CrowdfundingError::NoContributionToRefund)),
        "Unauthorized here means the reentrancy lock was not released after the first call"
    );
}

// ── Test 2: double-withdrawal is prevented by CEI pattern ─────────────────────
//
// The contribution record is zeroed BEFORE the token transfer.
// A second refund call therefore finds amount == 0 and is rejected.

#[test]
fn test_double_withdrawal_prevented() {
    let env = Env::default();
    let (client, admin, token_id) = setup_test(&env);
    let pool_id = make_pool(&env, &client, &admin);

    // Fund contributor and contribute while pool is active
    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
    token_admin_client.mint(&contributor, &500_000);

    env.ledger().with_mut(|li| li.timestamp = 1_000);
    client.contribute(&pool_id, &contributor, &token_id, &500_000, &false);

    advance_past_grace(&env);

    // First refund: succeeds and zeroes the on-chain balance
    let first = client.try_refund(&pool_id, &contributor);
    assert_eq!(first, Ok(Ok(())), "first refund must succeed");

    // Second refund: balance is 0 → rejected (double withdrawal blocked)
    let second = client.try_refund(&pool_id, &contributor);
    assert_eq!(
        second,
        Err(Ok(CrowdfundingError::NoContributionToRefund)),
        "second refund must be blocked — double withdrawal prevention"
    );
}

// ── Test 3: multiple independent contributors can each refund once ─────────────

#[test]
fn test_independent_contributors_refund_independently() {
    let env = Env::default();
    let (client, admin, token_id) = setup_test(&env);
    let pool_id = make_pool(&env, &client, &admin);

    let contributor_a = Address::generate(&env);
    let contributor_b = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
    token_admin_client.mint(&contributor_a, &200_000);
    token_admin_client.mint(&contributor_b, &300_000);

    env.ledger().with_mut(|li| li.timestamp = 1_000);
    client.contribute(&pool_id, &contributor_a, &token_id, &200_000, &false);
    client.contribute(&pool_id, &contributor_b, &token_id, &300_000, &false);

    advance_past_grace(&env);

    // A refunds — must not affect B
    assert_eq!(client.try_refund(&pool_id, &contributor_a), Ok(Ok(())));
    // A cannot refund twice
    assert_eq!(
        client.try_refund(&pool_id, &contributor_a),
        Err(Ok(CrowdfundingError::NoContributionToRefund))
    );

    // B refunds successfully even after A already refunded
    assert_eq!(client.try_refund(&pool_id, &contributor_b), Ok(Ok(())));
    // B cannot refund twice
    assert_eq!(
        client.try_refund(&pool_id, &contributor_b),
        Err(Ok(CrowdfundingError::NoContributionToRefund))
    );
}

// ── Test 4: refund blocked when contract is paused ────────────────────────────

#[test]
fn test_refund_blocked_when_paused() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);
    let pool_id = make_pool(&env, &client, &admin);
    advance_past_grace(&env);

    client.pause();

    let contributor = Address::generate(&env);
    assert_eq!(
        client.try_refund(&pool_id, &contributor),
        Err(Ok(CrowdfundingError::ContractPaused))
    );
}

// ── Test 5: refund rejected before the pool deadline ─────────────────────────

#[test]
fn test_refund_rejected_before_deadline() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);
    let pool_id = make_pool(&env, &client, &admin);
    // pool created at t=1_000 with duration=1 → deadline=1_001
    // ledger stays at 1_000 — pool has not yet expired
    assert_eq!(
        client.try_refund(&pool_id, &Address::generate(&env)),
        Err(Ok(CrowdfundingError::PoolNotExpired))
    );
}

// ── Test 6: refund rejected while inside the grace period ─────────────────────

#[test]
fn test_refund_rejected_inside_grace_period() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);
    let pool_id = make_pool(&env, &client, &admin);

    // Past deadline (1_001) but still one second inside the 7-day grace window
    env.ledger()
        .with_mut(|li| li.timestamp = 1_001 + 604_800 - 1);

    assert_eq!(
        client.try_refund(&pool_id, &Address::generate(&env)),
        Err(Ok(CrowdfundingError::RefundGracePeriodNotPassed))
    );
}

// ── Test 7: refund rejected on a disbursed pool ───────────────────────────────

#[test]
fn test_refund_rejected_on_disbursed_pool() {
    let env = Env::default();
    let (client, admin, token_id) = setup_test(&env);
    let pool_id = make_pool(&env, &client, &admin);

    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
    token_admin_client.mint(&contributor, &100_000);

    env.ledger().with_mut(|li| li.timestamp = 1_000);
    client.contribute(&pool_id, &contributor, &token_id, &100_000, &false);

    use crate::base::types::PoolState;
    client.update_pool_state(&pool_id, &PoolState::Disbursed);

    advance_past_grace(&env);

    assert_eq!(
        client.try_refund(&pool_id, &contributor),
        Err(Ok(CrowdfundingError::PoolAlreadyDisbursed))
    );
}

// ── Test 8: emergency withdrawal cannot be executed twice ─────────────────────
//
// The withdrawal record is deleted on first execute.
// A second execute call finds nothing and returns EmergencyWithdrawalNotRequested.

#[test]
fn test_emergency_withdraw_cannot_execute_twice() {
    let env = Env::default();
    let (client, _admin, token_id) = setup_test(&env);

    // Mint tokens into the contract so the withdrawal has funds
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id);
    token_admin_client.mint(&client.address, &1_000_000);

    env.ledger().with_mut(|li| li.timestamp = 0);
    client.request_emergency_withdraw(&token_id, &100_000);

    // Advance past the mandatory 24-hour waiting period
    env.ledger().with_mut(|li| li.timestamp = 86_401);

    let first = client.try_execute_emergency_withdraw();
    assert_eq!(first, Ok(Ok(())), "first execute must succeed");

    // Record was deleted — second execute must fail
    let second = client.try_execute_emergency_withdraw();
    assert_eq!(
        second,
        Err(Ok(CrowdfundingError::EmergencyWithdrawalNotRequested)),
        "second emergency withdrawal must be blocked"
    );
}

// ── Test 9: emergency withdrawal cannot be requested twice ────────────────────

#[test]
fn test_emergency_withdraw_cannot_request_twice() {
    let env = Env::default();
    let (client, _admin, token_id) = setup_test(&env);

    client.request_emergency_withdraw(&token_id, &100_000);

    let second = client.try_request_emergency_withdraw(&token_id, &100_000);
    assert_eq!(
        second,
        Err(Ok(CrowdfundingError::EmergencyWithdrawalAlreadyRequested))
    );
}

// ── Test 10: emergency withdrawal blocked before grace period ends ─────────────

#[test]
fn test_emergency_withdraw_blocked_before_grace_period() {
    let env = Env::default();
    let (client, _admin, token_id) = setup_test(&env);

    env.ledger().with_mut(|li| li.timestamp = 0);
    client.request_emergency_withdraw(&token_id, &100_000);

    // One second before the 24-hour window closes
    env.ledger().with_mut(|li| li.timestamp = 86_399);

    assert_eq!(
        client.try_execute_emergency_withdraw(),
        Err(Ok(CrowdfundingError::EmergencyWithdrawalPeriodNotPassed))
    );
}
