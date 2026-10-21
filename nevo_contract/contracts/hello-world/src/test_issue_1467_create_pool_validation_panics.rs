#![cfg(test)]
//! Locks in the current panic text for `create_pool` input validation —
//! issue #1467.
//!
//! On main only the two description checks still panic with raw strings;
//! an empty title already uses the typed `ContractError::InvalidPoolName`,
//! there is no title-length check, and `create_pool` takes no duration.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn create_pool_with(env: &Env, title: &str, description: &str) -> u32 {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    client.create_pool(
        &Address::generate(env),
        &String::from_str(env, title),
        &String::from_str(env, description),
        &1_000_000u128,
        &200_000u64,
    )
}

#[test]
#[should_panic(expected = "Description cannot be empty")]
fn test_create_pool_empty_description_panics_with_raw_string() {
    let env = Env::default();
    create_pool_with(&env, "Valid title", "");
}

#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_create_pool_too_long_description_panics_with_raw_string() {
    let env = Env::default();
    let description = "a".repeat(MAX_DESCRIPTION_LENGTH + 1);
    create_pool_with(&env, "Valid title", &description);
}

#[test]
fn test_create_pool_description_at_max_length_is_accepted() {
    let env = Env::default();
    let description = "a".repeat(MAX_DESCRIPTION_LENGTH);
    assert_eq!(create_pool_with(&env, "Valid title", &description), 1);
}

/// Unlike the description checks, an empty title is already typed.
#[test]
#[should_panic(expected = "Error(Contract, #16)")]
fn test_create_pool_empty_title_is_typed_invalid_pool_name() {
    let env = Env::default();
    create_pool_with(&env, "", "A real description");
}

/// `MAX_TITLE_LENGTH` exists but `create_pool` never checks it.
#[test]
fn test_create_pool_title_longer_than_max_is_not_rejected() {
    let env = Env::default();
    let title = "t".repeat(MAX_TITLE_LENGTH as usize + 1);
    assert_eq!(create_pool_with(&env, &title, "A real description"), 1);
}
