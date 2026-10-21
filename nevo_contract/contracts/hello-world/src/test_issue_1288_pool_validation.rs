#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

/// Pool creation is `create_pool`. It is the function that validates the
/// name, target, and deadline and returns a generated pool id.
#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_save_pool_empty_name_fails_with_invalid_pool_name() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, ""),
        &String::from_str(&env, "A real description"),
        &1_000_000u128,
        &200_000u64,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_save_pool_zero_target_fails_with_invalid_pool_target() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Named Pool"),
        &String::from_str(&env, "A real description"),
        &0u128,
        &200_000u64,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #18)")]
fn test_save_pool_past_deadline_fails_with_invalid_pool_deadline() {
    let env = Env::default();
    env.ledger().set_timestamp(50_000);
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Named Pool"),
        &String::from_str(&env, "A real description"),
        &1_000_000u128,
        &49_999u64,
    );
}

#[test]
fn test_save_pool_valid_parameters_succeed_and_are_retrievable() {
    let env = Env::default();
    env.ledger().set_timestamp(10_000);
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let deadline = 80_000u64;
    let goal = 7_500_000u128;
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Valid Pool"),
        &String::from_str(&env, "Valid description"),
        &goal,
        &deadline,
    );

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, goal);
    assert_eq!(pool.5, deadline);
}

#[test]
fn test_save_pool_generated_ids_are_nonzero_and_unique() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    let id1 = client.create_pool(
        &creator,
        &String::from_str(&env, "First Pool"),
        &String::from_str(&env, "First"),
        &1_000_000u128,
        &200_000u64,
    );
    let id2 = client.create_pool(
        &creator,
        &String::from_str(&env, "Second Pool"),
        &String::from_str(&env, "Second"),
        &2_000_000u128,
        &200_000u64,
    );

    assert_ne!(id1, 0);
    assert_ne!(id2, 0);
    assert_ne!(id1, id2);
    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(client.get_pool(&id1).0, id1);
    assert_eq!(client.get_pool(&id2).0, id2);
}
