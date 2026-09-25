#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn create_pool(env: &Env, client: &ContractClient) -> (u32, Address) {
    let sponsor = Address::generate(env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(env, "Multisig Pool"),
        &String::from_str(env, "Pool description"),
        &1_000_000u128,
        &200_000u64,
    );
    (pool_id, sponsor)
}

#[test]
fn test_save_pool_valid_signatures_and_signers_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let (pool_id, _sponsor) = create_pool(&env, &client);

    let signer_a = Address::generate(&env);
    let signer_b = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer_a.clone(), signer_b.clone()]);

    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Updated description"),
        &String::from_str(&env, "https://example.com"),
        &String::from_str(&env, "abcd"),
        &2u32,
        &signers,
    );

    let (required, stored) = client.get_pool_signers(&pool_id);
    assert_eq!(required, 2);
    assert_eq!(stored.len(), 2);
    assert_eq!(stored.get(0).unwrap(), signer_a);
    assert_eq!(stored.get(1).unwrap(), signer_b);
    assert_eq!(client.get_pool(&pool_id).0, pool_id);
}

#[test]
#[should_panic(expected = "required_signatures exceeds signer count")]
fn test_save_pool_required_signatures_greater_than_signers_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let (pool_id, _) = create_pool(&env, &client);

    let signers = Vec::from_array(&env, [Address::generate(&env)]);
    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Description"),
        &String::from_str(&env, "https://example.com"),
        &String::from_str(&env, "abcd"),
        &3u32,
        &signers,
    );
}

#[test]
#[should_panic(expected = "Zero required_signatures")]
fn test_save_pool_zero_required_signatures_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let (pool_id, _) = create_pool(&env, &client);

    let signers = Vec::from_array(&env, [Address::generate(&env), Address::generate(&env)]);
    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Description"),
        &String::from_str(&env, "https://example.com"),
        &String::from_str(&env, "abcd"),
        &0u32,
        &signers,
    );
}

#[test]
#[should_panic(expected = "Empty signers list")]
fn test_save_pool_empty_signers_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let (pool_id, _) = create_pool(&env, &client);

    let signers: Vec<Address> = Vec::new(&env);
    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Description"),
        &String::from_str(&env, "https://example.com"),
        &String::from_str(&env, "abcd"),
        &1u32,
        &signers,
    );
}

#[test]
#[should_panic(expected = "Mismatched multi-signature parameters")]
fn test_save_pool_mismatched_signers_fail() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let (pool_id, _) = create_pool(&env, &client);

    let signer = Address::generate(&env);
    let signers = Vec::from_array(&env, [signer.clone(), signer]);
    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Description"),
        &String::from_str(&env, "https://example.com"),
        &String::from_str(&env, "abcd"),
        &2u32,
        &signers,
    );
}
