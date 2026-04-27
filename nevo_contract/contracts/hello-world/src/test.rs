#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

#[test]
fn test_create_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Emergency Relief Fund");
    let description = String::from_str(&env, "Helping those in need");
    let goal: u128 = 1_000_000_000; // 100 XLM in stroops

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    assert_eq!(pool_id, 1);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, 1); // id
    assert_eq!(pool.1, creator); // creator
    assert_eq!(pool.2, goal); // goal
    assert_eq!(pool.3, 0); // collected
    assert_eq!(pool.4, false); // is_closed
}

#[test]
fn test_donate() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Educational Scholarship");
    let description = String::from_str(&env, "Support for students");
    let goal: u128 = 10_000_000_000; // 1000 XLM

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    // Donate to the pool
    let donation_amount: u128 = 100_000_000; // 10 XLM
    client.donate(&pool_id, &donor, &donation_amount);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, donation_amount); // collected amount
}

#[test]
fn test_multiple_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let title = String::from_str(&env, "Community Project");
    let description = String::from_str(&env, "Building together");
    let goal: u128 = 5_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    // First donation
    client.donate(&pool_id, &donor1, &100_000_000);

    // Second donation from different donor
    client.donate(&pool_id, &donor2, &200_000_000);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 300_000_000); // collected amount
}

#[test]
fn test_close_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Closed Pool");
    let description = String::from_str(&env, "Test pool");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    // Close the pool
    client.close_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true); // is_closed
}

#[test]
#[should_panic(expected = "Pool is closed")]
fn test_donate_to_closed_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Test Pool");
    let description = String::from_str(&env, "Test");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    client.close_pool(&pool_id);

    // This should panic
    client.donate(&pool_id, &donor, &100_000_000);
}

#[test]
fn test_multiple_pools() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let pool_id_1 = client.create_pool(
        &creator1,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "First pool"),
        &1_000_000_000,
    );

    let pool_id_2 = client.create_pool(
        &creator2,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Second pool"),
        &2_000_000_000,
    );

    assert_eq!(pool_id_1, 1);
    assert_eq!(pool_id_2, 2);
    assert_eq!(client.get_pool_count(), 2);
}

#[test]
fn test_apply_for_scholarship_success() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let title = String::from_str(&env, "Scholarship Pool");
    let description = String::from_str(&env, "For students");
    let goal: u128 = 10_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    let application_data = String::from_str(&env, "My application details");
    let result = client.apply_for_scholarship(&pool_id, &student, &application_data);

    assert_eq!(result.0, 1); // application id
    assert_eq!(result.1, student);
    assert_eq!(result.2, application_data);
}

#[test]
#[should_panic(expected = "Duplicate application")]
fn test_apply_for_scholarship_duplicate() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let title = String::from_str(&env, "Scholarship Pool");
    let description = String::from_str(&env, "For students");
    let goal: u128 = 10_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    let application_data = String::from_str(&env, "My application details");
    client.apply_for_scholarship(&pool_id, &student, &application_data);

    // This should panic
    client.apply_for_scholarship(&pool_id, &student, &application_data);
}

#[test]
#[should_panic(expected = "Pool not found")]
fn test_apply_for_scholarship_invalid_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let student = Address::generate(&env);
    let application_data = String::from_str(&env, "My application details");

    // Invalid pool id
    client.apply_for_scholarship(&999, &student, &application_data);
}
