#![cfg(test)]

use crate::{
    base::errors::CrowdfundingError,
    base::types::ApplicationStatus,
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

/// Use save_pool so metrics.total_raised stays 0 and remaining_funds = target_amount.
fn create_pool(env: &Env, client: &CrowdfundingContractClient<'_>, _token: &Address) -> u64 {
    use crate::base::types::PoolMetadata;
    let creator = Address::generate(env);
    let deadline = env.ledger().timestamp() + 30 * 24 * 60 * 60;
    client.save_pool(
        &String::from_str(env, "Scholarship Fund"),
        &PoolMetadata {
            description: String::from_str(env, "Fund for student scholarships"),
            external_url: String::from_str(env, "https://example.com"),
            image_hash: String::from_str(env, "abc123"),
        },
        &creator,
        &100_000,
        &deadline,
        &None,
        &None,
    )
}

// ── basic apply / approve / reject ───────────────────────────────────────────

#[test]
fn test_apply_for_scholarship_success() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[1, 2, 3, 4]), &5_000);

    let app = client.get_application(&pool_id, &applicant);
    assert_eq!(app.status, ApplicationStatus::Pending);
    assert_eq!(app.pool_id, pool_id);
    assert_eq!(app.requested_amount, 5_000);
}

#[test]
fn test_approve_application_changes_status() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);
    let validator = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[5, 6, 7]), &10_000);
    client.approve_application(&pool_id, &applicant, &validator, &Some(String::from_str(&env, "Approved")));

    let app = client.get_application(&pool_id, &applicant);
    assert_eq!(app.status, ApplicationStatus::Approved);
    assert_eq!(app.reviewer.unwrap(), validator);
}

#[test]
fn test_reject_application_changes_status() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);
    let validator = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[9, 10, 11]), &15_000);
    client.reject_application(&pool_id, &applicant, &validator, &Some(String::from_str(&env, "Rejected")));

    let app = client.get_application(&pool_id, &applicant);
    assert_eq!(app.status, ApplicationStatus::Rejected);
    assert_eq!(app.reviewer.unwrap(), validator);
}

#[test]
fn test_apply_for_scholarship_empty_credentials_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);

    let result = client.try_apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[]), &5_000);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidApplicationCredentials)));
}

#[test]
fn test_apply_for_scholarship_duplicate_application_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);
    let creds = Bytes::from_array(&env, &[1, 2, 3, 4]);

    client.apply_for_scholarship(&pool_id, &applicant, &creds, &5_000);
    let result = client.try_apply_for_scholarship(&pool_id, &applicant, &creds, &5_000);
    assert_eq!(result, Err(Ok(CrowdfundingError::ApplicationAlreadySubmitted)));
}

#[test]
fn test_apply_for_scholarship_exceeds_remaining_funds_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);

    let result = client.try_apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[1, 2, 3, 4]), &150_000);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_apply_for_scholarship_zero_amount_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);

    let result = client.try_apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[1, 2, 3, 4]), &0);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_apply_for_scholarship_negative_amount_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);

    let result = client.try_apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[1, 2, 3, 4]), &-1_000);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_apply_for_scholarship_exactly_remaining_funds_succeeds() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);
    let applicant = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[1, 2, 3, 4]), &100_000);

    let app = client.get_application(&pool_id, &applicant);
    assert_eq!(app.status, ApplicationStatus::Pending);
    assert_eq!(app.requested_amount, 100_000);
}

#[test]
fn test_apply_for_scholarship_multiple_applicants_different_amounts() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);

    let a1 = Address::generate(&env);
    let a2 = Address::generate(&env);

    client.apply_for_scholarship(&pool_id, &a1, &Bytes::from_array(&env, &[1, 2, 3, 4]), &30_000);
    client.apply_for_scholarship(&pool_id, &a2, &Bytes::from_array(&env, &[5, 6, 7, 8]), &40_000);

    assert_eq!(client.get_application(&pool_id, &a1).requested_amount, 30_000);
    assert_eq!(client.get_application(&pool_id, &a2).requested_amount, 40_000);
}

// ── get_applications_paginated ───────────────────────────────────────────────

#[test]
fn paginated_empty_pool_returns_empty() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);

    assert_eq!(client.get_applications_paginated(&pool_id, &0, &10).len(), 0);
}

#[test]
fn paginated_offset_beyond_total_returns_empty() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);

    let a = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &a, &Bytes::from_array(&env, &[1]), &1_000);

    assert_eq!(client.get_applications_paginated(&pool_id, &5, &10).len(), 0);
}

#[test]
fn paginated_first_page_returns_correct_slice() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);

    for i in 0u8..5 {
        let a = Address::generate(&env);
        client.apply_for_scholarship(&pool_id, &a, &Bytes::from_array(&env, &[i + 1]), &1_000);
    }

    assert_eq!(client.get_applications_paginated(&pool_id, &0, &3).len(), 3);
}

#[test]
fn paginated_second_page_returns_remainder() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);

    for i in 0u8..5 {
        let a = Address::generate(&env);
        client.apply_for_scholarship(&pool_id, &a, &Bytes::from_array(&env, &[i + 1]), &1_000);
    }

    assert_eq!(client.get_applications_paginated(&pool_id, &3, &10).len(), 2);
}

#[test]
fn paginated_limit_exceeds_max_reverts() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);

    let result = client.try_get_applications_paginated(&pool_id, &0, &201);
    assert_eq!(result, Err(Ok(CrowdfundingError::VectorLimitExceeded)));
}

#[test]
fn paginated_unknown_pool_reverts() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let result = client.try_get_applications_paginated(&9999, &0, &10);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn paginated_limit_clamps_at_total() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);

    for i in 0u8..3 {
        let a = Address::generate(&env);
        client.apply_for_scholarship(&pool_id, &a, &Bytes::from_array(&env, &[i + 1]), &1_000);
    }

    assert_eq!(client.get_applications_paginated(&pool_id, &0, &100).len(), 3);
}

#[test]
fn paginated_results_match_individual_get_application() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&env, &client, &token);

    let applicant = Address::generate(&env);
    client.apply_for_scholarship(&pool_id, &applicant, &Bytes::from_array(&env, &[42]), &7_500);

    let page = client.get_applications_paginated(&pool_id, &0, &10);
    let single = client.get_application(&pool_id, &applicant);

    assert_eq!(page.len(), 1);
    assert_eq!(page.get(0).unwrap().requested_amount, single.requested_amount);
    assert_eq!(page.get(0).unwrap().status, single.status);
}
