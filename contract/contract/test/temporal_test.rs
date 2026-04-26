#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, BytesN, Env, String,
};

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(token_admin).address();
    client.initialize(&admin, &token, &0);
    (client, admin, token)
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn campaign_id(env: &Env, seed: u8) -> BytesN<32> {
    let mut b = [0u8; 32];
    b[0] = seed;
    BytesN::from_array(env, &b)
}

// ── donate: deadline boundary ────────────────────────────────────────────────

#[test]
fn donate_before_deadline_succeeds() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let id = campaign_id(&env, 1);
    let donor = Address::generate(&env);
    let deadline = 1_000_000 + 86_400;

    client.create_campaign(&id, &String::from_str(&env, "T"), &donor, &1_000, &deadline, &token);
    mint(&env, &token, &donor, 100);

    client.donate(&id, &donor, &token, &100);
}

#[test]
fn donate_at_deadline_reverts() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let id = campaign_id(&env, 2);
    let donor = Address::generate(&env);
    let deadline = 1_000_000 + 86_400;

    client.create_campaign(&id, &String::from_str(&env, "T"), &donor, &1_000, &deadline, &token);
    mint(&env, &token, &donor, 100);

    env.ledger().with_mut(|l| l.timestamp = deadline);
    let result = client.try_donate(&id, &donor, &token, &100);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignExpired)));
}

#[test]
fn donate_after_deadline_reverts() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let id = campaign_id(&env, 3);
    let donor = Address::generate(&env);
    let deadline = 1_000_000 + 86_400;

    client.create_campaign(&id, &String::from_str(&env, "T"), &donor, &1_000, &deadline, &token);
    mint(&env, &token, &donor, 100);

    env.ledger().with_mut(|l| l.timestamp = deadline + 1);
    let result = client.try_donate(&id, &donor, &token, &100);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignExpired)));
}

// ── get_pool_remaining_time ──────────────────────────────────────────────────

#[test]
fn remaining_time_before_deadline() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    mint(&env, &token, &creator, 1_000);
    let pool_id = client.create_pool(&creator, &PoolConfig {
        name: String::from_str(&env, "P"),
        description: String::from_str(&env, "D"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 500,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: creator.clone(),
    });

    assert_eq!(client.get_pool_remaining_time(&pool_id), 500);
}

#[test]
fn remaining_time_at_deadline_returns_zero() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    mint(&env, &token, &creator, 1_000);
    let pool_id = client.create_pool(&creator, &PoolConfig {
        name: String::from_str(&env, "P"),
        description: String::from_str(&env, "D"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 500,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: creator.clone(),
    });

    env.ledger().with_mut(|l| l.timestamp = 1_000_500);
    assert_eq!(client.get_pool_remaining_time(&pool_id), 0);
}

#[test]
fn remaining_time_past_deadline_no_underflow() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    mint(&env, &token, &creator, 1_000);
    let pool_id = client.create_pool(&creator, &PoolConfig {
        name: String::from_str(&env, "P"),
        description: String::from_str(&env, "D"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 500,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: creator.clone(),
    });

    env.ledger().with_mut(|l| l.timestamp = 2_000_000);
    assert_eq!(client.get_pool_remaining_time(&pool_id), 0);
}

// ── refund: time-lock boundaries ─────────────────────────────────────────────

#[test]
fn refund_before_deadline_reverts() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    mint(&env, &token, &creator, 1_000);
    mint(&env, &token, &contributor, 100);

    let pool_id = client.create_pool(&creator, &PoolConfig {
        name: String::from_str(&env, "P"),
        description: String::from_str(&env, "D"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: creator.clone(),
    });

    client.contribute(&pool_id, &contributor, &token, &100, &false);

    // still before deadline
    env.ledger().with_mut(|l| l.timestamp = 1_000_000 + 86_399);
    let result = client.try_refund(&pool_id, &contributor);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotExpired)));
}

#[test]
fn refund_in_grace_period_reverts() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    mint(&env, &token, &creator, 1_000);
    mint(&env, &token, &contributor, 100);

    let pool_id = client.create_pool(&creator, &PoolConfig {
        name: String::from_str(&env, "P"),
        description: String::from_str(&env, "D"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: creator.clone(),
    });

    client.contribute(&pool_id, &contributor, &token, &100, &false);

    // past deadline but inside 7-day grace period
    let deadline = 1_000_000 + 86_400;
    env.ledger().with_mut(|l| l.timestamp = deadline + 1);
    let result = client.try_refund(&pool_id, &contributor);
    assert_eq!(result, Err(Ok(CrowdfundingError::RefundGracePeriodNotPassed)));
}

#[test]
fn refund_after_grace_period_succeeds() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    mint(&env, &token, &creator, 1_000);
    mint(&env, &token, &contributor, 100);

    let pool_id = client.create_pool(&creator, &PoolConfig {
        name: String::from_str(&env, "P"),
        description: String::from_str(&env, "D"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: creator.clone(),
    });

    client.contribute(&pool_id, &contributor, &token, &100, &false);

    // deadline + 7 days grace period
    env.ledger().with_mut(|l| l.timestamp = 1_000_000 + 86_400 + 604_800);
    client.refund(&pool_id, &contributor);
}

// ── extend_campaign_deadline ─────────────────────────────────────────────────

#[test]
fn extend_deadline_below_current_reverts() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    let id = campaign_id(&env, 10);
    let deadline = 1_000_000 + 86_400;
    client.create_campaign(&id, &String::from_str(&env, "T"), &creator, &1_000, &deadline, &token);

    let result = client.try_extend_campaign_deadline(&id, &(deadline - 1));
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDeadline)));
}

#[test]
fn extend_deadline_beyond_90_days_reverts() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    let id = campaign_id(&env, 11);
    let deadline = 1_000_000 + 86_400;
    client.create_campaign(&id, &String::from_str(&env, "T"), &creator, &1_000, &deadline, &token);

    let too_far = 1_000_000 + (90 * 24 * 60 * 60) + 1;
    let result = client.try_extend_campaign_deadline(&id, &too_far);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDeadline)));
}

#[test]
fn extend_deadline_valid_succeeds() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let creator = Address::generate(&env);
    let id = campaign_id(&env, 12);
    let deadline = 1_000_000 + 86_400;
    client.create_campaign(&id, &String::from_str(&env, "T"), &creator, &1_000, &deadline, &token);

    let new_deadline = deadline + 86_400; // +1 more day, well within 90 days
    client.extend_campaign_deadline(&id, &new_deadline);

    let campaign = client.get_campaign(&id);
    assert_eq!(campaign.deadline, new_deadline);
}
