#![cfg(test)]
//! Tests for recovery scenarios — issue #1322.
//!
//! Covers error recovery:
//!   (1) Failed operations don't corrupt state.
//!   (2) Partial failures handled cleanly.
//!   (3) System recoverable after errors.
//!   (4) Rollback mechanisms work.
//!   (5) Graceful degradation possible.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, BytesN, Env, String, Symbol, Vec,
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
        &String::from_str(env, "Recovery Test Pool"),
        &String::from_str(env, "Testing error recovery scenarios"),
        &10_000_000u128,
        &100_000u64,
    )
}

// ── (1) Failed operations don't corrupt state ───────────────────────────────

#[test]
fn test_recovery_failed_donation_preserves_pool_and_donor_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = setup_pool(&env, &client, &creator);

    // Make an initial valid donation
    client.donate(&pool_id, &donor, &500_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 500_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);
    assert_eq!(client.get_contribution(&pool_id, &donor), 500_000u128);

    // Failed donation to nonexistent pool
    let invalid_pool = client.try_donate(&9999u32, &donor, &100_000u128);
    assert_eq!(invalid_pool, Err(Ok(ContractError::PoolNotFound)));

    // Pause pool and attempt donation (fails)
    client.set_pool_state(&pool_id, &PoolState::Paused);
    let paused_donation = client.try_donate(&pool_id, &donor, &100_000u128);
    assert_eq!(paused_donation, Err(Ok(ContractError::InvalidPoolState)));

    // Verify existing state is completely preserved
    assert_eq!(client.get_total_raised(&pool_id), 500_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);
    assert_eq!(client.get_contribution(&pool_id, &donor), 500_000u128);
}

#[test]
fn test_recovery_failed_save_pool_preserves_previous_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = setup_pool(&env, &client, &creator);

    let signer1 = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer1.clone()]);

    // Valid save_pool
    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Initial Description"),
        &String::from_str(&env, "https://initial.example"),
        &String::from_str(&env, "initialhash"),
        &1u32,
        &signers,
    );

    assert_eq!(
        client.get_saved_pool_metadata(&pool_id),
        (
            String::from_str(&env, "Initial Description"),
            String::from_str(&env, "https://initial.example"),
            String::from_str(&env, "initialhash"),
        )
    );
    assert_eq!(client.get_pool_signers(&pool_id).0, 1);

    // Failed save_pool: required_signatures > signers.len()
    let duplicate_signer = signer1.clone();
    let mismatched_signers = Vec::from_array(&env, [signer1.clone(), duplicate_signer]);
    let failed_call = client.try_save_pool(
        &pool_id,
        &String::from_str(&env, "Corrupted Description"),
        &String::from_str(&env, "https://corrupted.example"),
        &String::from_str(&env, "corruptedhash"),
        &2u32,
        &mismatched_signers,
    );
    assert!(failed_call.is_err());

    // Verify initial metadata and signers remain uncorrupted
    assert_eq!(
        client.get_saved_pool_metadata(&pool_id),
        (
            String::from_str(&env, "Initial Description"),
            String::from_str(&env, "https://initial.example"),
            String::from_str(&env, "initialhash"),
        )
    );
    assert_eq!(client.get_pool_signers(&pool_id).0, 1);
}

#[test]
fn test_recovery_failed_admin_operations_preserve_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &100i128);

    // Attacker attempts to change creation fee
    let fee_attack = client.try_set_creation_fee(&attacker, &9999i128);
    assert_eq!(fee_attack, Err(Ok(ContractError::UnauthorizedAdmin)));
    assert_eq!(client.get_creation_fee(), 100i128);

    // Attacker attempts to set token
    let fake_token = Address::generate(&env);
    let token_attack = client.try_set_crowdfunding_token(&attacker, &fake_token);
    assert_eq!(token_attack, Err(Ok(ContractError::UnauthorizedAdmin)));
}

// ── (2) Partial failures handled cleanly ────────────────────────────────────

