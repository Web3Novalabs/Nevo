#![cfg(test)]

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{ApplicationStatus, PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token_address, &0);
    (client, admin, token_address)
}

fn create_pool(env: &Env, client: &CrowdfundingContractClient<'_>, token_address: &Address) -> u64 {
    let creator = Address::generate(env);
    let config = PoolConfig {
        name: String::from_str(env, "Scholarship Fund"),
        description: String::from_str(env, "Fund for student scholarships"),
        target_amount: 100_000i128,
        min_contribution: 0,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: admin.clone(),
    };

    client.create_pool(&creator, &config)
}

#[test]
fn test_apply_for_scholarship_success() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[1, 2, 3, 4]);
    let requested_amount = 5_000i128;

    client.apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);

    let application = client.get_application(&pool_id, &applicant);
    assert_eq!(application.status, ApplicationStatus::Pending);
    assert_eq!(application.pool_id, pool_id);
    assert_eq!(application.requested_amount, requested_amount);
}

#[test]
fn test_approve_application_changes_status() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let validator = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[5, 6, 7]);
    let requested_amount = 10_000i128;

    client.apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);
    client.approve_application(
        &pool_id,
        &applicant,
        &validator,
        &Some(String::from_str(&env, "Approved")),
    );

    let application = client.get_application(&pool_id, &applicant);
    assert_eq!(application.status, ApplicationStatus::Approved);
    assert_eq!(application.reviewer.unwrap(), validator);
}

#[test]
fn test_reject_application_changes_status() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let validator = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[9, 10, 11]);
    let requested_amount = 15_000i128;

    client.apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);
    client.reject_application(
        &pool_id,
        &applicant,
        &validator,
        &Some(String::from_str(&env, "Rejected")),
    );

    let application = client.get_application(&pool_id, &applicant);
    assert_eq!(application.status, ApplicationStatus::Rejected);
    assert_eq!(application.reviewer.unwrap(), validator);
}

#[test]
fn test_apply_for_scholarship_empty_credentials_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[]);
    let requested_amount = 5_000i128;

    let result =
        client.try_apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InvalidApplicationCredentials))
    );
}

#[test]
fn test_apply_for_scholarship_duplicate_application_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[1, 2, 3, 4]);
    let requested_amount = 5_000i128;

    // First application should succeed
    client.apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);

    // Second application from same applicant should fail with ApplicationAlreadySubmitted
    let result =
        client.try_apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::ApplicationAlreadySubmitted))
    );
}

#[test]
fn test_apply_for_scholarship_exceeds_remaining_funds_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[1, 2, 3, 4]);

    // Pool target is 100_000, so requesting more should fail
    let requested_amount = 150_000i128;

    let result =
        client.try_apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_apply_for_scholarship_zero_amount_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[1, 2, 3, 4]);
    let requested_amount = 0i128;

    let result =
        client.try_apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_apply_for_scholarship_negative_amount_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[1, 2, 3, 4]);
    let requested_amount = -1000i128;

    let result =
        client.try_apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_apply_for_scholarship_exactly_remaining_funds_succeeds() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[1, 2, 3, 4]);

    // Pool target is 100_000, requesting exactly that should succeed
    let requested_amount = 100_000i128;

    client.apply_for_scholarship(&pool_id, &applicant, &credentials, &requested_amount);

    let application = client.get_application(&pool_id, &applicant);
    assert_eq!(application.status, ApplicationStatus::Pending);
    assert_eq!(application.requested_amount, requested_amount);
}

#[test]
fn test_apply_for_scholarship_multiple_applicants_different_amounts() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let pool_id = create_pool(&env, &client, &token_address);

    let applicant1 = Address::generate(&env);
    let applicant2 = Address::generate(&env);
    let credentials1 = Bytes::from_array(&env, &[1, 2, 3, 4]);
    let credentials2 = Bytes::from_array(&env, &[5, 6, 7, 8]);

    let amount1 = 30_000i128;
    let amount2 = 40_000i128;

    // Both applications should succeed
    client.apply_for_scholarship(&pool_id, &applicant1, &credentials1, &amount1);
    client.apply_for_scholarship(&pool_id, &applicant2, &credentials2, &amount2);

    let app1 = client.get_application(&pool_id, &applicant1);
    let app2 = client.get_application(&pool_id, &applicant2);

    assert_eq!(app1.requested_amount, amount1);
    assert_eq!(app2.requested_amount, amount2);
}
