#![cfg(test)]

//! Issue #1456 — `setup_application_milestones` stores a Vec keyed by
//! (pool_id, student) and validates sum == pool.goal, but `claim_funds`
//! never reads the milestone storage key. Milestones do not gate claims.
//!
//! Issue #1661 — milestones are now per-student: they must sum to the
//! student's `approved_amount`, each milestone must be school-released, and
//! `claim_funds` is capped at the released milestone total. These tests
//! assert the new gated semantics.

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

fn approved(env: &Env) -> String {
    String::from_str(env, "Approved")
}

// =============================================================================
// Test 1 — claims are rejected until milestones are school-released
// =============================================================================

/// Issue #1661.
///
/// A student with milestones configured but none released cannot claim:
/// the released milestone total is zero, so `claim_funds` is gated.
#[test]
fn test_claim_rejected_without_released_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token = create_token(&env, 2_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1661 No Release"),
        &String::from_str(&env, "Claim gated by released milestones"),
        &1_000_000u128,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &creator, &token, &500_000i128);
    client.set_application_status(&pool_id, &student, &approved(&env));
    client.approve_application_amount(&pool_id, &student, &400_000i128);

    // Milestones sum to the student's approved amount (400k), not the goal.
    let milestones = vec![
        &env,
        Milestone { amount: 200_000u128 },
        Milestone { amount: 200_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);

    // Nothing released yet — claim must be rejected.
    let result = client.try_claim_funds(&student, &pool_id, &100_000i128, &token);
    assert!(result.is_err());

    let claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed, 0);
}

// =============================================================================
// Test 2 — claims are capped at the released milestone total
// =============================================================================

/// Issue #1661.
///
/// Milestones total to the approved amount (300k + 300k = 600k). After the
/// school releases only the first milestone, the student may claim up to
/// 300k but no more.
#[test]
fn test_claim_capped_at_released_total() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token = create_token(&env, 2_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1661 Released Cap"),
        &String::from_str(&env, "Claim capped at released total"),
        &900_000u128,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &creator, &token, &500_000i128);
    client.set_application_status(&pool_id, &student, &approved(&env));
    client.approve_application_amount(&pool_id, &student, &600_000i128);

    let milestones = vec![
        &env,
        Milestone { amount: 300_000u128 },
        Milestone { amount: 300_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);

    // Release only the first milestone.
    client.release_milestone(&pool_id, &student, &0u32);

    // Claiming more than the released total is rejected.
    let over = client.try_claim_funds(&student, &pool_id, &400_000i128, &token);
    assert!(over.is_err());

    // Claiming within the released total succeeds.
    client.claim_funds(&student, &pool_id, &300_000i128, &token);
    let claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed, 300_000);

    // Released total is exhausted — further claims are rejected.
    let again = client.try_claim_funds(&student, &pool_id, &1i128, &token);
    assert!(again.is_err());

    // Release the second milestone; the remaining 300k becomes claimable.
    client.release_milestone(&pool_id, &student, &1u32);
    client.claim_funds(&student, &pool_id, &300_000i128, &token);
    let claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed, 600_000);
}

// =============================================================================
// Test 3 — milestones must sum to the student's approved amount
// =============================================================================

/// Issue #1661.
///
/// Milestones summing to the pool goal (but not the approved amount) are
/// rejected; milestones summing to the approved amount are accepted.
#[test]
fn test_milestones_must_sum_to_approved_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token = create_token(&env, 2_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1661 Approved Sum"),
        &String::from_str(&env, "Milestones sum to approved amount"),
        &600_000u128,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &creator, &token, &500_000i128);
    client.set_application_status(&pool_id, &student, &approved(&env));
    client.approve_application_amount(&pool_id, &student, &400_000i128);

    // Sums to the goal (600k), not the approved amount (400k) — rejected.
    let wrong = vec![
        &env,
        Milestone { amount: 300_000u128 },
        Milestone { amount: 300_000u128 },
    ];
    let bad = client.try_setup_application_milestones(&pool_id, &student, &wrong);
    assert!(bad.is_err());

    // Sums to the approved amount — accepted.
    let right = vec![
        &env,
        Milestone { amount: 200_000u128 },
        Milestone { amount: 200_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student, &right);

    let stored = client.get_milestones(&pool_id, &student);
    assert_eq!(stored.len(), 2);
    assert_eq!(stored.get(0).unwrap().amount, 200_000);
    assert_eq!(stored.get(1).unwrap().amount, 200_000);
}

// =============================================================================
// Test 4 — integration: two students share one pool
// =============================================================================

/// Issue #1661.
///
/// Two students share a single pool. Each has its own approved amount and
/// milestone schedule summing to that amount. Releasing and claiming for one
/// student does not affect the other, and both can claim their full approved
/// amounts from the shared pool.
#[test]
fn test_two_students_share_one_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student_a = Address::generate(&env);
    let student_b = Address::generate(&env);
    let token = create_token(&env, 2_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1661 Shared Pool"),
        &String::from_str(&env, "Two students share one pool"),
        &1_000_000u128,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &creator, &token, &1_000_000i128);

    // Apply and approve distinct amounts for each student.
    client.set_application_status(&pool_id, &student_a, &approved(&env));
    client.approve_application_amount(&pool_id, &student_a, &400_000i128);
    client.set_application_status(&pool_id, &student_b, &approved(&env));
    client.approve_application_amount(&pool_id, &student_b, &600_000i128);

    // Per-student milestone schedules sum to each approved amount.
    let milestones_a = vec![
        &env,
        Milestone { amount: 200_000u128 },
        Milestone { amount: 200_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student_a, &milestones_a);

    let milestones_b = vec![
        &env,
        Milestone { amount: 300_000u128 },
        Milestone { amount: 300_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student_b, &milestones_b);

    // Student A: release first milestone, claim it.
    client.release_milestone(&pool_id, &student_a, &0u32);
    client.claim_funds(&student_a, &pool_id, &200_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student_a), 200_000);

    // Student B is unaffected: still gated until its own release.
    let b_blocked = client.try_claim_funds(&student_b, &pool_id, &100_000i128, &token);
    assert!(b_blocked.is_err());

    // Student A: release and claim the second milestone.
    client.release_milestone(&pool_id, &student_a, &1u32);
    client.claim_funds(&student_a, &pool_id, &200_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student_a), 400_000);

    // Student B: release both milestones and claim the full approved amount.
    client.release_milestone(&pool_id, &student_b, &0u32);
    client.release_milestone(&pool_id, &student_b, &1u32);
    client.claim_funds(&student_b, &pool_id, &600_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student_b), 600_000);
}