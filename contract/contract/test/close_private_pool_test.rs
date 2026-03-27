#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, PoolState},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
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

fn create_private_pool(
    client: &CrowdfundingContractClient,
    env: &Env,
    creator: &Address,
    token_address: &Address,
) -> u64 {
    let config = PoolConfig {
        name: String::from_str(env, "Private Pool"),
        description: String::from_str(env, "A private pool for testing"),
        target_amount: 1_000_000,
        min_contribution: 100,
        is_private: true,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
    };
    client.create_pool(creator, &config)
}

fn create_public_pool(
    client: &CrowdfundingContractClient,
    env: &Env,
    creator: &Address,
    token_address: &Address,
) -> u64 {
    let config = PoolConfig {
        name: String::from_str(env, "Public Pool"),
        description: String::from_str(env, "A public pool for testing"),
        target_amount: 1_000_000,
        min_contribution: 100,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
    };
    client.create_pool(creator, &config)
}

#[test]
fn test_owner_can_close_private_pool() {
    let env = Env::default();
    let (client, _admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token_address);

    client.close_pool(&pool_id, &owner);

    let is_closed = client.is_closed(&pool_id);
    assert!(is_closed);
}

#[test]
fn test_contribute_fails_on_closed_private_pool() {
    let env = Env::default();
    let (client, _admin, token) = setup_test(&env);

    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token);

    client.close_pool(&pool_id, &owner);

    let result = client.try_contribute(&pool_id, &contributor, &token, &1000, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));
}

#[test]
fn test_owner_cannot_close_public_pool_when_active() {
    let env = Env::default();
    let (client, _admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_public_pool(&client, &env, &owner, &token_address);

    let result = client.try_close_pool(&pool_id, &owner);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotDisbursedOrRefunded))
    );
}

#[test]
fn test_non_owner_cannot_close_private_pool() {
    let env = Env::default();
    let (client, _admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token_address);

    let result = client.try_close_pool(&pool_id, &non_owner);
    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_admin_can_close_private_pool() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token_address);

    let result = client.try_close_pool(&pool_id, &admin);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotDisbursedOrRefunded))
    );
}

#[test]
fn test_owner_can_close_paused_private_pool() {
    let env = Env::default();
    let (client, _admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token_address);

    client.update_pool_state(&pool_id, &PoolState::Paused);
    client.close_pool(&pool_id, &owner);

    let is_closed = client.is_closed(&pool_id);
    assert!(is_closed);
}

#[test]
fn test_owner_cannot_close_completed_private_pool() {
    let env = Env::default();
    let (client, _admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token_address);

    client.update_pool_state(&pool_id, &PoolState::Completed);

    let result = client.try_close_pool(&pool_id, &owner);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));
}

#[test]
fn test_close_private_pool_before_deadline() {
    let env = Env::default();
    let (client, _admin, token) = setup_test(&env);

    let owner = Address::generate(&env);
    let contributor = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token);

    use soroban_sdk::token;
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &10000);

    client.contribute(&pool_id, &contributor, &token, &1000, &false);
    client.close_pool(&pool_id, &owner);

    assert!(client.is_closed(&pool_id));

    let result = client.try_contribute(&pool_id, &contributor, &token, &1000, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));
}

#[test]
fn test_close_already_closed_private_pool() {
    let env = Env::default();
    let (client, _admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token_address);

    client.close_pool(&pool_id, &owner);

    let result = client.try_close_pool(&pool_id, &owner);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));
}

#[test]
fn test_multiple_private_pools_independent_closure() {
    let env = Env::default();
    let (client, _admin, token) = setup_test(&env);

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let contributor = Address::generate(&env);

    let pool_id_1 = create_private_pool(&client, &env, &owner1, &token);
    let pool_id_2 = create_private_pool(&client, &env, &owner2, &token);

    use soroban_sdk::token;
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &10000);

    client.close_pool(&pool_id_1, &owner1);

    assert!(client.is_closed(&pool_id_1));
    assert!(!client.is_closed(&pool_id_2));

    let result = client.try_contribute(&pool_id_1, &contributor, &token, &1000, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyClosed)));

    client.contribute(&pool_id_2, &contributor, &token, &1000, &false);
}

#[test]
fn test_admin_can_close_after_disbursement() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token_address);

    client.update_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id, &admin);

    assert!(client.is_closed(&pool_id));
}

#[test]
fn test_owner_can_close_after_cancellation() {
    let env = Env::default();
    let (client, _admin, token_address) = setup_test(&env);

    let owner = Address::generate(&env);
    let pool_id = create_private_pool(&client, &env, &owner, &token_address);

    client.update_pool_state(&pool_id, &PoolState::Cancelled);
    client.close_pool(&pool_id, &owner);

    assert!(client.is_closed(&pool_id));
}
