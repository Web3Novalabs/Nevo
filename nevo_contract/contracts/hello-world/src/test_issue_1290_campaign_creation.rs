#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Symbol,
};

/// Campaign creation is `create_pool`: there is no separate `create_campaign`.
#[test]
fn test_create_campaign_maximum_title_length_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, &"T".repeat(MAX_TITLE_LENGTH as usize));
    let description = String::from_str(&env, "Boundary title");
    let deadline = 200_000u64;

    let pool_id = client.create_pool(
        &creator,
        &title,
        &description,
        &1_000_000_000u128,
        &deadline,
    );

    let (stored_title, stored_description) = client.get_pool_metadata(&pool_id);
    assert_eq!(stored_title.len(), MAX_TITLE_LENGTH);
    assert_eq!(stored_title, title);
    assert_eq!(stored_description, description);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.5, deadline);
}

#[test]
fn test_create_campaign_maximum_goal_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = u128::MAX;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Maximum Goal"),
        &String::from_str(&env, "Stores the largest u128 goal"),
        &goal,
        &200_000u64,
    );

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.2, goal);
    assert_eq!(pool.3, 0u128);
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_create_campaign_deadline_equal_to_ledger_timestamp_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let now = 150_000u64;
    env.ledger().set_timestamp(now);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Equal Deadline"),
        &String::from_str(&env, "Must be rejected"),
        &1_000_000_000u128,
        &now,
    );
}

#[test]
fn test_create_campaign_future_deadline_succeeds() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let now = 150_000u64;
    env.ledger().set_timestamp(now);
    let deadline = now + 1;

    let pool_id = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Future Deadline"),
        &String::from_str(&env, "One second ahead"),
        &1_000_000_000u128,
        &deadline,
    );

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.5, deadline);
    assert!(pool.5 > now);
}

#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_create_campaign_duplicate_id_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Original Campaign"),
        &String::from_str(&env, "First write"),
        &1_000_000_000u128,
        &200_000u64,
    );
    assert_eq!(pool_id, 1);

    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, POOL_COUNT), &0u32);
    });

    client.create_pool(
        &creator,
        &String::from_str(&env, "Duplicate Campaign"),
        &String::from_str(&env, "Same generated id"),
        &2_000_000_000u128,
        &200_000u64,
    );
}
