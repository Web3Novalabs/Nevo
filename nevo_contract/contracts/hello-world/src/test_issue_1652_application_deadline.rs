//! Tests for issue #1652: `apply_to_pool` must reject applications at or
//! after `Pool.application_deadline`, matching the guard `donate` already has.
//!
//! Boundary coverage: `deadline - 1` (accepted), `deadline` (rejected),
//! `deadline + 1` (rejected).

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String,
};

const APPLICATION_DEADLINE: u64 = 10_000;

/// Register the contract, wire it up with a school and a pool whose
/// `application_deadline` is `APPLICATION_DEADLINE`, and return the pieces
/// the tests need.
fn setup_pool(env: &Env) -> (Address, Address, Address, u32) {
    env.mock_all_auths();
    // Timestamp before the pool is created, so `create_pool`'s "deadline in
    // the future" validation passes.
    env.ledger().set_timestamp(1_000);

    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let school = Address::generate(env);
    let student = Address::generate(env);

    // `initialize` sets the admin; school registration links a pool to the school.
    client.initialize(&admin);
    client.register_school(&school, &String::from_str(env, "Test School"));

    let pool_id = client.create_pool(
        &school,
        &String::from_str(env, "Test Pool"),
        &String::from_str(env, "Description"),
        &1_000_000_i128,
        &APPLICATION_DEADLINE,
    );

    (contract_id, school, student, pool_id)
}

fn apply(client: &ContractClient, pool_id: u32, student: &Address) -> Result<(), soroban_sdk::Error> {
    client.try_apply_to_pool(
        &pool_id,
        student,
        &String::from_str(&client.env, "application data"),
    )
}

#[test]
fn application_before_deadline_is_accepted() {
    let env = Env::default();
    let (contract_id, _school, student, pool_id) = setup_pool(&env);
    let client = ContractClient::new(&env, &contract_id);

    env.ledger().set_timestamp(APPLICATION_DEADLINE - 1);

    apply(&client, pool_id, &student).expect("application one ledger before the deadline must succeed");
}

#[test]
fn application_at_deadline_is_rejected() {
    let env = Env::default();
    let (contract_id, _school, student, pool_id) = setup_pool(&env);
    let client = ContractClient::new(&env, &contract_id);

    env.ledger().set_timestamp(APPLICATION_DEADLINE);

    let err = apply(&client, pool_id, &student).unwrap_err();
    assert_eq!(err, ContractError::CampaignExpired.into());
}

#[test]
fn application_after_deadline_is_rejected() {
    let env = Env::default();
    let (contract_id, _school, student, pool_id) = setup_pool(&env);
    let client = ContractClient::new(&env, &contract_id);

    env.ledger().set_timestamp(APPLICATION_DEADLINE + 1);

    let err = apply(&client, pool_id, &student).unwrap_err();
    assert_eq!(err, ContractError::CampaignExpired.into());
}