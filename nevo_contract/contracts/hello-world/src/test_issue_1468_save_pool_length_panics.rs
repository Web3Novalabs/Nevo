#![cfg(test)]
//! Tests for `save_pool` length-check boundary panics and `pool.sponsor` auth - issue #1468.
//!
//! Covers:
//!   (1) Exact boundary behavior (`at limit` succeeds, `limit + 1` panics) for
//!       `MAX_DESCRIPTION_LENGTH` (500), `MAX_URL_LENGTH` (256), and
//!       `MAX_IMAGE_HASH_LENGTH` (64).
//!   (2) Verification that `save_pool` requires `pool.sponsor` authorization
//!       and rejects calls authorized only by a non-sponsor address.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, String, Vec,
};

fn make_ascii_string(env: &Env, len: usize) -> String {
    let bytes = vec![b'a'; len];
    let s = core::str::from_utf8(&bytes).unwrap();
    String::from_str(env, s)
}

#[test]
fn test_save_pool_at_exact_length_limits_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Boundary Pool"),
        &String::from_str(&env, "Initial description"),
        &10_000_000u128,
        &100_000u64,
    );

    let mut signers = Vec::new(&env);
    signers.push_back(sponsor.clone());

    let desc = make_ascii_string(&env, MAX_DESCRIPTION_LENGTH);
    let url = make_ascii_string(&env, MAX_URL_LENGTH);
    let image_hash = make_ascii_string(&env, MAX_IMAGE_HASH_LENGTH);

    client.save_pool(&pool_id, &desc, &url, &image_hash, &1u32, &signers);
}

#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_save_pool_description_over_limit_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Boundary Pool"),
        &String::from_str(&env, "Initial"),
        &10_000_000u128,
        &100_000u64,
    );

    let mut signers = Vec::new(&env);
    signers.push_back(sponsor);

    let desc = make_ascii_string(&env, MAX_DESCRIPTION_LENGTH + 1);
    let url = make_ascii_string(&env, MAX_URL_LENGTH);
    let image_hash = make_ascii_string(&env, MAX_IMAGE_HASH_LENGTH);

    client.save_pool(&pool_id, &desc, &url, &image_hash, &1u32, &signers);
}

#[test]
#[should_panic(expected = "URL exceeds maximum length")]
fn test_save_pool_url_over_limit_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Boundary Pool"),
        &String::from_str(&env, "Initial"),
        &10_000_000u128,
        &100_000u64,
    );

    let mut signers = Vec::new(&env);
    signers.push_back(sponsor);

    let desc = make_ascii_string(&env, MAX_DESCRIPTION_LENGTH);
    let url = make_ascii_string(&env, MAX_URL_LENGTH + 1);
    let image_hash = make_ascii_string(&env, MAX_IMAGE_HASH_LENGTH);

    client.save_pool(&pool_id, &desc, &url, &image_hash, &1u32, &signers);
}

#[test]
#[should_panic(expected = "Image hash exceeds maximum length")]
fn test_save_pool_image_hash_over_limit_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Boundary Pool"),
        &String::from_str(&env, "Initial"),
        &10_000_000u128,
        &100_000u64,
    );

    let mut signers = Vec::new(&env);
    signers.push_back(sponsor);

    let desc = make_ascii_string(&env, MAX_DESCRIPTION_LENGTH);
    let url = make_ascii_string(&env, MAX_URL_LENGTH);
    let image_hash = make_ascii_string(&env, MAX_IMAGE_HASH_LENGTH + 1);

    client.save_pool(&pool_id, &desc, &url, &image_hash, &1u32, &signers);
}

#[test]
fn test_save_pool_requires_pool_sponsor_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let outsider = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Auth Pool"),
        &String::from_str(&env, "Initial"),
        &10_000_000u128,
        &100_000u64,
    );

    let mut signers = Vec::new(&env);
    signers.push_back(sponsor.clone());
    let desc = String::from_str(&env, "Updated description");
    let url = String::from_str(&env, "https://nevo.example.org");
    let image_hash = String::from_str(&env, "abcd1234");

    // Authorizing as a non-sponsor address must fail
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "save_pool",
            args: (
                pool_id,
                desc.clone(),
                url.clone(),
                image_hash.clone(),
                1u32,
                signers.clone(),
            )
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client
        .try_save_pool(&pool_id, &desc, &url, &image_hash, &1u32, &signers)
        .is_err());

    // Authorizing as `pool.sponsor` succeeds
    env.mock_auths(&[MockAuth {
        address: &sponsor,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "save_pool",
            args: (pool_id, desc.clone(), url.clone(), image_hash.clone(), 1u32, signers.clone())
                .into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.save_pool(&pool_id, &desc, &url, &image_hash, &1u32, &signers);
}