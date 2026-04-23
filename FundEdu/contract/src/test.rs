#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::contract::{FundEduContract, FundEduContractClient};

fn setup() -> (Env, FundEduContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(FundEduContract, ());
    let client = FundEduContractClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_create_pool_returns_incremental_ids() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let token = Address::generate(&env);

    let id0 = client.create_pool(
        &sponsor,
        &String::from_str(&env, "STEM 2026"),
        &50_000_000,
        &token,
    );
    let id1 = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Arts 2026"),
        &20_000_000,
        &token,
    );

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
}

#[test]
fn test_get_pool_returns_correct_data() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let token = Address::generate(&env);

    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "STEM 2026"),
        &50_000_000,
        &token,
    );

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.name, String::from_str(&env, "STEM 2026"));
    assert_eq!(pool.target_amount, 50_000_000);
    assert_eq!(pool.sponsor, sponsor);
    assert!(pool.is_active);
}

#[test]
fn test_get_pool_returns_none_for_missing_id() {
    let (_env, client) = setup();
    assert!(client.get_pool(&99).is_none());
}
