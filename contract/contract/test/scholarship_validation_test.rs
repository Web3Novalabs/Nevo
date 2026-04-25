#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Bytes, Env, String};

use crate::{
    base::{
        errors::SecondCrowdfundingError,
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
        .register_stellar_asset_contract_v2(token_admin.clone())
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
    let token_admin_client = token::StellarAssetClient::new(env, token);
    token_admin_client.mint(creator, &1_000_000i128);

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
        application_deadline: 0,
    };
    client.create_pool(creator, &config)
}

fn creds(env: &Env) -> Bytes {
    Bytes::from_slice(env, b"test_credentials")
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
    let result = client.try_apply_for_scholarship(&pool_id, &student, &creds(&env), &100_000i128);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &student);
    assert_eq!(app.status, ApplicationStatus::Pending);
    assert_eq!(app.applicant, student);
    assert_eq!(app.pool_id, pool_id);
}

#[test]
fn test_apply_for_scholarship_pool_not_found() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let student = Address::generate(&env);
    let result = client.try_apply_for_scholarship(&999u64, &student, &creds(&env), &100_000i128);
    assert_eq!(result, Err(Ok(SecondCrowdfundingError::PoolNotFound)));
}

#[test]
fn test_apply_for_scholarship_duplicate_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student, &creds(&env), &100_000i128);

    let result = client.try_apply_for_scholarship(&pool_id, &student, &creds(&env), &100_000i128);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::ApplicationAlreadySubmitted))
    );
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
    client.apply_for_scholarship(&pool_id, &student, &creds(&env), &100_000i128);

    let result = client.try_approve_application(&pool_id, &student, &validator, &None);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &student);
    assert_eq!(app.status, ApplicationStatus::Approved);
}

#[test]
fn test_approve_application_not_found_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    let result = client.try_approve_application(&pool_id, &student, &validator, &None);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::ApplicationNotFound))
    );
}

#[test]
fn test_approve_already_processed_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student, &creds(&env), &100_000i128);
    client.approve_application(&pool_id, &student, &validator, &None);

    let result = client.try_approve_application(&pool_id, &student, &validator, &None);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::ApplicationAlreadyReviewed))
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

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student, &creds(&env), &100_000i128);

    let result = client.try_reject_application(&pool_id, &student, &validator, &None);
    assert_eq!(result, Ok(Ok(())));

    let app = client.get_application(&pool_id, &student);
    assert_eq!(app.status, ApplicationStatus::Rejected);
}

#[test]
fn test_reject_already_processed_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);

    let student = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &student, &creds(&env), &100_000i128);
    client.reject_application(&pool_id, &student, &validator, &None);

    let result = client.try_reject_application(&pool_id, &student, &validator, &None);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::ApplicationAlreadyReviewed))
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

    let student = Address::generate(&env);
    let result = client.try_get_application(&pool_id, &student);
    assert_eq!(
        result,
        Err(Ok(SecondCrowdfundingError::ApplicationNotFound))
    );
}
