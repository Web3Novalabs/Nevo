#![cfg(test)]

use crate::{
    base::{
        errors::ValidationError,
        types::{ApplicationStatus, PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{testutils::Address as _, token, Address, Env, String};

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

    let token_admin_client = token::StellarAssetClient::new(env, &token_address);
    token_admin_client.mint(&admin, &10_000_000i128);

    client.initialize(&admin, &token_address, &0);
    (client, admin, token_address)
}

fn create_pool(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token_address: &Address,
) -> (u64, Address) {
    let creator = Address::generate(env);
    let validator = Address::generate(env);

    let token_admin_client = token::StellarAssetClient::new(env, token_address);
    token_admin_client.mint(&creator, &100_000i128);

    let now = env.ledger().timestamp();
    let config = PoolConfig {
        name: String::from_str(env, "Scholarship Fund"),
        description: String::from_str(env, "Fund for student scholarships"),
        target_amount: 100_000i128,
        min_contribution: 0,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: now,
        application_deadline: now + 30 * 24 * 60 * 60,
        token_address: token_address.clone(),
        validator: validator.clone(),
    };

    let pool_id = client.create_pool(&creator, &config);
    (pool_id, validator)
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn test_apply_for_scholarship_success() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _validator) = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant);

    let application = client.get_application(&pool_id, &applicant);
    assert_eq!(application.status, ApplicationStatus::Pending);
    assert_eq!(application.pool_id, pool_id);
    assert_eq!(application.applicant, applicant);
}

#[test]
fn test_approve_application_changes_status() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _validator) = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant);
    client.approve_application(&(pool_id as u32), &applicant);

    let application = client.get_application(&pool_id, &applicant);
    assert_eq!(application.status, ApplicationStatus::Approved);
}

#[test]
fn test_reject_application_changes_status() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, validator) = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant);
    client.reject_application(&pool_id, &applicant, &validator);

    let application = client.get_application(&pool_id, &applicant);
    assert_eq!(application.status, ApplicationStatus::Rejected);
}

#[test]
fn test_apply_for_scholarship_duplicate_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _validator) = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant);

    let result = client.try_apply_for_scholarship(&pool_id, &applicant);
    assert_eq!(
        result,
        Err(Ok(ValidationError::ApplicationAlreadyExists)),
        "duplicate application must fail"
    );
}

#[test]
fn test_apply_for_scholarship_pool_not_found() {
    let env = Env::default();
    let (client, _, _) = setup(&env);
    let applicant = Address::generate(&env);

    let result = client.try_apply_for_scholarship(&999u64, &applicant);
    assert_eq!(
        result,
        Err(Ok(ValidationError::PoolNotFound)),
        "non-existent pool must fail"
    );
}

#[test]
fn test_approve_application_not_found() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _validator) = create_pool(&env, &client, &token_address);
    let student = Address::generate(&env);

    let result = client.try_approve_application(&(pool_id as u32), &student);
    assert_eq!(result, Err(Ok(ValidationError::ApplicationNotFound)));
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
fn test_reject_application_unauthorized() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _validator) = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let impostor = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant);

    let result = client.try_reject_application(&pool_id, &applicant, &impostor);
    assert_eq!(result, Err(Ok(ValidationError::Unauthorized)));
}

#[test]
fn test_approve_already_processed_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _validator) = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant);
    client.approve_application(&(pool_id as u32), &applicant);

    let result = client.try_approve_application(&(pool_id as u32), &applicant);
    assert_eq!(
        result,
        Err(Ok(ValidationError::ApplicationAlreadyProcessed))
    );
}

#[test]
fn test_get_application_not_found() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _) = create_pool(&env, &client, &token_address);
    let student = Address::generate(&env);

    let result = client.try_get_application(&pool_id, &student);
    assert_eq!(result, Err(Ok(ValidationError::ApplicationNotFound)));
}

#[test]
fn test_multiple_applicants_independent() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let (pool_id, _validator) = create_pool(&env, &client, &token_address);

    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant1);
    client.apply_for_scholarship(&pool_id, &applicant2);

    let app1 = client.get_application(&pool_id, &applicant1);
    let app2 = client.get_application(&pool_id, &applicant2);

    assert_eq!(app1.status, ApplicationStatus::Pending);
    assert_eq!(app2.status, ApplicationStatus::Pending);
    assert_eq!(app1.applicant, applicant1);
    assert_eq!(app2.applicant, applicant2);
}
