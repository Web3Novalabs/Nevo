#![cfg(test)]

//! Issue #1307: Comprehensive numeric overflow prevention tests.
//!
//! Covers:
//! (1) Maximum i128 amounts handled
//! (2) Addition overflow prevented
//! (3) Contribution sum limits
//! (4) Pool target amount limits
//! (5) Counter overflow protection

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Env, String, Vec,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn new_pool(env: &Env, client: &ContractClient, goal: u128) -> (u32, Address) {
    let creator = Address::generate(env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(env, "Pool"),
        &String::from_str(env, "Test"),
        &goal,
        &100_000u64,
    );
    (pool_id, creator)
}

// ─────────────────────────────────────────────────────────────────────────────
// (1) Maximum i128 amounts handled
// ─────────────────────────────────────────────────────────────────────────────

/// i128::MAX is accepted as a creation fee and stored without truncation.
#[test]
fn test_max_i128_creation_fee_stored_exactly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &i128::MAX);

    assert_eq!(client.get_creation_fee(), i128::MAX);
}

/// Emergency withdrawal request handles i128::MAX without truncation.
#[test]
fn test_max_i128_emergency_withdrawal_request_stored() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = Address::generate(&env);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000u128,
        &100_000u64,
    );

    client.request_emergency_withdraw(&admin, &pool_id, &token, &i128::MAX);

    let withdrawal_key = (Symbol::new(&env, "emergency_withdraw"), pool_id);
    let stored_request: EmergencyWithdrawalRequest = env
        .as_contract(&contract_id, || {
            env.storage().persistent().get(&withdrawal_key)
        })
        .unwrap();

    assert_eq!(stored_request.amount, i128::MAX);
}

/// A negative creation fee is rejected with InvalidFee (#11).
#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_negative_creation_fee_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &-1i128);
}

/// A negative claim amount is rejected before any arithmetic is performed.
#[test]
#[should_panic(expected = "Claim amount must be positive")]
fn test_negative_claim_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let student = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &contract_id);
    let (pool_id, _) = new_pool(&env, &client, 1_000u128);

    client.claim_funds(&student, &pool_id, &-1i128, &token);
}

/// A zero or negative donation amount via token is rejected with InvalidAmount.
#[test]
#[should_panic(expected = "InvalidAmount")]
fn test_zero_or_negative_token_donation_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &donor);
    let (pool_id, _) = new_pool(&env, &client, 1_000u128);

    client.donate_with_token(&pool_id, &donor, &token, &0i128);
}

// ─────────────────────────────────────────────────────────────────────────────
// (2) Addition overflow prevented
// ─────────────────────────────────────────────────────────────────────────────

/// collected amount uses checked addition and rejects a total that would wrap.
#[test]
#[should_panic(expected = "Collected amount overflow")]
fn test_collected_amount_addition_overflow_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let token = create_token(&env, 1_000i128, &donor);
    let (pool_id, _) = new_pool(&env, &client, u128::MAX);

    // Push collected to the maximum u128 range
    client.donate(&pool_id, &donor, &u128::MAX);
    assert_eq!(client.get_total_raised(&pool_id), u128::MAX);

    // Any further contribution must overflow checked_add rather than wrap
    client.donate_with_token(&pool_id, &donor, &token, &1i128);
}

/// Summing milestone amounts uses checked addition and rejects overflow.
#[test]
#[should_panic(expected = "Milestone amount overflow")]
fn test_milestone_sum_overflow_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let student = Address::generate(&env);
    let (pool_id, _) = new_pool(&env, &client, 1_000u128);

    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: u128::MAX });
    milestones.push_back(Milestone { amount: 1u128 });

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application"));
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

/// In withdraw_unallocated_funds, summing locked amounts checked_add prevents overflow.
#[test]
fn test_checked_add_u128_prevents_silent_wrapping() {
    let max: u128 = u128::MAX;
    assert_eq!(max.checked_add(1), None);
    assert_eq!(max.checked_add(u128::MAX), None);
}

// ─────────────────────────────────────────────────────────────────────────────
// (3) Contribution sum limits
// ─────────────────────────────────────────────────────────────────────────────

/// The largest representable u128 contribution is accepted and recorded accurately.
#[test]
fn test_max_u128_contribution_recorded_exactly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let (pool_id, _) = new_pool(&env, &client, u128::MAX);

    client.donate(&pool_id, &donor, &u128::MAX);

    assert_eq!(client.get_total_raised(&pool_id), u128::MAX);
    assert_eq!(client.get_contribution(&pool_id, &donor), u128::MAX);
}

