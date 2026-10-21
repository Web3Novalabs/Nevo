#![cfg(test)]

//! Issue #1477: `create_pool_for_school` relationship and authorization policy.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup(env: &Env) -> ContractClient<'_> {
    let contract_id = env.register(Contract, ());
    ContractClient::new(env, &contract_id)
}

/// TEST A: `get_pool_school` immediately returns the exact school address
/// passed to `create_pool_for_school`.
#[test]
fn test_create_pool_for_school_links_exact_school_address() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let school = Address::generate(&env);
    client.register_school(&school, &BytesN::from_array(&env, &[8u8; 32]));

    let creator = Address::generate(&env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Linked Pool"),
        &String::from_str(&env, "Immediate link check"),
        &1_000_000u128,
        &school,
        &200_000u64,
    );

    assert_eq!(client.get_pool_school(&pool_id), school);
}

/// TEST B: any authorized creator may link a pool to any registered school.
/// The contract has no school-owner check — only `creator.require_auth()` and
/// `is_school_registered`.
#[test]
fn test_create_pool_for_school_allows_any_registered_school() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let school_a = Address::generate(&env);
    let school_b = Address::generate(&env);
    client.register_school(&school_a, &BytesN::from_array(&env, &[1u8; 32]));
    client.register_school(&school_b, &BytesN::from_array(&env, &[2u8; 32]));

    let creator = Address::generate(&env);
    assert_ne!(creator, school_a);
    assert_ne!(creator, school_b);

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Other School Pool"),
        &String::from_str(&env, "Creator is not the school"),
        &1_000_000u128,
        &school_b,
        &200_000u64,
    );

    assert_eq!(
        client.get_pool_school(&pool_id),
        school_b,
        "authorized creators may associate a pool with any registered school"
    );
}
