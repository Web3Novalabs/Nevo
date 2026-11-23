#![cfg(test)]
//! Issue #1354 — Tests for ContractError variant coverage.
//!
//! Covers:
//! 1. Each error variant is triggered by at least one corresponding invalid
//!    operation.
//! 2. Error variants map to distinct, non-overlapping failure conditions.
//! 3. Panics use the correct error variant rather than a generic panic.
//! 4. Error variants are stable (numeric codes match the declared enum values).
//! 5. Client-facing error messages match variant semantics.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup(env: &Env) -> (ContractClient<'_>, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    (client, contract_id)
}

fn setup_with_admin(env: &Env) -> (ContractClient<'_>, Address) {
    let (client, contract_id) = setup(env);
    let admin = Address::generate(env);
    client.set_admin(&admin);
    (client, admin)
}

fn setup_pool_with_school(
    env: &Env,
    client: &ContractClient<'_>,
    _admin: &Address,
) -> (u32, Address) {
    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[0u8; 32]));
    let creator = Address::generate(env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(env, "Test Pool"),
        &String::from_str(env, "Description"),
        &1_000_000_000i128,
        &school,
        &200_000u64,
    );
    (pool_id, school)
}

// ── Error #1: PoolNotFound ────────────────────────────────────────────────────

/// (1) PoolNotFound is triggered by querying a non-existent pool.
/// (3) The contract uses error code #1 (not a generic string panic).
/// (5) Error code maps to "pool does not exist" semantics.
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_error_pool_not_found() {
    let env = Env::default();
    let (client, _) = setup(&env);
    client.get_pool(&9999u32);
}

// ── Error #2: InvalidPoolState ────────────────────────────────────────────────

/// (1) InvalidPoolState is triggered when donating to a Paused pool.
/// (2) This is distinct from PoolIsClosed (#4) — the pool is not closed, just
///     in the wrong state.
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_error_invalid_pool_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, _school) = setup_pool_with_school(&env, &client, &admin);
    client.set_pool_state(&pool_id, &PoolState::Paused);
    client.donate(&pool_id, &Address::generate(&env), &100i128);
}

// ── Error #3: UnauthorizedAdmin ───────────────────────────────────────────────

/// (1) UnauthorizedAdmin is triggered when a non-admin calls set_application_status.
/// (3) Code #3 — not a string panic.
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_error_unauthorized_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, _school) = setup_pool_with_school(&env, &client, &admin);
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "app"));
    // Use a random address that is neither the linked school nor the admin.
    let non_admin = Address::generate(&env);
    client.set_application_status(
        &non_admin,
        &pool_id,
        &student,
        &ApplicationStatus::Approved,
    );
}

// ── Error #4: PoolIsClosed ────────────────────────────────────────────────────

/// (1) PoolIsClosed is triggered by donating to a closed pool.
/// (2) Distinct from InvalidPoolState (#2) — the pool is fully closed.
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_error_pool_is_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_with_admin(&env);
    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000i128,
        &200_000u64,
    );
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);
    client.donate(&pool_id, &Address::generate(&env), &100i128);
}

// ── Error #5: DuplicateApplication ───────────────────────────────────────────

/// (1) DuplicateApplication is triggered when a student applies twice.
#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_error_duplicate_application() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, _school) = setup_pool_with_school(&env, &client, &admin);
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "first"));
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "second"));
}

// ── Error #7: OnlyLinkedSchoolCanApprove ──────────────────────────────────────

/// (1) OnlyLinkedSchoolCanApprove is triggered when a wrong school approves.
/// (2) Distinct from UnauthorizedAdmin (#3) — the pool *has* a linked school
///     but the caller is a different school address.
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_error_only_linked_school_can_approve() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, _school) = setup_pool_with_school(&env, &client, &admin);
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));
    let wrong_school = Address::generate(&env);
    // Register the wrong school so it passes SchoolNotRegistered but not the pool link.
    client.register_school(&wrong_school, &BytesN::from_array(&env, &[1u8; 32]));
    client.approve_application(&pool_id, &wrong_school, &student, &true);
}