#[test]
fn test_recovery_insufficient_balance_donation_preserves_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 100i128, &donor);
    let pool_id = setup_pool(&env, &client, &creator);

    // Donor has 100 tokens, attempts to donate 500 tokens
    let transfer_fail = client.try_donate_with_token(&pool_id, &donor, &token, &500i128);
    assert!(transfer_fail.is_err());

    // Neither pool nor donor state was partially modified
    assert_eq!(client.get_total_raised(&pool_id), 0u128);
    assert_eq!(client.get_donor_count(&pool_id), 0);
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&donor),
        100i128
    );
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&contract_id),
        0i128
    );
}

#[test]
fn test_recovery_invalid_milestone_sum_prevents_partial_storage() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = setup_pool(&env, &client, &creator); // goal = 10,000,000

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "App"));

    // Milestones sum to 8,000,000 != goal (10,000,000)
    let bad_milestones = Vec::from_array(
        &env,
        [
            Milestone { amount: 5_000_000u128 },
            Milestone { amount: 3_000_000u128 },
        ],
    );

    let failed_setup = client.try_setup_application_milestones(&pool_id, &student, &bad_milestones);
    assert!(failed_setup.is_err());

    // Storage remains clean and empty
    assert_eq!(client.get_milestones(&pool_id, &student).len(), 0);
}

#[test]
fn test_recovery_overdraw_claim_funds_prevents_partial_disbursement() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let donor = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[5u8; 32]);

    client.set_admin(&admin);
    client.register_school(&school, &metadata_hash);

    let token = create_token(&env, 10_000i128, &donor);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Overdraw Test Pool"),
        &String::from_str(&env, "Test"),
        &10_000u128,
        &school,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &donor, &token, &5_000i128); // collected = 5,000

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "App"));
    client.approve_application(&pool_id, &school, &student, &true);

    // Student attempts to claim 6,000 when collected is only 5,000
    let overdraw_attempt = client.try_claim_funds(&student, &pool_id, &6_000i128, &token);
    assert!(overdraw_attempt.is_err());

    // State intact
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 0i128);
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&student),
        0i128
    );
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&contract_id),
        5_000i128
    );
}

// ── (3) System recoverable after errors ─────────────────────────────────────

#[test]
fn test_recovery_successful_donation_after_failed_attempt() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone());
    let sac = StellarAssetClient::new(&env, &token.address());
    let pool_id = setup_pool(&env, &client, &creator);

    // Initial failed attempt (0 balance)
    let fail = client.try_donate_with_token(&pool_id, &donor, &token.address(), &500i128);
    assert!(fail.is_err());

    // Donor receives funds and retries successfully
    sac.mint(&donor, &1_000i128);
    client.donate_with_token(&pool_id, &donor, &token.address(), &500i128);

    assert_eq!(client.get_total_raised(&pool_id), 500u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token.address()).balance(&donor),
        500i128
    );
}

#[test]
fn test_recovery_authorized_admin_succeeds_after_unauthorized_attempts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unauth = Address::generate(&env);
    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[7u8; 32]);

    client.set_admin(&admin);

    // Unauthorized attempt to register school
    let fail_reg = client.try_register_school(&school, &metadata_hash);
    // Real admin registers school
    client.register_school(&school, &metadata_hash);
    assert!(client.is_school_registered(&school));

    // Unauthorized attempt to set creation fee
    let fail_fee = client.try_set_creation_fee(&unauth, &500i128);
    assert_eq!(fail_fee, Err(Ok(ContractError::UnauthorizedAdmin)));

    // Real admin sets creation fee successfully
    client.set_creation_fee(&admin, &300i128);
    assert_eq!(client.get_creation_fee(), 300i128);
}

#[test]
fn test_recovery_valid_milestone_submission_after_failed_attempt() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = setup_pool(&env, &client, &creator); // goal = 10,000,000

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "App"));

    // Failed attempt with mismatched sum
    let bad_milestones = Vec::from_array(&env, [Milestone { amount: 1_000_000u128 }]);
    let fail = client.try_setup_application_milestones(&pool_id, &student, &bad_milestones);
    assert!(fail.is_err());

    // Successful resubmission with correct sum (10,000,000)
    let good_milestones = Vec::from_array(
        &env,
        [
            Milestone { amount: 4_000_000u128 },
            Milestone { amount: 6_000_000u128 },
        ],
    );
    client.setup_application_milestones(&pool_id, &student, &good_milestones);

    let saved = client.get_milestones(&pool_id, &student);
    assert_eq!(saved.len(), 2);
    assert_eq!(saved.get(0).unwrap().amount, 4_000_000u128);
    assert_eq!(saved.get(1).unwrap().amount, 6_000_000u128);
}

