#![cfg(test)]
//! Tests for business logic consistency — issue #1321.
//!
//! Covers business rules:
//!   (1) Campaign rules consistently applied across creation flows and getters.
//!   (2) Pool rules enforced uniformly for school linkage, applications, and donor metrics.
//!   (3) Fee calculations accurate for creation fees, protocol disbursement fees, and fee claims.
//!   (4) Time-based rules work correctly for deadlines, grace periods, and timestamp updates.
//!   (5) State machine logic sound across pool lifecycle transitions and operation guards.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, BytesN, Env, String, Vec,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(token_admin);
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn setup_pool(env: &Env, client: &ContractClient, creator: &Address) -> u32 {
    client.create_pool(
        creator,
        &String::from_str(env, "Business Logic Test Pool"),
        &String::from_str(env, "Testing business rule consistency"),
        &10_000_000u128,
        &100_000u64,
    )
}

// ── (1) Campaign rules consistently applied ─────────────────────────────────

#[test]
fn test_campaign_rules_creation_and_getters_consistent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let pool_id1 = client.create_pool(
        &creator1,
        &String::from_str(&env, "First Campaign"),
        &String::from_str(&env, "Campaign 1 Description"),
        &50_000_000u128,
        &200_000u64,
    );

    let pool_id2 = client.create_pool(
        &creator2,
        &String::from_str(&env, "Second Campaign"),
        &String::from_str(&env, "Campaign 2 Description"),
        &100_000_000u128,
        &300_000u64,
    );

    assert_eq!(pool_id1, 1);
    assert_eq!(pool_id2, 2);
    assert_eq!(client.get_pool_count(), 2);

    let all_campaigns = client.get_all_campaigns();
    assert_eq!(all_campaigns.len(), 2);
    assert_eq!(all_campaigns.get(0).unwrap(), 1);
    assert_eq!(all_campaigns.get(1).unwrap(), 2);

    assert_eq!(client.get_campaign_goal(&pool_id1), 50_000_000u128);
    assert_eq!(client.get_campaign_goal(&pool_id2), 100_000_000u128);
    assert_eq!(client.get_campaign_balance(&pool_id1), 0u128);
    assert_eq!(client.get_total_raised(&pool_id1), 0u128);

    let (id1, sponsor1, goal1, collected1, is_closed1, deadline1) = client.get_pool(&pool_id1);
    assert_eq!(id1, 1);
    assert_eq!(sponsor1, creator1);
    assert_eq!(goal1, 50_000_000u128);
    assert_eq!(collected1, 0u128);
    assert!(!is_closed1);
    assert_eq!(deadline1, 200_000u64);
}

#[test]
#[should_panic(expected = "Title cannot be empty")]
fn test_campaign_rules_empty_title_rejected() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    client.create_pool(
        &creator,
        &String::from_str(&env, ""),
        &String::from_str(&env, "Valid Description"),
        &1_000_000u128,
        &100_000u64,
    );
}

#[test]
#[should_panic(expected = "Description cannot be empty")]
fn test_campaign_rules_empty_description_rejected() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    client.create_pool(
        &creator,
        &String::from_str(&env, "Valid Title"),
        &String::from_str(&env, ""),
        &1_000_000u128,
        &100_000u64,
    );
}

#[test]
#[should_panic(expected = "Duration must be greater than zero")]
fn test_campaign_rules_zero_duration_rejected() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    client.create_pool(
        &creator,
        &String::from_str(&env, "Valid Title"),
        &String::from_str(&env, "Valid Description"),
        &1_000_000u128,
        &0u64,
    );
}

#[test]
fn test_campaign_rules_create_campaign_with_fee_alias_matches_pool_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 10_000i128, &creator);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &500i128);

    let pool_id = client.create_campaign_with_fee(
        &creator,
        &String::from_str(&env, "Fee Campaign"),
        &String::from_str(&env, "Campaign with fee"),
        &5_000_000u128,
        &150_000u64,
        &token,
    );

    assert_eq!(pool_id, 1);
    assert_eq!(client.get_pool(&pool_id).0, 1);
    assert_eq!(client.get_pool(&pool_id).2, 5_000_000u128);
    assert_eq!(soroban_sdk::token::Client::new(&env, &token).balance(&contract_id), 500i128);
}

// ── (2) Pool rules enforced uniformly ───────────────────────────────────────

#[test]
fn test_pool_rules_school_linkage_and_registration() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[9u8; 32]);

    client.set_admin(&admin);
    assert!(!client.is_school_registered(&school));

    let unreg_result = client.try_create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "School Description"),
        &2_000_000u128,
        &school,
        &100_000u64,
    );
    assert_eq!(unreg_result, Err(Ok(ContractError::SchoolNotRegistered)));

    client.register_school(&school, &metadata_hash);
    assert!(client.is_school_registered(&school));
    assert_eq!(client.get_school_metadata(&school), metadata_hash);

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "School Description"),
        &2_000_000u128,
        &school,
        &100_000u64,
    );

    assert_eq!(client.get_pool_school(&pool_id), school);
}