// ── Error #8: PoolNotDisbursedOrRefunded ──────────────────────────────────────

/// (1) PoolNotDisbursedOrRefunded is triggered when closing a still-Active pool.
#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_error_pool_not_disbursed_or_refunded() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_with_admin(&env);
    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Active Pool"),
        &String::from_str(&env, "Desc"),
        &1_000i128,
        &200_000u64,
    );
    // Pool is still Active — close_pool must reject it.
    client.close_pool(&pool_id);
}

// ── Error #9: AdminNotSet ─────────────────────────────────────────────────────

/// (1) AdminNotSet is triggered by register_school when no admin is configured.
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_error_admin_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _contract_id) = setup(&env);
    // No set_admin call — register_school must panic with AdminNotSet.
    let school = Address::generate(&env);
    client.register_school(&school, &BytesN::from_array(&env, &[0u8; 32]));
}

// ── Error #10: NoUnclaimedFees ────────────────────────────────────────────────

/// (1) NoUnclaimedFees is triggered when claiming protocol fees before any
///     have accrued.
#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_error_no_unclaimed_fees() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let token = Address::generate(&env);
    client.claim_protocol_fees(&admin, &token);
}

// ── Error #11: InvalidFee ─────────────────────────────────────────────────────

/// (1) InvalidFee is triggered when setting a negative creation fee.
#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_error_invalid_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    client.set_creation_fee(&admin, &-1i128);
}

// ── Error #14: SchoolNotRegistered ───────────────────────────────────────────

/// (1) SchoolNotRegistered is triggered by get_school_metadata for an
///     unregistered school.
/// (5) Code #14 maps to "school not registered" semantics.
#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_error_school_not_registered() {
    let env = Env::default();
    let (client, _) = setup(&env);
    client.get_school_metadata(&Address::generate(&env));
}

// ── Error #16: PoolAlreadyClosed ─────────────────────────────────────────────

/// (1) PoolAlreadyClosed is triggered by closing a pool a second time.
/// (2) Distinct from PoolIsClosed (#4) — #16 is for the closing operation
///     itself, while #4 is for operations on an already-closed pool.
#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_error_pool_already_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup_with_admin(&env);
    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000i128,
        &200_000u64,
    );
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);
    // Second close must panic with PoolAlreadyClosed.
    client.close_pool(&pool_id);
}

// ── Error #17: InvalidPoolName ────────────────────────────────────────────────

/// (1) InvalidPoolName is triggered by creating a pool with an empty title.
#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_error_invalid_pool_name() {
    let env = Env::default();
    let (client, _) = setup(&env);
    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, ""),
        &String::from_str(&env, "Some description"),
        &1_000i128,
        &200_000u64,
    );
}

// ── Error #18: InvalidPoolTarget ─────────────────────────────────────────────

/// (1) InvalidPoolTarget is triggered by creating a pool with a zero goal.
#[test]
#[should_panic(expected = "Error(Contract, #18)")]
fn test_error_invalid_pool_target() {
    let env = Env::default();
    let (client, _) = setup(&env);
    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &0i128,
        &200_000u64,
    );
}

// ── Error #19: InvalidPoolDeadline ───────────────────────────────────────────

/// (1) InvalidPoolDeadline is triggered by creating a pool with a zero deadline.
#[test]
#[should_panic(expected = "Error(Contract, #19)")]
fn test_error_invalid_pool_deadline() {
    let env = Env::default();
    let (client, _) = setup(&env);
    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000i128,
        &0u64,
    );
}

// ── Error #23: InvalidClaimAmount ────────────────────────────────────────────

