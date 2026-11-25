#![cfg(test)]

use crate::{test_issues::approve_application_for_test, Contract, ContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin);
    let sac = soroban_sdk::token::StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// Test that get_claimed_amount returns 0 for a student who has never claimed,
/// while get_application returns None - documenting the intentional inconsistency.
#[test]
fn test_issue_1473_claimed_amount_and_application_fallback_inconsistency() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let school = Address::generate(&env);
    let student = Address::generate(&env);

    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(&env, &[0u8; 32]));

    let pool_id = client.create_pool_for_school(
        &sponsor,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "A test pool"),
        &(1000i128),
        &school,
        &100u64,
    );

    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Application data"),
    );

    let claimed_amount = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed_amount, 0, "get_claimed_amount should return 0 for unclaimed student");

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
    let admin = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let school = Address::generate(&env);
    let student = Address::generate(&env);

    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(&env, &[0u8; 32]));

    let pool_id = client.create_pool_for_school(
        &sponsor,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "A test pool"),
        &(1000i128),
        &school,
        &100u64,
    );

    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Application data"),
    );

    approve_application_for_test(&env, &contract_id, &client, pool_id, &school, &student, true);

    let token = create_token(&env, 600i128, &contract_id);
    client.set_pool_token(&pool_id, &token);
    client.donate(&pool_id, &sponsor, &500i128);

    client.claim_funds(&student, &pool_id, &100i128, &token);

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