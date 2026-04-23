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

    let applicant = Address::generate(&env);
    let result = client.try_apply_for_scholarship(&pool_id, &applicant);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &applicant);
    assert_eq!(app.status, ApplicationStatus::Pending);
    assert_eq!(app.applicant, applicant);
    assert_eq!(app.pool_id, pool_id);
}

#[test]
fn test_apply_for_scholarship_pool_not_found() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let applicant = Address::generate(&env);
    let result = client.try_apply_for_scholarship(&999u64, &applicant);
    assert_eq!(result, Err(Ok(ValidationError::PoolNotFound)));
}

#[test]
fn test_apply_for_scholarship_duplicate_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);

    let result = client.try_apply_for_scholarship(&pool_id, &applicant);
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

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);

    let result = client.try_approve_application(&pool_id, &applicant, &validator);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &applicant);
    assert_eq!(app.status, ApplicationStatus::Approved);
}

#[test]
fn test_approve_application_wrong_validator_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);

    let impostor = Address::generate(&env);
    let result = client.try_approve_application(&pool_id, &applicant, &impostor);
    assert_eq!(result, Err(Ok(ValidationError::Unauthorized)));
}

#[test]
fn test_approve_application_not_found_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    let result = client.try_approve_application(&pool_id, &applicant, &validator);
    assert_eq!(result, Err(Ok(ValidationError::ApplicationNotFound)));
}

#[test]
fn test_approve_already_processed_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);
    client.approve_application(&pool_id, &applicant, &validator);

    // Attempt to approve again
    let result = client.try_approve_application(&pool_id, &applicant, &validator);
    assert_eq!(
        result,
        Err(Ok(ValidationError::ApplicationAlreadyProcessed))
    );
}

// ── reject_application ────────────────────────────────────────────────────────

#[test]
fn test_reject_application_success() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);

    let result = client.try_reject_application(&pool_id, &applicant, &validator);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &applicant);
    assert_eq!(app.status, ApplicationStatus::Rejected);
}

#[test]
fn test_reject_application_wrong_validator_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);

    let impostor = Address::generate(&env);
    let result = client.try_reject_application(&pool_id, &applicant, &impostor);
    assert_eq!(result, Err(Ok(ValidationError::Unauthorized)));
}

#[test]
fn test_reject_already_processed_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);
    client.reject_application(&pool_id, &applicant, &validator);

    // Attempt to reject again
    let result = client.try_reject_application(&pool_id, &applicant, &validator);
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

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);
    client.approve_application(&pool_id, &applicant, &validator);

    // Cannot reject an already-approved application
    let result = client.try_reject_application(&pool_id, &applicant, &validator);
    assert_eq!(
        result,
        Err(Ok(ValidationError::ApplicationAlreadyProcessed))
    );
}

// ── get_application ───────────────────────────────────────────────────────────

#[test]
fn test_get_application_not_found() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    let result = client.try_get_application(&pool_id, &applicant);
    assert_eq!(result, Err(Ok(ValidationError::ApplicationNotFound)));
}

// ── auth enforcement ──────────────────────────────────────────────────────────

/// Verifies that unauthenticated accounts cannot approve applications.
/// With mock_all_auths disabled, require_auth() will panic for any address
/// that hasn't been explicitly authorized.
#[test]
#[should_panic]
fn test_approve_panics_without_auth() {
    let env = Env::default();
    // Do NOT call env.mock_all_auths() — auth is enforced
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    // Initialize with mocked auth just for setup
    env.mock_all_auths();
    client.initialize(&admin, &token, &0);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);

    // Clear mocked auths — now auth is enforced
    env.set_auths(&[]);

    // This must panic because validator has not signed
    client.approve_application(&pool_id, &applicant, &validator);
}

/// Verifies that unauthenticated accounts cannot reject applications.
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

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant);

    env.set_auths(&[]);

    // This must panic because validator has not signed
    client.reject_application(&pool_id, &applicant, &validator);
}
