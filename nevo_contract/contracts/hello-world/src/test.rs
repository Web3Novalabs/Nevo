#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

#[test]
fn test_create_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Emergency Relief Fund");
    let description = String::from_str(&env, "Helping those in need");
    let goal: u128 = 1_000_000_000;

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
    let goal: u128 = 10_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    let donation_amount: u128 = 100_000_000;
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

    client.donate(&pool_id, &donor1, &100_000_000);
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

// ============= CLAIM_FUNDS TESTS =============

#[test]
#[should_panic(expected = "Application status not found")]
fn test_claim_funds_no_status() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Donate to the pool
    client.donate(&pool_id, &creator, &500_000_000);

    // Try to claim without setting status - should panic
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token_address);
}

#[test]
#[should_panic(expected = "Application is not approved")]
fn test_claim_funds_rejected_application() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Donate to the pool
    client.donate(&pool_id, &creator, &500_000_000);

    // Set status to "Rejected"
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Rejected"));

    // Try to claim with rejected status - should panic
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token_address);
}

#[test]
#[should_panic(expected = "Overdraw attempt")]
fn test_claim_funds_overdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Donate only 100_000_000 to the pool
    client.donate(&pool_id, &creator, &100_000_000);

    // Set status to "Approved"
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Try to claim more than available - should panic
    client.claim_funds(&student, &pool_id, &500_000_000i128, &token_address);
}

#[test]
#[should_panic(expected = "Claim amount must be positive")]
fn test_claim_funds_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Donate to the pool
    client.donate(&pool_id, &creator, &500_000_000);

    // Set status to "Approved"
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Try to claim negative amount - should panic
    client.claim_funds(&student, &pool_id, &-100_000_000i128, &token_address);
}

#[test]
fn test_claim_funds_get_claimed_amount() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Initially, claimed amount should be 0
    let initial_claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(initial_claimed, 0);

    // Donate to the pool
    client.donate(&pool_id, &creator, &500_000_000);

    // Set status to "Approved"
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));
}

#[test]
fn test_get_application_status() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Initially, status should be empty
    let initial_status = client.get_application_status(&pool_id, &student);
    assert_eq!(initial_status, String::from_str(&env, ""));

    // Set status to "Approved"
    let approved_status = String::from_str(&env, "Approved");
    client.set_application_status(&pool_id, &student, &approved_status);

    // Check that status was set correctly
    let status_after_set = client.get_application_status(&pool_id, &student);
    assert_eq!(status_after_set, approved_status);
}
