#![cfg(test)]

//! Issue #1459 — `set_pool_token` overwrites POOL_TOKEN_PREFIX with no guard
//! against existing collected balance or a previously different token.

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env, String, Symbol,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn stored_pool_token(env: &Env, contract_id: &Address, pool_id: u32) -> Option<Address> {
    env.as_contract(contract_id, || {
        let key = (Symbol::new(env, "pool_token"), pool_id);
        env.storage().persistent().get::<_, Address>(&key)
    })
}

fn approved(env: &Env) -> String {
    String::from_str(env, "Approved")
}

#[test]
#[should_panic(expected = "TokenTransferFailed")]
fn test_refund_blocked_after_token_swap() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token_a = create_token(&env, 1_000_000, &donor);
    let token_b = create_token(&env, 1_000_000, &creator);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1459 Refund Pool"),
        &String::from_str(&env, "Token swap then refund"),
        &1_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token_a);
    client.donate_with_token(&pool_id, &donor, &token_a, &500_000i128);
    client.set_pool_token(&pool_id, &token_b);
    client.refund_donation(&pool_id, &donor, &token_a);
}

#[test]
fn test_claim_funds_uses_unrelated_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_a = create_token(&env, 1_000_000, &contract_id);
    let token_b = create_token(&env, 1_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1459 Claim Pool"),
        &String::from_str(&env, "Claim in unrelated token"),
        &1_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token_a);
    client.donate_with_token(&pool_id, &creator, &token_a, &500_000i128);
    client.set_application_status(&pool_id, &student, &approved(&env));

    client.claim_funds(&student, &pool_id, &100_000i128, &token_b);

    let student_balance_b =
        soroban_sdk::token::Client::new(&env, &token_b).balance(&student);
    assert_eq!(student_balance_b, 99_000);

    let stored = stored_pool_token(&env, &contract_id, pool_id);
    assert_eq!(stored, Some(token_a));
}