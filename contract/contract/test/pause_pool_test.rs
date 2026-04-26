#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env, String};

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

fn create_pool(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token_address: &Address,
) -> (u64, Address) {
    let sponsor = Address::generate(env);
    let token_admin_client = token::StellarAssetClient::new(env, token_address);
    token_admin_client.mint(&sponsor, &10_000i128);

    let config = PoolConfig {
        name: String::from_str(env, "Test Pool"),
        description: String::from_str(env, "A pool for testing"),
        target_amount: 10_000,
        min_contribution: 0,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: sponsor.clone(),
    };

    let pool_id = client.create_pool(&sponsor, &config);
    (pool_id, sponsor)
}

#[test]
fn test_pause_pool_sets_paused_state() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_pool(&env, &client, &token_address);

    client.pause_pool(&pool_id, &sponsor);

    let state: PoolState = env
        .storage()
        .instance()
        .get(&crate::base::types::StorageKey::PoolState(pool_id))
        .unwrap();
    assert_eq!(state, PoolState::Paused);
}

#[test]
fn test_pause_pool_unauthorized_fails() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, _sponsor) = create_pool(&env, &client, &token_address);

    let stranger = Address::generate(&env);
    let result = client.try_pause_pool(&pool_id, &stranger);
    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_pause_pool_already_paused_fails() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_pool(&env, &client, &token_address);

    client.pause_pool(&pool_id, &sponsor);
    let result = client.try_pause_pool(&pool_id, &sponsor);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractAlreadyPaused)));
}

#[test]
fn test_pause_pool_not_found_fails() {
    let env = Env::default();
    let (client, _admin, _token_address) = setup(&env);

    let caller = Address::generate(&env);
    let result = client.try_pause_pool(&999u64, &caller);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_claim_pool_funds_blocked_when_paused() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_pool(&env, &client, &token_address);

    client.pause_pool(&pool_id, &sponsor);

    let student = Address::generate(&env);
    let result = client.try_claim_pool_funds(&pool_id, &student);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));
}

#[test]
fn test_pause_then_unpause_allows_contributions() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, sponsor) = create_pool(&env, &client, &token_address);

    client.pause_pool(&pool_id, &sponsor);
    client.unpause_pool(&pool_id, &sponsor);

    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&contributor, &1_000i128);

    client.contribute(&pool_id, &contributor, &token_address, &500i128, &false);
}