#[test]
fn test_pool_rules_application_workflow_uniformity() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let other_school = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);

    client.set_admin(&admin);
    client.register_school(&school, &metadata_hash);
    client.register_school(&other_school, &metadata_hash);

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Scholarship Pool"),
        &String::from_str(&env, "Student funding"),
        &10_000_000u128,
        &school,
        &100_000u64,
    );

    assert!(!client.has_applied(&pool_id, &student));

    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Application Data JSON"),
    );

    assert!(client.has_applied(&pool_id, &student));
    assert_eq!(
        client.get_application_status(&pool_id, &student),
        String::from_str(&env, "Pending")
    );

    let dup_result = client.try_apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Duplicate Application Data"),
    );
    assert_eq!(dup_result, Err(Ok(ContractError::DuplicateApplication)));

    let unauth_approve = client.try_approve_application(
        &pool_id,
        &other_school,
        &student,
        &true,
    );
    assert_eq!(
        unauth_approve,
        Err(Ok(ContractError::OnlyLinkedSchoolCanApprove))
    );

    client.approve_application(&pool_id, &school, &student, &true);
    assert_eq!(
        client.get_application_status(&pool_id, &student),
        String::from_str(&env, "Approved")
    );
}

#[test]
fn test_pool_rules_donor_tracking_and_metrics_uniformity() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = setup_pool(&env, &client, &creator);

    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    assert_eq!(client.get_donor_count(&pool_id), 0);
    assert_eq!(client.get_contribution(&pool_id, &donor1), 0);

    client.donate(&pool_id, &donor1, &100_000u128);
    client.donate(&pool_id, &donor1, &150_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);
    assert_eq!(client.get_contribution(&pool_id, &donor1), 250_000u128);

    client.donate(&pool_id, &donor2, &300_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 2);
    assert_eq!(client.get_contribution(&pool_id, &donor2), 300_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 550_000u128);
}

// ── (3) Fee calculations accurate ───────────────────────────────────────────

#[test]
fn test_fee_calculations_creation_fee_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let creator = Address::generate(&env);

    client.set_admin(&admin);
    assert_eq!(client.get_creation_fee(), 0);

    let unauth_fee = client.try_set_creation_fee(&non_admin, &1_000i128);
    assert_eq!(unauth_fee, Err(Ok(ContractError::UnauthorizedAdmin)));

    let invalid_fee = client.try_set_creation_fee(&admin, &-50i128);
    assert_eq!(invalid_fee, Err(Ok(ContractError::InvalidFee)));

    client.set_creation_fee(&admin, &250i128);
    assert_eq!(client.get_creation_fee(), 250);

    let fee_token = create_token(&env, 1_000i128, &creator);
    let pool_id = client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Fee Pool"),
        &String::from_str(&env, "Description"),
        &5_000_000u128,
        &100_000u64,
        &fee_token,
    );
    assert_eq!(pool_id, 1);
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &fee_token).balance(&contract_id),
        250i128
    );
}

#[test]
fn test_fee_calculations_disbursement_protocol_fee_and_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let donor = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[3u8; 32]);

    client.set_admin(&admin);
    client.register_school(&school, &metadata_hash);

    let token = create_token(&env, 50_000i128, &donor);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Disbursement Pool"),
        &String::from_str(&env, "Description"),
        &10_000u128,
        &school,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);

    client.donate_with_token(&pool_id, &donor, &token, &10_000i128);
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Application"),
    );
    client.approve_application(&pool_id, &school, &student, &true);

    // Student claims 10,000 tokens:
    // Protocol fee is 1% = 100 tokens, net transfer = 9,900 tokens
    client.claim_funds(&student, &pool_id, &10_000i128, &token);

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&student), 9_900i128);
    assert_eq!(token_client.balance(&contract_id), 100i128);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 10_000i128);

    // Non-admin cannot claim fees
    let unauth_claim = client.try_claim_protocol_fees(&student, &token);
    assert_eq!(unauth_claim, Err(Ok(ContractError::UnauthorizedAdmin)));

    // Admin claims protocol fees
    let claimed_fees = client.claim_protocol_fees(&admin, &token);
    assert_eq!(claimed_fees, 100i128);
    assert_eq!(token_client.balance(&admin), 100i128);
    assert_eq!(token_client.balance(&contract_id), 0i128);

    // Second claim fails as no fees remain
    let no_fees = client.try_claim_protocol_fees(&admin, &token);
    assert_eq!(no_fees, Err(Ok(ContractError::NoUnclaimedFees)));
}

// ── (4) Time-based rules work correctly ─────────────────────────────────────

