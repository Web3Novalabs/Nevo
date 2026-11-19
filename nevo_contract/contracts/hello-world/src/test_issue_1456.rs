#![cfg(test)]

//! Issue #1456 — `setup_application_milestones` stores a Vec keyed by
//! (pool_id, student) and validates sum == pool.goal, but `claim_funds`
//! never reads the milestone storage key. Milestones do not gate claims.

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
// Test 1 — claim succeeds identically with or without milestones
// =============================================================================

/// Bug #1456, part 1.
///
/// Two pools, same donation amount. Student A has milestones configured,
/// Student B does not. Both can claim the same amount — milestones have
/// no effect on `claim_funds`.
#[test]
fn test_claim_succeeds_without_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student_no_milestones = Address::generate(&env);
    let token = create_token(&env, 2_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1456 No Milestones"),
        &String::from_str(&env, "Claim without setup_application_milestones"),
        &1_000_000u128,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &creator, &token, &500_000i128);
    client.set_application_status(
        &pool_id,
        &student_no_milestones,
        &approved(&env),
    );

    // No setup_application_milestones call. Claim still works.
    client.claim_funds(&student_no_milestones, &pool_id, &100_000i128, &token);

    let claimed = client.get_claimed_amount(&pool_id, &student_no_milestones);
    assert_eq!(claimed, 100_000);
}

// =============================================================================
// Test 2 — claim amount is not bounded by individual milestone boundaries
// =============================================================================

/// Bug #1456, part 2.
///
/// Milestones total to the pool goal (300k + 300k + 300k = 900k). The student
/// claims 500k in a single call — larger than any single milestone (300k).
/// `claim_funds` accepts it because it only checks against `collected`,
/// not against milestone boundaries.
#[test]
fn test_claim_amount_not_bounded_by_individual_milestones() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token = create_token(&env, 2_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1456 Milestone Boundaries"),
        &String::from_str(&env, "Claim exceeds each milestone"),
        &900_000u128,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &creator, &token, &500_000i128);
    client.set_application_status(&pool_id, &student, &approved(&env));

    // Milestones total = goal (900k), 300k each.
    let milestones = vec![
        &env,
        Milestone { amount: 300_000u128 },
        Milestone { amount: 300_000u128 },
        Milestone { amount: 300_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);

    // Confirm milestones were stored.
    let stored = client.get_milestones(&pool_id, &student);
    assert_eq!(stored.len(), 3);

    // First claim is 500k — larger than any single 300k milestone.
    // Succeeds because claim_funds never reads milestone boundaries.
    client.claim_funds(&student, &pool_id, &500_000i128, &token);

    let claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed, 500_000);
}

// =============================================================================
// Test 3 — milestones survive the claim unchanged (they're never consumed)
// =============================================================================

/// Bug #1456, part 3.
///
/// If milestone-gated streaming existed, we'd expect the milestone record to
/// be consumed or advanced as the student claims. It isn't — the stored vec
/// is exactly what `setup_application_milestones` wrote, before and after.
#[test]
fn test_milestones_unchanged_after_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token = create_token(&env, 2_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1456 Milestones Survive"),
        &String::from_str(&env, "Milestones do not advance"),
        &600_000u128,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &creator, &token, &500_000i128);
    client.set_application_status(&pool_id, &student, &approved(&env));

    let milestones = vec![
        &env,
        Milestone { amount: 300_000u128 },
        Milestone { amount: 300_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);

    let before = client.get_milestones(&pool_id, &student);
    assert_eq!(before.len(), 2);
    assert_eq!(before.get(0).unwrap().amount, 300_000);

    client.claim_funds(&student, &pool_id, &250_000i128, &token);

    let after = client.get_milestones(&pool_id, &student);
    assert_eq!(after.len(), 2);
    assert_eq!(after.get(0).unwrap().amount, 300_000);
    assert_eq!(after.get(1).unwrap().amount, 300_000);
}