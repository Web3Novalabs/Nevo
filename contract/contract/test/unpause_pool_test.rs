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
    let creator = Address::generate(env);
    let token_admin_client = token::StellarAssetClient::new(env, token_address);
    token_admin_client.mint(&creator, &10_000i128);

    let config = PoolConfig {
        name: String::from_str(env, "Test Pool"),
        description: String::from_str(env, "A pool for testing"),
        target_amount: 10_000,
        min_contribution: 0,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: creator.clone(),
    };

    let pool_id = client.create_pool(&creator, &config);
    (pool_id, creator)
}

#[test]
fn test_unpause_pool_restores_active_state() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, creator) = create_pool(&env, &client, &token_address);

    // Pause the pool via update_pool_state
    client.update_pool_state(&pool_id, &creator, &PoolState::Paused);

    // Confirm it is paused
    let state: PoolState = env
        .storage()
        .instance()
        .get(&crate::base::types::StorageKey::PoolState(pool_id))
        .unwrap();
    assert_eq!(state, PoolState::Paused);

    // Unpause
    client.unpause_pool(&pool_id, &creator);

    // State must be Active again
    let state_after: PoolState = env
        .storage()
        .instance()
        .get(&crate::base::types::StorageKey::PoolState(pool_id))
        .unwrap();
    assert_eq!(state_after, PoolState::Active);
}

#[test]
fn test_unpause_pool_allows_contributions() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, creator) = create_pool(&env, &client, &token_address);

    // Pause then unpause
    client.update_pool_state(&pool_id, &creator, &PoolState::Paused);
    client.unpause_pool(&pool_id, &creator);

    // Contribution should succeed after unpause
    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&contributor, &1_000i128);

    client.contribute(&pool_id, &contributor, &token_address, &500i128, &false);
}

#[test]
fn test_unpause_pool_unauthorized_fails() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, creator) = create_pool(&env, &client, &token_address);

    client.update_pool_state(&pool_id, &creator, &PoolState::Paused);

    let stranger = Address::generate(&env);
    let result = client.try_unpause_pool(&pool_id, &stranger);
    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_unpause_pool_not_paused_fails() {
    let env = Env::default();
    let (client, _admin, token_address) = setup(&env);
    let (pool_id, creator) = create_pool(&env, &client, &token_address);

    // Pool is Active — unpausing should fail
    let result = client.try_unpause_pool(&pool_id, &creator);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractAlreadyUnpaused)));
}

#[test]
fn test_unpause_pool_not_found_fails() {
    let env = Env::default();
    let (client, _admin, _token_address) = setup(&env);

    let caller = Address::generate(&env);
    let result = client.try_unpause_pool(&999u64, &caller);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}