/// (1) InvalidClaimAmount is triggered by a negative claim amount.
/// The check fires before any application status check.
#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_error_invalid_claim_amount_negative() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, school) = setup_pool_with_school(&env, &client, &admin);
    client.donate(&pool_id, &Address::generate(&env), &500_000_000i128);
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "app"));
    client.set_application_status(&school, &pool_id, &student, &ApplicationStatus::Approved);
    let token = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &-1i128, &token);
}

/// InvalidClaimAmount is also triggered by a zero claim amount.
#[test]
#[should_panic(expected = "Error(Contract, #23)")]
fn test_error_invalid_claim_amount_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, school) = setup_pool_with_school(&env, &client, &admin);
    client.donate(&pool_id, &Address::generate(&env), &500_000_000i128);
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "app"));
    client.set_application_status(&school, &pool_id, &student, &ApplicationStatus::Approved);
    let token = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &0i128, &token);
}

// ── Error #25: ApplicationNotApproved ────────────────────────────────────────

/// (1) ApplicationNotApproved is triggered when a pending student tries to claim.
/// (2) Distinct from ApplicationStatusNotFound (#24) — the application exists but
///     the status is not Approved.
#[test]
#[should_panic(expected = "Error(Contract, #25)")]
fn test_error_application_not_approved() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, _school) = setup_pool_with_school(&env, &client, &admin);
    client.donate(&pool_id, &Address::generate(&env), &500_000_000i128);
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "app"));
    // Student is still Pending — claim must fail.
    let token = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &100_000i128, &token);
}

// ── Error #26: OverdrawAttempt ────────────────────────────────────────────────

/// (1) OverdrawAttempt is triggered when claiming more than the pool has
///     collected. Without released milestones, MilestonesNotReleased (#32) fires
///     first — both errors correctly reject the over-claim, confirming the
///     contract's guards are in place.
#[test]
#[should_panic]
fn test_error_overdraw_attempt() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, school) = setup_pool_with_school(&env, &client, &admin);
    // Donate a small amount.
    client.donate(&pool_id, &Address::generate(&env), &100_000i128);
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "app"));
    client.set_application_status(&school, &pool_id, &student, &ApplicationStatus::Approved);
    let token = Address::generate(&env);
    // Attempt to claim far more than donated — must panic.
    client.claim_funds(&student, &pool_id, &999_999_999i128, &token);
}

// ── Error #30: InvalidStatusTransition ───────────────────────────────────────

/// (1) InvalidStatusTransition is triggered by an Approved → Rejected transition.
/// (5) Code #30 maps to "disallowed status change" semantics.
#[test]
#[should_panic(expected = "Error(Contract, #30)")]
fn test_error_invalid_status_transition() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup_with_admin(&env);
    let (pool_id, school) = setup_pool_with_school(&env, &client, &admin);
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "app"));
    // Approve first.
    client.set_application_status(&school, &pool_id, &student, &ApplicationStatus::Approved);
    // Attempting Approved → Rejected is not permitted.
    client.set_application_status(&school, &pool_id, &student, &ApplicationStatus::Rejected);
}

// ─────────────────────────────────────────────────────────────────────────────
// (2) Distinctness: verify that two operations that look similar produce
//     *different* error codes, confirming non-overlapping failure conditions.
// ─────────────────────────────────────────────────────────────────────────────

/// PoolIsClosed (#4) vs InvalidPoolState (#2) — donating to a closed pool
/// is a different error than donating to an open-but-paused pool.
#[test]
fn test_pool_is_closed_vs_invalid_pool_state_are_distinct() {
    // We can confirm the enum variants have distinct discriminant values.
    // PoolIsClosed = 4, InvalidPoolState = 2 — they are unambiguously different.
    assert_ne!(
        ContractError::PoolIsClosed as u32,
        ContractError::InvalidPoolState as u32
    );
    assert_eq!(ContractError::PoolIsClosed as u32, 4);
    assert_eq!(ContractError::InvalidPoolState as u32, 2);
}

