#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_get_pool_metadata_existing_pool_returns_saved_fields() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let title = String::from_str(&env, "School Supplies");
    let description = String::from_str(&env, "Notebooks and uniforms for the term");
    let pool_id = client.create_pool(
        &Address::generate(&env),
        &title,
        &description,
        &4_000_000_000u128,
        &250_000u64,
    );

    let (stored_title, stored_description) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, title);
    assert_eq!(stored_description, description);
}

#[test]
fn test_get_pool_metadata_nonexistent_pool_returns_empty_strings() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (title, description) = client.get_pool_metadata(&424242u32);
    assert_eq!(title, String::from_str(&env, ""));
    assert_eq!(description, String::from_str(&env, ""));
}

#[test]
fn test_get_pool_metadata_matches_explicit_saved_values() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let title = String::from_str(&env, "Exact Title");
    let description = String::from_str(&env, "Exact description value");
    let pool_id = client.create_pool(
        &Address::generate(&env),
        &title,
        &description,
        &9_000_000u128,
        &180_000u64,
    );

    let (stored_title, stored_description) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title, String::from_str(&env, "Exact Title"));
    assert_eq!(
        stored_description,
        String::from_str(&env, "Exact description value")
    );
    assert_eq!(stored_title, title);
    assert_eq!(stored_description, description);
}

#[test]
fn test_get_pool_metadata_pools_are_independent() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let title_a = String::from_str(&env, "Pool Alpha");
    let description_a = String::from_str(&env, "Alpha description");
    let title_b = String::from_str(&env, "Pool Beta");
    let description_b = String::from_str(&env, "Beta description");

    let id_a = client.create_pool(
        &Address::generate(&env),
        &title_a,
        &description_a,
        &1_000_000u128,
        &120_000u64,
    );
    let id_b = client.create_pool(
        &Address::generate(&env),
        &title_b,
        &description_b,
        &2_000_000u128,
        &130_000u64,
    );

    let (stored_title_a, stored_description_a) = client.get_pool_metadata(&id_a);
    let (stored_title_b, stored_description_b) = client.get_pool_metadata(&id_b);

    assert_eq!(stored_title_a, title_a);
    assert_eq!(stored_description_a, description_a);
    assert_eq!(stored_title_b, title_b);
    assert_eq!(stored_description_b, description_b);
    assert_ne!(stored_title_a, stored_title_b);
    assert_ne!(stored_description_a, stored_description_b);
}
