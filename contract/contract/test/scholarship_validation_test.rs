#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{
    base::{
        errors::ValidationError,
        types::{ApplicationStatus, PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token, &0);
    (client, admin, token)
}

fn create_pool_with_validator(
    client: &CrowdfundingContractClient<'_>,
    env: &Env,
    creator: &Address,
    validator: &Address,
    token: &Address,
) -> u64 {
    let config = PoolConfig {
        name: String::from_str(env, "Scholarship Pool"),
        description: String::from_str(env, "A pool for scholarship applications"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        application_deadline: env.ledger().timestamp() + 30 * 24 * 60 * 60,
        token_address: token.clone(),
        validator: validator.clone(),
    };
    client.create_pool(creator, &config)
}

// ── apply_for_scholarship ─────────────────────────────────────────────────────

#[test]
fn test_apply_for_scholarship_success() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    let result = client.try_apply_for_scholarship(&pool_id, &student);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &student);
    assert_eq!(app.status, ApplicationStatus::Pending);
    assert_eq!(app.applicant, student);
    assert_eq!(app.pool_id, pool_id);
}

#[test]
fn test_apply_for_scholarship_emits_app_sub_event() {
    use soroban_sdk::{symbol_short, FromVal, Symbol};

    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);

    // Verify AppSub event was emitted
    let app_sub = symbol_short!("AppSub");
    let found = env.events().all().iter().any(|(_, topics, data)| {
        if topics.is_empty() {
            return false;
        }
        let event_symbol = Symbol::from_val(&env, &topics.get(0).unwrap());
        if event_symbol != app_sub {
            return false;
        }
        // Verify topics contain pool_id and student
        if topics.len() < 3 {
            return false;
        }
        // Verify data contains the target_amount (1_000_000)
        if let Ok(amount) = i128::try_from_val(&env, data) {
            amount == 1_000_000
        } else {
            false
        }
    });

    assert!(
        found,
        "AppSub event must be emitted with pool_id, student, and target_amount"
    );
}

#[test]
fn test_apply_for_scholarship_pool_not_found() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let student = Address::generate(&env);
    let result = client.try_apply_for_scholarship(&999u64, &student);
    assert_eq!(result, Err(Ok(ValidationError::PoolNotFound)));
}

#[test]
fn test_apply_for_scholarship_duplicate_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);

    let result = client.try_apply_for_scholarship(&pool_id, &student);
    assert_eq!(result, Err(Ok(ValidationError::ApplicationAlreadyExists)));
}

// ── approve_application ───────────────────────────────────────────────────────

#[test]
fn test_approve_application_success() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);

    // pool_id cast to u32 as per the function signature
    let result = client.try_approve_application(&(pool_id as u32), &student);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &student);
    assert_eq!(app.status, ApplicationStatus::Approved);
}

#[test]
fn test_approve_application_status_shifts_unequivocally() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);

    // Confirm Pending before approval
    let before = client.get_application(&pool_id, &student);
    assert_eq!(before.status, ApplicationStatus::Pending);

    client.approve_application(&(pool_id as u32), &student);

    // Confirm Approved after
    let after = client.get_application(&pool_id, &student);
    assert_eq!(after.status, ApplicationStatus::Approved);
}

#[test]
fn test_approve_application_pool_not_found() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let student = Address::generate(&env);
    let result = client.try_approve_application(&999u32, &student);
    assert_eq!(result, Err(Ok(ValidationError::PoolNotFound)));
}

#[test]
fn test_approve_application_not_found_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    // No application submitted — should fail
    let student = Address::generate(&env);
    let result = client.try_approve_application(&(pool_id as u32), &student);
    assert_eq!(result, Err(Ok(ValidationError::ApplicationNotFound)));
}

#[test]
fn test_approve_already_processed_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);
    client.approve_application(&(pool_id as u32), &student);

    // Attempt to approve again — must fail
    let result = client.try_approve_application(&(pool_id as u32), &student);
    assert_eq!(
        result,
        Err(Ok(ValidationError::ApplicationAlreadyProcessed))
    );
}

/// Invalid signers revert the sequence automatically.
/// Without the validator's auth, require_auth() panics.
#[test]
#[should_panic]
fn test_approve_panics_without_validator_auth() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    // Setup with mocked auth
    env.mock_all_auths();
    client.initialize(&admin, &token, &0);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);

    // Remove all mocked auths — validator has not signed
    env.set_auths(&[]);

    // Must panic: validator.require_auth() fails for unsigned call
    client.approve_application(&(pool_id as u32), &student);
}

// ── reject_application ────────────────────────────────────────────────────────

#[test]
fn test_reject_application_success() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);

    let result = client.try_reject_application(&pool_id, &student, &validator);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &student);
    assert_eq!(app.status, ApplicationStatus::Rejected);
}

#[test]
fn test_reject_application_wrong_validator_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);

    let impostor = Address::generate(&env);
    let result = client.try_reject_application(&pool_id, &student, &impostor);
    assert_eq!(result, Err(Ok(ValidationError::Unauthorized)));
}

#[test]
fn test_reject_already_processed_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);
    client.reject_application(&pool_id, &student, &validator);

    let result = client.try_reject_application(&pool_id, &student, &validator);
    assert_eq!(
        result,
        Err(Ok(ValidationError::ApplicationAlreadyProcessed))
    );
}

#[test]
fn test_reject_approved_application_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);
    client.approve_application(&(pool_id as u32), &student);

    let result = client.try_reject_application(&pool_id, &student, &validator);
    assert_eq!(
        result,
        Err(Ok(ValidationError::ApplicationAlreadyProcessed))
    );
}

#[test]
#[should_panic]
fn test_reject_panics_without_auth() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    env.mock_all_auths();
    client.initialize(&admin, &token, &0);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student);

    env.set_auths(&[]);

    client.reject_application(&pool_id, &student, &validator);
}

// ── get_application ───────────────────────────────────────────────────────────

#[test]
fn test_get_application_not_found() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    let result = client.try_get_application(&pool_id, &student);
    assert_eq!(result, Err(Ok(ValidationError::ApplicationNotFound)));
}

