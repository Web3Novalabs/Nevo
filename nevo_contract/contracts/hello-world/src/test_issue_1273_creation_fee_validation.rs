#![cfg(test)]
//! Tests for creation fee configuration validation — issue #1273.
//!
//! Covers:
//!   1. Admin can set positive fee.
//!   2. Admin can set zero fee.
//!   3. Negative fee fails with InvalidFee.
//!   4. Non-admin authorization fails.
//!   5. Fee update emits event.
//!   6. Get function returns updated fee.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env,
};

/// (1) Admin can set positive fee.
#[test]
fn test_admin_can_set_positive_creation_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let fee: i128 = 500_000_000;

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    assert_eq!(client.get_creation_fee(), fee);
}

/// (2) Admin can set zero fee.
#[test]
fn test_admin_can_set_zero_creation_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.set_admin(&admin);
    // Initially set positive fee
    client.set_creation_fee(&admin, &250_000_000i128);
    assert_eq!(client.get_creation_fee(), 250_000_000i128);

    // Update fee to zero
    client.set_creation_fee(&admin, &0i128);
    assert_eq!(client.get_creation_fee(), 0i128);
}

/// (3) Negative fee fails with InvalidFee.
#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_negative_fee_fails_with_invalid_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &-100i128);
}

#[test]
fn test_negative_fee_try_fails_with_invalid_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    client.set_admin(&admin);
    let res = client.try_set_creation_fee(&admin, &-500i128);

    assert_eq!(
        res,
        Err(Ok(ContractError::InvalidFee)),
        "Negative fee must return ContractError::InvalidFee"
    );
}

/// (4) Non-admin authorization fails.
#[test]
fn test_non_admin_authorization_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);

    client.set_admin(&admin);

    let res = client.try_set_creation_fee(&non_admin, &100_000_000i128);
    assert_eq!(
        res,
        Err(Ok(ContractError::UnauthorizedAdmin)),
        "Non-admin call must return ContractError::UnauthorizedAdmin"
    );
}

/// (5) Fee update emits event.
#[test]
fn test_fee_update_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let fee: i128 = 300_000_000;

    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    let events = env.events().all();
    let contract_events = events
        .iter()
        .filter(|(contract, _topics, _data)| *contract == contract_id)
        .count();

    assert!(
        contract_events > 0,
        "Fee update must emit an event"
    );
}

/// (6) Get function returns updated fee.
#[test]
fn test_get_function_returns_updated_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // Unset returns default 0
    assert_eq!(client.get_creation_fee(), 0i128);

    client.set_admin(&admin);

    client.set_creation_fee(&admin, &123_456_789i128);
    assert_eq!(client.get_creation_fee(), 123_456_789i128);

    client.set_creation_fee(&admin, &987_654_321i128);
    assert_eq!(client.get_creation_fee(), 987_654_321i128);
}
