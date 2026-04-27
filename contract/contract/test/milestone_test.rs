#![cfg(test)]

use crate::{
    base::{
        errors::{CrowdfundingError, SecondCrowdfundingError},
        types::{ApplicationStatus, Milestone, PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String, Vec,
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    // Mint tokens so the pool creator can deposit target_amount on create_pool
    let token_admin_client = token::StellarAssetClient::new(env, &token_address);
    token_admin_client.mint(&admin, &10_000_000i128);

    client.initialize(&admin, &token_address, &0);
    (client, admin, token_address)
}

/// Creates a pool and returns (pool_id, validator_address).
/// The creator is minted enough tokens to cover the target_amount deposit.
fn create_funded_pool(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token_address: &Address,
    target_amount: i128,
) -> (u64, Address, Address) {
    let creator = Address::generate(env);
    let validator = Address::generate(env);

    let token_admin_client = token::StellarAssetClient::new(env, token_address);
    token_admin_client.mint(&creator, &target_amount);

    let now = env.ledger().timestamp();
    let config = PoolConfig {
        name: String::from_str(env, "Milestone Test Pool"),
        description: String::from_str(env, "Pool for milestone tests"),
        target_amount,
        min_contribution: 0,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: now,
        application_deadline: now + 30 * 24 * 60 * 60,
        token_address: token_address.clone(),
        validator: validator.clone(),
    };

    let pool_id = client.create_pool(&creator, &config);
    (pool_id, validator, creator)
}

/// Submits and approves a scholarship application, returning the applicant address.
fn apply_and_approve(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    pool_id: u64,
    requested_amount: i128,
) -> Address {
    let applicant = Address::generate(env);
    client.apply_for_scholarship(&pool_id, &applicant);
    client.approve_application(&(pool_id as u32), &applicant);

    // Seed the ApplicationDetails record that setup_application_milestones reads.
    // The simple apply_for_scholarship only writes ScholarshipApplication;
    // we also need an ApplicationDetails entry with the requested_amount.
    use crate::base::types::{ApplicationDetails, ApplicationStatus, StorageKey};
    use soroban_sdk::Bytes;
    let app_key = StorageKey::Application(pool_id, applicant.clone());
    let details = ApplicationDetails {
        pool_id,
        applicant: applicant.clone(),
        credentials: Bytes::from_array(env, &[1, 2, 3]),
        requested_amount,
        submitted_at: env.ledger().timestamp(),
        status: ApplicationStatus::Approved,
        reviewer: None,
        review_note: None,
        milestones: Vec::new(env),
        amount_claimed: 0,
    };
    env.as_contract(&client.address, || {
        env.storage().instance().set(&app_key, &details);
    });

    applicant
}

// ── setup_application_milestones: happy path ──────────────────────────────────

#[test]
fn test_setup_milestones_success() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let requested = 90_000i128;
    let (pool_id, _validator, _creator) =
        create_funded_pool(&env, &client, &token_address, requested);
    let applicant = apply_and_approve(&env, &client, pool_id, requested);

    // Build three milestones that sum exactly to requested_amount
    let now = env.ledger().timestamp();
    let milestones = Vec::from_array(
        &env,
        [
            Milestone { unlock_date: now + 30 * 86_400, unlocked: false, amount: 30_000 },
            Milestone { unlock_date: now + 60 * 86_400, unlocked: false, amount: 30_000 },
            Milestone { unlock_date: now + 90 * 86_400, unlocked: false, amount: 30_000 },
        ],
    );

    let result = client.try_setup_application_milestones(&pool_id, &applicant, &milestones);
    assert_eq!(result, Ok(Ok(())), "setup_application_milestones must succeed");

    // Verify the milestones were persisted
    let details = client.get_application_details(&pool_id, &applicant);
    assert_eq!(details.milestones.len(), 3);
    assert_eq!(details.milestones.get(0).unwrap().amount, 30_000);
    assert_eq!(details.milestones.get(1).unwrap().amount, 30_000);
    assert_eq!(details.milestones.get(2).unwrap().amount, 30_000);
    assert_eq!(details.amount_claimed, 0);
}

// ── setup_application_milestones: amount mismatch ─────────────────────────────

#[test]
fn test_setup_milestones_amount_mismatch_reverts() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let requested = 90_000i128;
    let (pool_id, _validator, _creator) =
        create_funded_pool(&env, &client, &token_address, requested);
    let applicant = apply_and_approve(&env, &client, pool_id, requested);

    let now = env.ledger().timestamp();
    // Sum is 80_000, not 90_000 — must revert
    let milestones = Vec::from_array(
        &env,
        [
            Milestone { unlock_date: now + 30 * 86_400, unlocked: false, amount: 40_000 },
            Milestone { unlock_date: now + 60 * 86_400, unlocked: false, amount: 40_000 },
        ],
    );

    let result = client.try_setup_application_milestones(&pool_id, &applicant, &milestones);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::MilestoneAmountMismatch)),
        "mismatched sum must revert with MilestoneAmountMismatch"
    );
}

// ── setup_application_milestones: empty vector ────────────────────────────────

