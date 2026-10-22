#![cfg(test)]

//! Tests for Issue #1455 — `claim_funds` derives `approved_amount` from
//! `pool.collected` instead of any real approval.
//!
//! These tests document the current behaviour so the design gap is visible.
//! They are expected to be updated once #1455 decides which fix to apply
//! (set `approved_amount` from milestone totals, or document that the
//! collected-balance cap is intentional).

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

fn approved_status(env: &Env) -> String {
    String::from_str(env, "Approved")
}

// =============================================================================
// Test 1 — First claimant can drain the entire pool.collected balance
// =============================================================================

/// Bug #1455, part 1.
///
/// A student whose milestones total 900 XLM is approved for a pool with
/// `goal = 900` but only `collected = 500`. The first time they call
/// `claim_funds`, the contract initialises `Application.approved_amount`
/// to `pool.collected` (500), not to their milestone total (900) and not to
/// the pool goal.
///
/// This test asserts today's behaviour: they can claim the entire 500 even
/// though their approval is for 900. If the fix wires `approved_amount`
/// to the milestone total, this test should be updated to expect `500`
/// to fail (because approved_amount would be 900 and 900 > 500 collected —
/// wait, 900 >= 500 so it would still succeed; the point of the test is
/// to show that the claimant's *approval limit* has nothing to do with
/// what they can withdraw).
#[test]
fn test_first_claimant_drains_pool_up_to_collected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let admin = Address::generate(&env);

    let goal: u128 = 900_000_000;
    let collected: u128 = 500_000_000;

    // Real token with `collected` supply, held by the contract.
    let token = create_token(&env, collected as i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1455 Pool"),
        &String::from_str(&env, "Milestone cap not enforced"),
        &goal,
        &100_000u64,
    );
    client.donate(&pool_id, &creator, &collected);

    // Milestones total = goal, as `setup_application_milestones` requires.
    let milestones = vec![
        &env,
        Milestone { amount: 300_000_000u128 },
        Milestone { amount: 300_000_000u128 },
        Milestone { amount: 300_000_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);
    client.set_application_status(&pool_id, &student, &approved_status(&env));

    // Claim everything the pool holds. This succeeds today because
    // approved_amount was initialised to `collected` on first claim.
    client.claim_funds(&student, &pool_id, &(collected as i128), &token);

    let app = client.get_application(&pool_id, &student).unwrap();
    assert_eq!(app.amount_claimed, collected as i128);
    assert_eq!(app.approved_amount, collected as i128);
    // Explicitly document the gap: approved_amount != milestone total (goal).
    assert_ne!(app.approved_amount, goal as i128);
}

/// Bug #1455, part 2.
///
/// Two approved students each independently initialise their Application
/// with `approved_amount = pool.collected` on first claim. Student A claims
/// the full pool; Student B still believes the full pool is available
/// because neither `pool.collected` nor B's `approved_amount` were touched.
///
/// The token transfer for B then fails at the token contract level (the
/// vault is empty), but the contract's own overdraw check passed. That
/// is the race: the contract cannot distinguish an available pool from a
/// drained one because it never decrements `pool.collected`.
#[test]
fn test_two_approved_students_each_see_whole_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student_a = Address::generate(&env);
    let student_b = Address::generate(&env);
    let admin = Address::generate(&env);

    let goal: u128 = 900_000_000;
    let collected: u128 = 500_000_000;

    let token = create_token(&env, collected as i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1455 Competition"),
        &String::from_str(&env, "Two approved students, one pool"),
        &goal,
        &100_000u64,
    );
    client.donate(&pool_id, &creator, &collected);

    let milestones = vec![
        &env,
        Milestone { amount: 450_000_000u128 },
        Milestone { amount: 450_000_000u128 },
    ];
    client.setup_application_milestones(&pool_id, &student_a, &milestones);
    client.setup_application_milestones(&pool_id, &student_b, &milestones);
    client.set_application_status(&pool_id, &student_a, &approved_status(&env));
    client.set_application_status(&pool_id, &student_b, &approved_status(&env));

    // Student A claims the whole pool.
    client.claim_funds(&student_a, &pool_id, &(collected as i128), &token);
    let app_a = client.get_application(&pool_id, &student_a).unwrap();
    assert_eq!(app_a.amount_claimed, collected as i128);
    assert_eq!(app_a.approved_amount, collected as i128);

    // The pool's `collected` field is unchanged — the contract did not
    // decrement it on claim.
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, collected); // collected still 500M

    // Student B *could* still pass the contract-level overdraw check
    // because their approved_amount would be initialised to the same
    // un-decremented `collected`. We don't actually attempt the claim
    // because the token contract has 0 balance at the vault, which
    // would panic at the transfer step — that's a token-level failure,
    // not a contract-level rejection, which is exactly the gap.
    //
    // Assert the shape of the bug: neither student's Application has
    // been constrained to their milestones.
    assert_ne!(app_a.approved_amount, 900_000_000i128); // not goal
    assert_ne!(app_a.approved_amount, 450_000_000i128); // not milestone half
}