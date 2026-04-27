#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Bytes, Env, String};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, PoolState},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);
    (client, admin, token_address)
}

fn create_funded_pool(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token_address: &Address,
    target: i128,
) -> (u64, Address) {
    let sponsor = Address::generate(env);
    let token_admin_client = token::StellarAssetClient::new(env, token_address);
    token_admin_client.mint(&sponsor, &target);

    let config = PoolConfig {
        name: String::from_str(env, "Scholarship Pool"),
        description: String::from_str(env, "Test pool"),
        target_amount: target,
        min_contribution: 0,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        application_deadline: env.ledger().timestamp() + 30 * 24 * 60 * 60,
        token_address: token_address.clone(),
        validator: sponsor.clone(),
    };

    let pool_id = client.create_pool(&sponsor, &config);
    (pool_id, sponsor)
}

fn submit_and_approve(
    client: &CrowdfundingContractClient<'_>,
    pool_id: u64,
    validator: &Address,
    token_address: &Address,
    amount: i128,
) -> Address {
    let env = client.env();
    let applicant = Address::generate(env);
    let creds = Bytes::from_slice(env, b"credentials");

    client.apply_for_scholarship(&pool_id, &applicant, &creds, &amount);
    client.approve_application(&pool_id, &applicant, validator, &None);
    applicant
}

// ── get_pool_liquid_balance ──────────────────────────────────────────────────

#[test]
fn test_liquid_balance_equals_total_when_no_approvals() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, _sponsor) = create_funded_pool(&env, &client, &token_address, 10_000);

    let liquid = client.get_pool_liquid_balance(&pool_id).unwrap();
    assert_eq!(liquid, 10_000);
}

#[test]
fn test_liquid_balance_decreases_on_approval() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_funded_pool(&env, &client, &token_address, 10_000);

    submit_and_approve(&client, pool_id, &sponsor, &token_address, 3_000);

    let liquid = client.get_pool_liquid_balance(&pool_id).unwrap();
    assert_eq!(liquid, 7_000);
}

#[test]
fn test_liquid_balance_multiple_approvals() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_funded_pool(&env, &client, &token_address, 10_000);

    submit_and_approve(&client, pool_id, &sponsor, &token_address, 2_000);
    submit_and_approve(&client, pool_id, &sponsor, &token_address, 3_000);

    let liquid = client.get_pool_liquid_balance(&pool_id).unwrap();
    assert_eq!(liquid, 5_000);
}

#[test]
fn test_liquid_balance_pool_not_found() {
    let env = Env::default();
    let (client, _admin, _token_address) = setup(&env);

    let result = client.try_get_pool_liquid_balance(&999u64);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

// ── withdraw_unallocated ─────────────────────────────────────────────────────

#[test]
fn test_withdraw_unallocated_succeeds_within_liquid_balance() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_funded_pool(&env, &client, &token_address, 10_000);

    // Approve 3_000 — liquid = 7_000
    submit_and_approve(&client, pool_id, &sponsor, &token_address, 3_000);

    // Withdraw 5_000 (within liquid)
    client.withdraw_unallocated(&pool_id, &sponsor, &5_000i128);

    let liquid = client.get_pool_liquid_balance(&pool_id).unwrap();
    assert_eq!(liquid, 2_000);
}

#[test]
fn test_withdraw_unallocated_cannot_exceed_liquid_balance() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_funded_pool(&env, &client, &token_address, 10_000);

    // Approve 8_000 — liquid = 2_000
    submit_and_approve(&client, pool_id, &sponsor, &token_address, 8_000);

    // Attempt to withdraw 5_000 — must fail (would steal 3_000 from approved student)
    let result = client.try_withdraw_unallocated(&pool_id, &sponsor, &5_000i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::InsufficientBalance)));
}

#[test]
fn test_withdraw_unallocated_cannot_steal_fully_allocated_pool() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_funded_pool(&env, &client, &token_address, 10_000);

    // Approve entire pool
    submit_and_approve(&client, pool_id, &sponsor, &token_address, 10_000);

    // Any withdrawal must fail
    let result = client.try_withdraw_unallocated(&pool_id, &sponsor, &1i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::InsufficientBalance)));
}

#[test]
fn test_withdraw_unallocated_unauthorized_fails() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, _sponsor) = create_funded_pool(&env, &client, &token_address, 10_000);

    let stranger = Address::generate(&env);
    let result = client.try_withdraw_unallocated(&pool_id, &stranger, &1_000i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_withdraw_unallocated_zero_amount_fails() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_funded_pool(&env, &client, &token_address, 10_000);

    let result = client.try_withdraw_unallocated(&pool_id, &sponsor, &0i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_withdraw_unallocated_pool_not_found_fails() {
    let env = Env::default();
    let (client, _admin, _token_address) = setup(&env);

    let caller = Address::generate(&env);
    let result = client.try_withdraw_unallocated(&999u64, &caller, &100i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

