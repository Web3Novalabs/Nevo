#![cfg(test)]

use crate::{Contract, ContractClient};
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, BytesN, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin);
    StellarAssetClient::new(env, &token.address()).mint(recipient, &amount);
    token.address()
}

/// Register a school and create a pool linked to it.
fn setup_school_pool(env: &Env, client: &ContractClient) -> (u32, Address) {
    let admin = Address::generate(env);
    let sponsor = Address::generate(env);
    let school = Address::generate(env);

    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(env, &[0u8; 32]));

    let pool_id = client.create_pool_for_school(
        &sponsor,
        &String::from_str(env, "Test Pool"),
        &String::from_str(env, "A test pool"),
        &1000u128,
        &school,
        &100u64,
    );
    (pool_id, school)
}

/// Test that get_claimed_amount returns 0 for a student who has applied but
/// never claimed, and that get_application returns the full record.
#[test]
fn test_issue_1473_claimed_amount_and_application_fallback_inconsistency() {
    let env = Env::default();
    env.mock_all_auths();
    let client = ContractClient::new(&env, &env.register(Contract, ()));

    let (pool_id, _school) = setup_school_pool(&env, &client);
    let student = Address::generate(&env);

    // Student applies but has never claimed any funds
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application data"));

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
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, school) = setup_school_pool(&env, &client);
    let student = Address::generate(&env);
    let donor = Address::generate(&env);

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "Application data"));
    client.approve_application(&pool_id, &school, &student, &true);

    // 500 collected; the contract holds the tokens it pays claims from.
    client.donate(&pool_id, &donor, &500u128);
    let token = create_token(&env, 500i128, &contract_id);

    // Student claims 100 of the 500 available
    client.claim_funds(&student, &pool_id, &100i128, &token);

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