#[test]
fn test_time_based_rules_helper_functions() {
    assert_eq!(current_timestamp(), 100_000u64);
    assert!(validate_deadline(200_000u64).is_ok());
    assert!(validate_deadline(100_000u64).is_err());
    assert!(validate_deadline(50_000u64).is_err());
    assert!(set_deadline(150_000u64).is_ok());
    assert!(set_deadline(100_000u64).is_err());
    assert!(is_within_grace_period(90_000u64, 20_000u64));
    assert!(!is_within_grace_period(70_000u64, 20_000u64));
    assert!(!is_within_grace_period(120_000u64, 20_000u64));
}

#[test]
fn test_time_based_rules_pool_deadline_and_refund_grace_period() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 5_000i128, &donor);

    let pool_id = setup_pool(&env, &client, &creator);
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &donor, &token, &3_000i128);

    assert_eq!(client.get_pool_deadline(&pool_id), 0);

    // Set deadline at sequence 100
    env.ledger().set_sequence_number(50);
    client.set_pool_deadline(&pool_id, &100u32);
    assert_eq!(client.get_pool_deadline(&pool_id), 100);

    // Ledger before deadline -> cannot refund
    env.ledger().set_sequence_number(90);
    let early_refund = client.try_refund_donation(&pool_id, &donor, &token);
    assert_eq!(early_refund, Err(Ok(ContractError::PoolNotExpired)));

    // Ledger after deadline but before grace period elapsed (REFUND_GRACE_PERIOD_LEDGERS = 17280)
    env.ledger().set_sequence_number(100 + 500);
    let grace_refund = client.try_refund_donation(&pool_id, &donor, &token);
    assert_eq!(grace_refund, Err(Ok(ContractError::PoolNotExpired)));

    // Ledger after deadline + grace period elapsed
    env.ledger().set_sequence_number(100 + REFUND_GRACE_PERIOD_LEDGERS);
    client.refund_donation(&pool_id, &donor, &token);

    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&donor),
        5_000i128
    );
    assert_eq!(client.get_pool(&pool_id).3, 0u128);
}

#[test]
fn test_time_based_rules_last_donation_timestamp_tracking() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = setup_pool(&env, &client, &creator);
    assert_eq!(client.get_last_donation_at(&pool_id), 0u64);

    env.ledger().set_timestamp(1_000u64);
    client.donate(&pool_id, &donor, &100u128);
    assert_eq!(client.get_last_donation_at(&pool_id), 1_000u64);

    env.ledger().set_timestamp(5_000u64);
    client.donate(&pool_id, &donor, &200u128);
    assert_eq!(client.get_last_donation_at(&pool_id), 5_000u64);
}

// ── (5) State machine logic sound ───────────────────────────────────────────

#[test]
fn test_state_machine_initial_state_and_operation_guards() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = setup_pool(&env, &client, &creator);

    // Initial state is Active: donations succeed
    client.donate(&pool_id, &donor, &500u128);
    assert_eq!(client.get_total_raised(&pool_id), 500u128);

    // Transition to Paused: donations fail
    client.set_pool_state(&pool_id, &PoolState::Paused);
    let paused_result = client.try_donate(&pool_id, &donor, &100u128);
    assert_eq!(paused_result, Err(Ok(ContractError::InvalidPoolState)));

    // Transition to Cancelled: donations fail
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    let cancelled_result = client.try_donate(&pool_id, &donor, &100u128);
    assert_eq!(cancelled_result, Err(Ok(ContractError::InvalidPoolState)));

    // Transition back to Active: donations succeed again
    client.set_pool_state(&pool_id, &PoolState::Active);
    client.donate(&pool_id, &donor, &300u128);
    assert_eq!(client.get_total_raised(&pool_id), 800u128);
}

#[test]
fn test_state_machine_close_pool_lifecycle_and_constraints() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = setup_pool(&env, &client, &creator);

    // Cannot close from Active state
    let close_active = client.try_close_pool(&pool_id);
    assert_eq!(
        close_active,
        Err(Ok(ContractError::PoolNotDisbursedOrRefunded))
    );

    // Cannot close from Paused state
    client.set_pool_state(&pool_id, &PoolState::Paused);
    let close_paused = client.try_close_pool(&pool_id);
    assert_eq!(
        close_paused,
        Err(Ok(ContractError::PoolNotDisbursedOrRefunded))
    );

    // Can close from Cancelled state
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    client.close_pool(&pool_id);
    assert!(client.get_pool(&pool_id).4);

    // Cannot close an already closed pool
    let already_closed = client.try_close_pool(&pool_id);
    assert_eq!(already_closed, Err(Ok(ContractError::PoolAlreadyClosed)));

    // Cannot donate to a closed pool
    let donate_closed = client.try_donate(&pool_id, &donor, &100u128);
    assert_eq!(donate_closed, Err(Ok(ContractError::PoolIsClosed)));
}