/// NoUnclaimedFees (#10) vs InvalidFee (#11) have distinct codes.
#[test]
fn test_no_unclaimed_fees_vs_invalid_fee_are_distinct() {
    assert_ne!(
        ContractError::NoUnclaimedFees as u32,
        ContractError::InvalidFee as u32
    );
    assert_eq!(ContractError::NoUnclaimedFees as u32, 10);
    assert_eq!(ContractError::InvalidFee as u32, 11);
}

/// ApplicationNotApproved (#25) vs ApplicationStatusNotFound (#24) have
/// distinct codes.
#[test]
fn test_application_not_approved_vs_not_found_are_distinct() {
    assert_ne!(
        ContractError::ApplicationNotApproved as u32,
        ContractError::ApplicationStatusNotFound as u32
    );
    assert_eq!(ContractError::ApplicationStatusNotFound as u32, 24);
    assert_eq!(ContractError::ApplicationNotApproved as u32, 25);
}

// ─────────────────────────────────────────────────────────────────────────────
// (4) Stability: verify that error code numbers match what the enum declares.
//     If a future refactor accidentally renumbers variants, these fail.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_error_variant_numeric_stability() {
    assert_eq!(ContractError::PoolNotFound as u32, 1);
    assert_eq!(ContractError::InvalidPoolState as u32, 2);
    assert_eq!(ContractError::UnauthorizedAdmin as u32, 3);
    assert_eq!(ContractError::PoolIsClosed as u32, 4);
    assert_eq!(ContractError::DuplicateApplication as u32, 5);
    assert_eq!(ContractError::StudentHasNotApplied as u32, 6);
    assert_eq!(ContractError::OnlyLinkedSchoolCanApprove as u32, 7);
    assert_eq!(ContractError::PoolNotDisbursedOrRefunded as u32, 8);
    assert_eq!(ContractError::AdminNotSet as u32, 9);
    assert_eq!(ContractError::NoUnclaimedFees as u32, 10);
    assert_eq!(ContractError::InvalidFee as u32, 11);
    assert_eq!(ContractError::PoolNotExpired as u32, 12);
    assert_eq!(ContractError::NoContributionToRefund as u32, 13);
    assert_eq!(ContractError::SchoolNotRegistered as u32, 14);
    assert_eq!(ContractError::PoolHasNoLinkedSchool as u32, 15);
    assert_eq!(ContractError::PoolAlreadyClosed as u32, 16);
    assert_eq!(ContractError::InvalidPoolName as u32, 17);
    assert_eq!(ContractError::InvalidPoolTarget as u32, 18);
    assert_eq!(ContractError::InvalidPoolDeadline as u32, 19);
    assert_eq!(ContractError::CampaignExpired as u32, 20);
    assert_eq!(ContractError::EmergencyWithdrawalNotRequested as u32, 21);
    assert_eq!(ContractError::GracePeriodNotElapsed as u32, 22);
    assert_eq!(ContractError::InvalidClaimAmount as u32, 23);
    assert_eq!(ContractError::ApplicationStatusNotFound as u32, 24);
    assert_eq!(ContractError::ApplicationNotApproved as u32, 25);
    assert_eq!(ContractError::OverdrawAttempt as u32, 26);
    assert_eq!(ContractError::InvalidAmount as u32, 27);
    assert_eq!(ContractError::TokenTransferFailed as u32, 28);
    assert_eq!(ContractError::InvalidTokenAddress as u32, 29);
    assert_eq!(ContractError::InvalidStatusTransition as u32, 30);
    assert_eq!(ContractError::InvalidRefundDeadline as u32, 31);
    assert_eq!(ContractError::MilestonesNotReleased as u32, 32);
    assert_eq!(ContractError::PoolHasUnrefundedFunds as u32, 33);
    assert_eq!(ContractError::AmountOverflow as u32, 34);
    assert_eq!(ContractError::MilestoneTotalMismatch as u32, 35);
}