#[test]
fn test_setup_milestones_empty_reverts() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let requested = 50_000i128;
    let (pool_id, _validator, _creator) =
        create_funded_pool(&env, &client, &token_address, requested);
    let applicant = apply_and_approve(&env, &client, pool_id, requested);

    let empty: Vec<Milestone> = Vec::new(&env);
    let result = client.try_setup_application_milestones(&pool_id, &applicant, &empty);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::EmptyMilestones)),
        "empty milestone list must revert"
    );
}

// ── setup_application_milestones: idempotency guard ───────────────────────────

#[test]
fn test_setup_milestones_cannot_overwrite() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let requested = 60_000i128;
    let (pool_id, _validator, _creator) =
        create_funded_pool(&env, &client, &token_address, requested);
    let applicant = apply_and_approve(&env, &client, pool_id, requested);

    let now = env.ledger().timestamp();
    let milestones = Vec::from_array(
        &env,
        [
            Milestone { unlock_date: now + 30 * 86_400, unlocked: false, amount: 30_000 },
            Milestone { unlock_date: now + 60 * 86_400, unlocked: false, amount: 30_000 },
        ],
    );

    // First call succeeds
    client.setup_application_milestones(&pool_id, &applicant, &milestones);

    // Second call must revert
    let result = client.try_setup_application_milestones(&pool_id, &applicant, &milestones);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::MilestonesAlreadySet)),
        "second setup call must revert with MilestonesAlreadySet"
    );
}

// ── setup_application_milestones: pool not found ──────────────────────────────

#[test]
fn test_setup_milestones_pool_not_found() {
    let env = Env::default();
    let (client, _, _) = setup(&env);
    let applicant = Address::generate(&env);
    let now = env.ledger().timestamp();
    let milestones = Vec::from_array(
        &env,
        [Milestone { unlock_date: now + 86_400, unlocked: false, amount: 1_000 }],
    );

    let result = client.try_setup_application_milestones(&999u64, &applicant, &milestones);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::AppNotFound)),
        "non-existent pool must revert"
    );
}

// ── setup_application_milestones: application not found ───────────────────────

#[test]
fn test_setup_milestones_application_not_found() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _validator, _creator) =
        create_funded_pool(&env, &client, &token_address, 50_000);

    let unknown = Address::generate(&env);
    let now = env.ledger().timestamp();
    let milestones = Vec::from_array(
        &env,
        [Milestone { unlock_date: now + 86_400, unlocked: false, amount: 50_000 }],
    );

    let result = client.try_setup_application_milestones(&pool_id, &unknown, &milestones);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::AppNotFound)),
        "missing application must revert"
    );
}

// ── setup_application_milestones: zero-amount milestone ───────────────────────

#[test]
fn test_setup_milestones_zero_amount_reverts() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let requested = 50_000i128;
    let (pool_id, _validator, _creator) =
        create_funded_pool(&env, &client, &token_address, requested);
    let applicant = apply_and_approve(&env, &client, pool_id, requested);

    let now = env.ledger().timestamp();
    // One milestone has amount 0 — invalid
    let milestones = Vec::from_array(
        &env,
        [
            Milestone { unlock_date: now + 30 * 86_400, unlocked: false, amount: 0 },
            Milestone { unlock_date: now + 60 * 86_400, unlocked: false, amount: 50_000 },
        ],
    );

    let result = client.try_setup_application_milestones(&pool_id, &applicant, &milestones);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::InvalidApplicationCredentials)),
        "zero-amount milestone must revert"
    );
}

// ── Milestone struct properties ───────────────────────────────────────────────

#[test]
fn test_milestone_struct_properties() {
    let milestone = Milestone {
        unlock_date: 1_700_000_000,
        unlocked: false,
        amount: 1_000,
    };
    assert_eq!(milestone.unlock_date, 1_700_000_000);
    assert!(!milestone.unlocked);
    assert_eq!(milestone.amount, 1_000);
}

// ── add_milestone / unlock_milestone (existing flow) ─────────────────────────

#[test]
fn test_add_and_unlock_milestone_flow() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let requested = 50_000i128;
    let (pool_id, _validator, _creator) =
        create_funded_pool(&env, &client, &token_address, requested);
    let applicant = apply_and_approve(&env, &client, pool_id, requested);

    let unlock_date = env.ledger().timestamp() + 86_400;

    // add_milestone
    let add_result = client.try_add_milestone(&pool_id, &applicant, &unlock_date, &5_000i128);
    assert_eq!(add_result, Ok(Ok(())));

    // unlock before date — must fail
    let early = client.try_unlock_milestone(&pool_id, &applicant, &0u32);
    assert_eq!(
        early,
        Err(Ok(SecondCrowdfundingError::InvalidApplicationCredentials)),
        "unlock before date must fail"
    );

    // advance time past unlock_date
    env.ledger().with_mut(|li| li.timestamp = unlock_date + 1);

    // unlock after date — must succeed
    let late = client.try_unlock_milestone(&pool_id, &applicant, &0u32);
    assert_eq!(late, Ok(Ok(())));

    // verify amount_claimed updated
    let details = client.get_application_details(&pool_id, &applicant);
    assert_eq!(details.amount_claimed, 5_000);
    assert!(details.milestones.get(0).unwrap().unlocked);
}
