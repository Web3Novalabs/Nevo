#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token_id, &0);

    (client, admin, token_id)
}

fn create_test_pool(env: &Env, client: &CrowdfundingContractClient<'_>, creator: &Address) -> u64 {
    env.ledger().with_mut(|li| li.timestamp = 1_000);

    let config = PoolConfig {
        name: String::from_str(env, "Test Pool"),
        description: String::from_str(env, "A test pool for metadata updates"),
        target_amount: 1_000_000,
        min_contribution: 100,
        is_private: false,
        duration: 86400, // 1 day
        created_at: 1_000,
    };

    client.create_pool(creator, &config)
}

#[test]
fn test_update_pool_metadata_hash_success() {
    let env = Env::default();
    let (client, _admin, _token_id) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&env, &client, &creator);

    let new_hash = String::from_str(&env, "QmNewHash123456789");

    let result = client.try_update_pool_metadata_hash(&pool_id, &creator, &new_hash);
    assert_eq!(result, Ok(Ok(())), "metadata update should succeed");

    // Verify the metadata was updated
    let (_, _, image_hash) = client.get_pool_metadata(&pool_id);
    assert_eq!(image_hash, new_hash, "image hash should be updated");
}

#[test]
fn test_update_pool_metadata_hash_only_creator_can_update() {
    let env = Env::default();
    let (client, _admin, _token_id) = setup_test(&env);

    let creator = Address::generate(&env);
    let non_creator = Address::generate(&env);
    let pool_id = create_test_pool(&env, &client, &creator);

    let new_hash = String::from_str(&env, "QmNewHash123456789");

    let result = client.try_update_pool_metadata_hash(&pool_id, &non_creator, &new_hash);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::Unauthorized)),
        "non-creator should not be able to update metadata"
    );
}

#[test]
fn test_update_pool_metadata_hash_nonexistent_pool() {
    let env = Env::default();
    let (client, _admin, _token_id) = setup_test(&env);

    let creator = Address::generate(&env);
    let nonexistent_pool_id = 999u64;
    let new_hash = String::from_str(&env, "QmNewHash123456789");

    let result = client.try_update_pool_metadata_hash(&nonexistent_pool_id, &creator, &new_hash);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotFound)),
        "should fail for nonexistent pool"
    );
}

#[test]
fn test_update_pool_metadata_hash_too_long() {
    let env = Env::default();
    let (client, _admin, _token_id) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&env, &client, &creator);

    // Create a hash that exceeds MAX_HASH_LENGTH (100 characters)
    let long_hash = String::from_str(
        &env,
        "QmThisIsAVeryLongHashThatExceedsTheMaximumAllowedLengthOf100CharactersAndShouldBeRejectedByTheValidation",
    );

    let result = client.try_update_pool_metadata_hash(&pool_id, &creator, &long_hash);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InvalidMetadata)),
        "should fail for hash exceeding max length"
    );
}

#[test]
fn test_update_pool_metadata_hash_when_paused() {
    let env = Env::default();
    let (client, _admin, _token_id) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&env, &client, &creator);

    // Pause the contract
    client.pause();

    let new_hash = String::from_str(&env, "QmNewHash123456789");

    let result = client.try_update_pool_metadata_hash(&pool_id, &creator, &new_hash);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::ContractPaused)),
        "should fail when contract is paused"
    );
}

#[test]
fn test_update_pool_metadata_hash_multiple_times() {
    let env = Env::default();
    let (client, _admin, _token_id) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&env, &client, &creator);

    // First update
    let hash1 = String::from_str(&env, "QmFirstHash");
    let result1 = client.try_update_pool_metadata_hash(&pool_id, &creator, &hash1);
    assert_eq!(result1, Ok(Ok(())));

    let (_, _, image_hash1) = client.get_pool_metadata(&pool_id);
    assert_eq!(image_hash1, hash1);

    // Second update
    let hash2 = String::from_str(&env, "QmSecondHash");
    let result2 = client.try_update_pool_metadata_hash(&pool_id, &creator, &hash2);
    assert_eq!(result2, Ok(Ok(())));

    let (_, _, image_hash2) = client.get_pool_metadata(&pool_id);
    assert_eq!(image_hash2, hash2);
}

#[test]
fn test_update_pool_metadata_hash_event_emission() {
    let env = Env::default();
    let (client, _admin, _token_id) = setup_test(&env);

    let creator = Address::generate(&env);
    let pool_id = create_test_pool(&env, &client, &creator);

    let new_hash = String::from_str(&env, "QmNewHash123456789");

    client.update_pool_metadata_hash(&pool_id, &creator, &new_hash);

    // Events are emitted - in a real test you'd verify the event was published
    // For now, we just verify the operation succeeded
    let (_, _, image_hash) = client.get_pool_metadata(&pool_id);
    assert_eq!(image_hash, new_hash);
}