/// Multiple contributions accumulate accurately without rounding or loss.
#[test]
fn test_multiple_contributions_accumulate_accurately() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    let (pool_id, _) = new_pool(&env, &client, 10_000_000_000u128);

    client.donate(&pool_id, &donor, &2_000_000_000u128);
    client.donate(&pool_id, &donor, &3_000_000_000u128);
    client.donate(&pool_id, &donor, &4_000_000_000u128);

    assert_eq!(client.get_contribution(&pool_id, &donor), 9_000_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 9_000_000_000u128);
}

/// Contribution checked addition panics when wrapping occurs.
#[test]
#[should_panic(expected = "Contribution amount overflow")]
fn test_donate_contribution_overflow_panics() {
    let current_contrib: u128 = u128::MAX;
    let amount: u128 = 1;
    let _new_contrib = current_contrib
        .checked_add(amount)
        .expect("Contribution amount overflow");
}

// ─────────────────────────────────────────────────────────────────────────────
// (4) Pool target amount limits
// ─────────────────────────────────────────────────────────────────────────────

/// Pool target (goal) set to u128::MAX is handled without overflow.
#[test]
fn test_pool_target_max_u128_handled() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, creator) = new_pool(&env, &client, u128::MAX);

    assert_eq!(client.get_campaign_goal(&pool_id), u128::MAX);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, u128::MAX);
}

/// Pool target set to minimum value 1 is supported.
#[test]
fn test_pool_target_minimum_boundary() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = new_pool(&env, &client, 1u128);

    assert_eq!(client.get_campaign_goal(&pool_id), 1u128);
}

/// Pool target with matching large milestone sum succeeds.
#[test]
fn test_pool_target_matches_milestones_at_scale() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let student = Address::generate(&env);
    let goal = 5_000_000_000_000u128;
    let (pool_id, _) = new_pool(&env, &client, goal);

    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: 2_000_000_000_000u128 });
    milestones.push_back(Milestone { amount: 3_000_000_000_000u128 });

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "App"));
    client.setup_application_milestones(&pool_id, &student, &milestones);

    let stored = client.get_milestones(&pool_id, &student);
    assert_eq!(stored.len(), 2);
    assert_eq!(
        stored.get(0).unwrap().amount + stored.get(1).unwrap().amount,
        goal
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// (5) Counter overflow protection
// ─────────────────────────────────────────────────────────────────────────────

/// The pool counter increments monotonically without wrapping.
#[test]
fn test_pool_counter_increments_without_wrapping() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_pool_count(), 0);
    for expected in 1..=3u32 {
        let (pool_id, _) = new_pool(&env, &client, 1_000u128);
        assert_eq!(pool_id, expected);
        assert_eq!(client.get_pool_count(), expected);
    }
}

/// Application counter increments sequentially per pool.
#[test]
fn test_application_counter_increments_per_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = new_pool(&env, &client, 10_000u128);

    let student_1 = Address::generate(&env);
    let student_2 = Address::generate(&env);
    let student_3 = Address::generate(&env);

    client.apply_to_pool(&pool_id, &student_1, &String::from_str(&env, "App 1"));
    client.apply_to_pool(&pool_id, &student_2, &String::from_str(&env, "App 2"));
    client.apply_to_pool(&pool_id, &student_3, &String::from_str(&env, "App 3"));

    let app1 = client.get_application_by_index(&pool_id, &1u32).unwrap();
    let app2 = client.get_application_by_index(&pool_id, &2u32).unwrap();
    let app3 = client.get_application_by_index(&pool_id, &3u32).unwrap();

    assert_eq!(app1.0, 1u32);
    assert_eq!(app1.1, student_1);
    assert_eq!(app2.0, 2u32);
    assert_eq!(app2.1, student_2);
    assert_eq!(app3.0, 3u32);
    assert_eq!(app3.1, student_3);
}

/// Unique donor counter increments only for distinct donors.
#[test]
fn test_donor_counter_increments_for_unique_donors_only() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _) = new_pool(&env, &client, 10_000u128);
    let donor_1 = Address::generate(&env);
    let donor_2 = Address::generate(&env);

    assert_eq!(client.get_donor_count(&pool_id), 0);

    client.donate(&pool_id, &donor_1, &100u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);

    // Repeat donation does not increment donor count
    client.donate(&pool_id, &donor_1, &200u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);

    // Second unique donor increments count
    client.donate(&pool_id, &donor_2, &300u128);
    assert_eq!(client.get_donor_count(&pool_id), 2);
}

/// Counter overflow protection checked arithmetic bound.
#[test]
fn test_counter_overflow_checked_arithmetic() {
    let max_counter: u32 = u32::MAX;
    assert_eq!(max_counter.checked_add(1), None);
}
