#![cfg(test)]
//! set_crowdfunding_token emits a single canonical event — issue #1462.
//!
//! The call used to publish both `tok_set` and an inline
//! `crowdfunding_token_set` topic for the same action. Only the canonical
//! `CROWDFUNDING_TOKEN_SET` (`tok_set`) event remains.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, IntoVal, Val,
};

#[test]
fn test_set_crowdfunding_token_emits_exactly_one_canonical_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_admin(&admin);

    client.set_crowdfunding_token(&admin, &token);

    // Exactly one event, on the canonical topic, carrying the new token.
    // The retired `crowdfunding_token_set` duplicate would make this compare fail.
    let expected: Vec<(Address, Vec<Val>, Val)> = soroban_sdk::vec![
        &env,
        (
            contract_id.clone(),
            soroban_sdk::vec![&env, CROWDFUNDING_TOKEN_SET.into_val(&env)],
            token.into_val(&env),
        ),
    ];
    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        expected,
        "set_crowdfunding_token must publish exactly one tok_set event"
    );
}

#[test]
fn test_each_set_crowdfunding_token_call_emits_one_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let first = Address::generate(&env);
    client.set_crowdfunding_token(&admin, &first);
    assert_eq!(env.events().all().filter_by_contract(&contract_id).len(), 1);

    let second = Address::generate(&env);
    client.set_crowdfunding_token(&admin, &second);
    let events = env.events().all().filter_by_contract(&contract_id);
    assert_eq!(events.len(), 1, "a second update must also emit a single event");
    assert_eq!(client.get_crowdfunding_token(), second);
}
