#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn new_pool(env: &Env, client: &ContractClient, creator: &Address) -> u32 {
    client.create_pool(
        creator,
        &String::from_str(env, "Configuration bounds"),
        &String::from_str(env, "Initial description"),
        &1_000_000u128,
        &100_000u64,
    )
}

#[test]
fn test_config_bounds_description_at_maximum_is_saved() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = new_pool(&env, &client, &creator);
    let description = String::from_str(&env, &"d".repeat(MAX_DESCRIPTION_LENGTH));

    client.save_pool(
        &pool_id,
        &description,
        &String::from_str(&env, "https://example.com"),
        &String::from_str(&env, "a".repeat(MAX_IMAGE_HASH_LENGTH)),
    );

    assert_eq!(client.get_saved_pool_metadata(&pool_id).0, description);
}

#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_config_bounds_description_over_maximum_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = new_pool(&env, &client, &creator);

    client.save_pool(
        &pool_id,
        &String::from_str(&env, &"d".repeat(MAX_DESCRIPTION_LENGTH + 1)),
        &String::from_str(&env, ""),
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic(expected = "URL exceeds maximum length")]
fn test_config_bounds_url_over_maximum_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = new_pool(&env, &client, &creator);

    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Description"),
        &String::from_str(&env, &"u".repeat(MAX_URL_LENGTH + 1)),
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic(expected = "Image hash exceeds maximum length")]
fn test_config_bounds_image_hash_over_maximum_is_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = new_pool(&env, &client, &creator);

    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Description"),
        &String::from_str(&env, ""),
        &String::from_str(&env, &"h".repeat(MAX_IMAGE_HASH_LENGTH + 1)),
    );
}
