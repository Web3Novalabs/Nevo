#![cfg(test)]
//! Tests for crowdfunding token configuration — issue #1272.
//!
//! Covers:
//!   1. Admin can update token successfully.
//!   2. Non-admin gets authorization error.
//!   3. Setting invalid token address handled properly.
//!   4. Token update emits correct event.
//!   5. Get function returns updated token.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env,
};

/// (1) Admin can update token successfully.
#[test]
fn test_admin_can_update_crowdfunding_token_successfully() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    client.set_admin(&admin);
    client.set_crowdfunding_token(&admin, &token);

    assert_eq!(client.get_crowdfunding_token(), token);
}

/// (2) Non-admin gets authorization error.
#[test]
fn test_non_admin_update_token_authorization_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let token = Address::generate(&env);

    client.set_admin(&admin);

    let res = client.try_set_crowdfunding_token(&non_admin, &token);
    assert_eq!(
        res,
        Err(Ok(ContractError::UnauthorizedAdmin)),
        "Non-admin must receive UnauthorizedAdmin error"
    );
}

/// (3) Setting invalid token address handled properly.
#[test]
#[should_panic(expected = "InvalidTokenAddress")]
fn test_setting_invalid_token_address_handled_properly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.set_admin(&admin);

    // Using contract's own address as token address is invalid
    client.set_crowdfunding_token(&admin, &contract_id);
}

/// (4) Token update emits correct event.
#[test]
fn test_token_update_emits_correct_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    client.set_admin(&admin);
    client.set_crowdfunding_token(&admin, &token);

    let events = env.events().all();
    let token_events_count = events
        .iter()
        .filter(|(contract, _topics, _data)| *contract == contract_id)
        .count();

    assert!(
        token_events_count > 0,
        "Token update must emit an event"
    );
}

/// (5) Get function returns updated token.
#[test]
fn test_get_crowdfunding_token_returns_updated_token() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_1 = Address::generate(&env);
    let token_2 = Address::generate(&env);

    client.set_admin(&admin);

    client.set_crowdfunding_token(&admin, &token_1);
    assert_eq!(client.get_crowdfunding_token(), token_1);

    client.set_crowdfunding_token(&admin, &token_2);
    assert_eq!(client.get_crowdfunding_token(), token_2);
}
