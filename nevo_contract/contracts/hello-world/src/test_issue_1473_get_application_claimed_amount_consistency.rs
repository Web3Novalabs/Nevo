#![cfg(test)]

use crate::{Contract, ContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let contract = soroban_sdk::token::Client::new(env, &env.register_stellar_asset_contract(
        soroban_sdk::symbol_short!("USD"),
    ));
    contract.mint(&recipient, &amount);
    contract.address.clone()
}

/// Test that get_claimed_amount returns 0 for a student who has never claimed,
/// while get_application returns None - documenting the intentional inconsistency.
#[test]
fn test_issue_1473_claimed_amount_and_application_fallback_inconsistency() {
    let env = Env::new();
    let admin = Address::random(&env);
    let sponsor = Address::random(&env);
    let school = Address::random(&env);
    let student = Address::random(&env);

    let client = ContractClient::new(&env, &env.register_contract(None, Contract));

    // Setup: initialize admin and register school
    client.set_admin(&admin);
    client.register_school(&school, &[0; 32].into());

    // Create a pool
    let pool_id = client.create_pool_for_school(
        &sponsor,
        &"Test Pool".into(),
        &"A test pool".into(),
        &1000u128,
        &school,
        &100u64,
    );

    // Student submits application but has never claimed any funds
    client.submit_application(&pool_id, &student, &"Application data".into());

    // get_claimed_amount returns 0 for a student who has never claimed
    let claimed_amount = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed_amount, 0, "get_claimed_amount should return 0 for unclaimed student");

    // get_application returns the full Application record (not None)
    let application = client.get_application(&pool_id, &student);
    assert!(application.is_some(), "get_application should return Some for applied student");
    
    let app = application.unwrap();
    assert_eq!(app.amount_claimed, 0, "Application.amount_claimed should be 0 initially");
    assert_eq!(app.student, student, "Application student should match");
    assert_eq!(app.pool_id, pool_id, "Application pool_id should match");
}

/// Test that after one partial claim, both get_claimed_amount and get_application
/// agree on the amount_claimed value.
#[test]
fn test_issue_1473_both_getters_agree_after_partial_claim() {
    let env = Env::new();
    let admin = Address::random(&env);
    let sponsor = Address::random(&env);
    let school = Address::random(&env);
    let student = Address::random(&env);

    let client = ContractClient::new(&env, &env.register_contract(None, Contract));

    // Setup: initialize admin and register school
    client.set_admin(&admin);
    client.register_school(&school, &[0; 32].into());

    // Create a pool
    let pool_id = client.create_pool_for_school(
        &sponsor,
        &"Test Pool".into(),
        &"A test pool".into(),
        &1000u128,
        &school,
        &100u64,
    );

    // Student submits application
    client.submit_application(&pool_id, &student, &"Application data".into());

    // School approves the student's application with 500 approved amount
    client.approve_application(&pool_id, &school, &student, &true);
    client.set_approved_amount(&pool_id, &student, &500i128);

    // Student claims some funds (100 out of 500 approved)
    let token = create_token(&env, 600i128, &sponsor);
    client.set_pool_token(&pool_id, &token);
    client.donate(&pool_id, &sponsor, &500u128);
    
    client.claim_funds(&pool_id, &student, &100i128, &token);

    // After claim, both getters should agree on amount_claimed
    let claimed_amount = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed_amount, 100, "get_claimed_amount should return 100 after claim");

    let application = client.get_application(&pool_id, &student);
    assert!(application.is_some(), "get_application should return Some after claim");
    
    let app = application.unwrap();
    assert_eq!(app.amount_claimed, 100, "Application.amount_claimed should be 100 after claim");
    assert_eq!(
        claimed_amount, app.amount_claimed,
        "Both getters should return the same amount_claimed value"
    );
}
