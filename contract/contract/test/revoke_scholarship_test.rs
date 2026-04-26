#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Bytes, Env, String};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{ApplicationStatus, PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

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

fn create_funded_pool(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token_address: &Address,
    validator: &Address,
) -> u64 {
    let sponsor = Address::generate(env);
    let token_admin_client = token::StellarAssetClient::new(env, token_address);
    token_admin_client.mint(&sponsor, &100_000i128);

    let config = PoolConfig {
        name: String::from_str(env, "Scholarship Pool"),
        description: String::from_str(env, "Test pool"),
        target_amount: 100_000,
        min_contribution: 0,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: validator.clone(),
    };

    client.create_pool(&sponsor, &config)
}

fn apply_and_approve(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    pool_id: u64,
    validator: &Address,
) -> Address {
    let student = Address::generate(env);
    let credentials = Bytes::from_array(env, &[1, 2, 3]);
    client.apply_for_scholarship(&pool_id, &student, &credentials, &10_000i128);
    client.approve_application(&pool_id, &student, validator, &None);
    student
}

#[test]
fn test_revoke_scholarship_sets_revoked_status() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let validator = Address::generate(&env);
    let pool_id = create_funded_pool(&env, &client, &token_address, &validator);
    let student = apply_and_approve(&env, &client, pool_id, &validator);

    client.revoke_scholarship(&pool_id, &student, &validator);

    let app = client.get_application(&pool_id, &student);
    assert_eq!(app.status, ApplicationStatus::Revoked);
}

#[test]
fn test_revoke_scholarship_unlocks_allocated_funds() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let validator = Address::generate(&env);
    let pool_id = create_funded_pool(&env, &client, &token_address, &validator);
    let student = apply_and_approve(&env, &client, pool_id, &validator);

    // Before revoke: liquid = total - allocated
    let liquid_before = client.get_pool_liquid_balance(&pool_id);

    client.revoke_scholarship(&pool_id, &student, &validator);

    // After revoke: allocated decreases, liquid increases by requested_amount
    let liquid_after = client.get_pool_liquid_balance(&pool_id);
    assert_eq!(liquid_after, liquid_before + 10_000i128);
}

#[test]
fn test_revoke_scholarship_unauthorized_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let validator = Address::generate(&env);
    let pool_id = create_funded_pool(&env, &client, &token_address, &validator);
    let student = apply_and_approve(&env, &client, pool_id, &validator);

    let stranger = Address::generate(&env);
    let result = client.try_revoke_scholarship(&pool_id, &student, &stranger);
    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_revoke_scholarship_not_approved_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let validator = Address::generate(&env);
    let pool_id = create_funded_pool(&env, &client, &token_address, &validator);

    // Apply but do NOT approve — status is Pending
    let student = Address::generate(&env);
    let credentials = Bytes::from_array(&env, &[1, 2, 3]);
    client.apply_for_scholarship(&pool_id, &student, &credentials, &5_000i128);

    let result = client.try_revoke_scholarship(&pool_id, &student, &validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::ApplicationAlreadyReviewed)));
}

#[test]
fn test_revoke_scholarship_not_found_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);
    let validator = Address::generate(&env);
    let pool_id = create_funded_pool(&env, &client, &token_address, &validator);

    let ghost = Address::generate(&env);
    let result = client.try_revoke_scholarship(&pool_id, &ghost, &validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::ApplicationNotFound)));
}

#[test]
fn test_revoke_scholarship_pool_not_found_fails() {
    let env = Env::default();
    let (client, _, _) = setup(&env);
    let validator = Address::generate(&env);
    let student = Address::generate(&env);

    let result = client.try_revoke_scholarship(&999u64, &student, &validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}