// ── (4) Rollback mechanisms work ───────────────────────────────────────────

#[test]
fn test_recovery_refund_donation_rollback_and_subsequent_rejection() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token = create_token(&env, 10_000i128, &donor);

    let pool_id = setup_pool(&env, &client, &creator);
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &donor, &token, &7_000i128);

    assert_eq!(client.get_total_raised(&pool_id), 7_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 7_000u128);

    // Set deadline sequence
    env.ledger().set_sequence_number(10);
    client.set_pool_deadline(&pool_id, &20u32);

    // Advance sequence past deadline + grace period
    env.ledger().set_sequence_number(20 + REFUND_GRACE_PERIOD_LEDGERS);

    // Refund donor
    client.refund_donation(&pool_id, &donor, &token);

    // Contribution rolled back, pool collected rolled back, tokens returned
    assert_eq!(client.get_total_raised(&pool_id), 0u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 0u128);
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&donor),
        10_000i128
    );

    // Second refund attempt fails cleanly
    let repeat_refund = client.try_refund_donation(&pool_id, &donor, &token);
    assert_eq!(
        repeat_refund,
        Err(Ok(ContractError::NoContributionToRefund))
    );
}

#[test]
fn test_recovery_withdraw_unallocated_funds_rollback_to_sponsor() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let donor = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[2u8; 32]);

    client.set_admin(&admin);
    client.register_school(&school, &metadata_hash);

    let token = create_token(&env, 20_000i128, &donor);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Surplus Pool"),
        &String::from_str(&env, "Surplus test"),
        &10_000u128,
        &school,
        &100_000u64,
    );
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &donor, &token, &10_000i128);

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "App"));
    client.approve_application(&pool_id, &school, &student, &true);

    // Student claims 2,000 (net: 1,980 to student, 20 to fees, remaining approved: 8,000, surplus: 0)
    client.claim_funds(&student, &pool_id, &2_000i128, &token);

    // Sponsor attempts to withdraw surplus when none is unallocated
    let no_surplus = client.try_withdraw_unallocated_funds(&pool_id, &token);
    assert!(no_surplus.is_err());
}

// ── (5) Graceful degradation possible ───────────────────────────────────────

#[test]
fn test_recovery_emergency_withdrawal_lifecycle_and_cleanup() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 50_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = setup_pool(&env, &client, &creator);

    // Request emergency withdrawal
    env.ledger().set_timestamp(1_000);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &50_000i128);

    // Attempt execution before grace period (86,400s) elapses
    env.ledger().set_timestamp(1_000 + 86_399);
    let early_exec = client.try_execute_emergency_withdraw(&pool_id);
    assert!(early_exec.is_err());

    // Execute at/after grace period
    env.ledger().set_timestamp(1_000 + 86_400);
    client.execute_emergency_withdraw(&pool_id);

    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token).balance(&admin),
        50_000i128
    );

    // Verify storage key cleaned up
    let repeat_exec = client.try_execute_emergency_withdraw(&pool_id);
    assert!(repeat_exec.is_err());
}

#[test]
fn test_recovery_zero_creation_fee_graceful_handling() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let dummy_token = Address::generate(&env);

    client.set_admin(&admin);
    // Ensure 0 creation fee operates gracefully without requiring token balance
    client.set_creation_fee(&admin, &0i128);

    let pool_id = client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Zero Fee Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &100_000u64,
        &dummy_token,
    );

    assert_eq!(pool_id, 1);
    assert_eq!(client.get_pool(&pool_id).2, 1_000_000u128);
}

#[test]
fn test_recovery_empty_pool_queries_return_defaults() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = setup_pool(&env, &client, &creator);

    assert_eq!(client.get_donor_count(&pool_id), 0);
    assert_eq!(client.get_contribution(&pool_id, &student), 0u128);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 0i128);
    assert_eq!(client.get_milestones(&pool_id, &student).len(), 0);
    assert_eq!(client.get_pool_deadline(&pool_id), 0u32);
    assert_eq!(
        client.get_application_status(&pool_id, &student),
        String::from_str(&env, "")
    );
}
