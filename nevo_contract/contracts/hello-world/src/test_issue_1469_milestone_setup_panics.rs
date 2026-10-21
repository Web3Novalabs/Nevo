#![cfg(test)]
//! Locks in the untyped panics in `setup_application_milestones` — issue #1469.

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env, String};

const GOAL: u128 = 1_000_000;

/// Creates a school-linked pool with `GOAL` and an approved application, so
/// `setup_application_milestones` reaches its milestone checks.
fn setup_approved_application(env: &Env) -> (ContractClient<'_>, u32, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    client.set_admin(&Address::generate(env));
    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[1u8; 32]));

    let pool_id = client.create_pool_for_school(
        &Address::generate(env),
        &String::from_str(env, "Milestone Pool"),
        &String::from_str(env, "Description"),
        &GOAL,
        &school,
        &200_000u64,
    );

    let student = Address::generate(env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(env, "application"));
    // approve_application currently fails with Error(Auth, ExistingValue)
    // (school auth is required twice), so approve via set_application_status.
    client.set_application_status(
        &school,
        &pool_id,
        &student,
        &String::from_str(env, APPLICATION_STATUS_APPROVED),
    );

    (client, pool_id, student)
}

#[test]
#[should_panic(expected = "Milestones required")]
fn test_setup_milestones_empty_list_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student) = setup_approved_application(&env);

    client.setup_application_milestones(&pool_id, &student, &Vec::new(&env));
}

#[test]
#[should_panic(expected = "Milestone total must equal pool goal")]
fn test_setup_milestones_sum_one_below_goal_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student) = setup_approved_application(&env);

    let milestones = vec![
        &env,
        Milestone { amount: GOAL / 2 },
        Milestone {
            amount: GOAL / 2 - 1,
        },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

#[test]
#[should_panic(expected = "Milestone total must equal pool goal")]
fn test_setup_milestones_sum_one_above_goal_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student) = setup_approved_application(&env);

    let milestones = vec![
        &env,
        Milestone { amount: GOAL / 2 },
        Milestone {
            amount: GOAL / 2 + 1,
        },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

#[test]
fn test_setup_milestones_sum_equal_to_goal_is_stored() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student) = setup_approved_application(&env);

    let milestones = vec![
        &env,
        Milestone { amount: GOAL / 2 },
        Milestone { amount: GOAL / 2 },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);

    assert_eq!(client.get_milestones(&pool_id, &student), milestones);
}

#[test]
#[should_panic(expected = "Milestone amount overflow")]
fn test_setup_milestones_sum_overflow_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student) = setup_approved_application(&env);

    let milestones = vec![
        &env,
        Milestone { amount: u128::MAX },
        Milestone { amount: 1 },
    ];
    client.setup_application_milestones(&pool_id, &student, &milestones);
}
