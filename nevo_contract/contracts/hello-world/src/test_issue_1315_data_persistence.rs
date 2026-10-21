#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, Address, BytesN, Env, String, Symbol,
};

#[test]
fn test_persistence_pool_and_metadata_survive_later_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Persistent pool"),
        &String::from_str(&env, "Original description"),
        &1_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &Address::generate(&env), &250u128);

    assert_eq!(client.get_pool(&pool_id).3, 250u128);
    assert_eq!(
        client.get_pool_metadata(&pool_id).1,
        String::from_str(&env, "Original description")
    );
}
#[test]
fn test_persistence_saved_metadata_updates_without_corrupting_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metadata pool"),
        &String::from_str(&env, "Initial"),
        &2_000u128,
        &100_000u64,
    );

    client.save_pool(
        &pool_id,
        &String::from_str(&env, "First"),
        &String::from_str(&env, "https://one.example"),
        &String::from_str(&env, "hash-one"),
    );
    client.save_pool(
        &pool_id,
        &String::from_str(&env, "Second"),
        &String::from_str(&env, "https://two.example"),
        &String::from_str(&env, "hash-two"),
    );

    assert_eq!(
        client.get_saved_pool_metadata(&pool_id),
        (
            String::from_str(&env, "Second"),
            String::from_str(&env, "https://two.example"),
            String::from_str(&env, "hash-two"),
        )
    );
    assert_eq!(client.get_pool(&pool_id).2, 2_000u128);
}

#[test]
fn test_persistence_registered_school_hash_survives_contract_reregistration() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let hash = BytesN::from_array(&env, &[7u8; 32]);

    client.set_admin(&admin);
    client.register_school(&school, &hash);

    let upgraded_id = env.register_at(&contract_id, Contract, ());
    let upgraded_client = ContractClient::new(&env, &upgraded_id);

    assert!(upgraded_client.is_school_registered(&school));
    assert_eq!(upgraded_client.get_school_metadata(&school), hash);
}

#[test]
fn test_persistence_cleanup_removes_completed_emergency_request() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin);
    let token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token.address());
    token_client.mint(&contract_id, &100i128);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cleanup pool"),
        &String::from_str(&env, "Test"),
        &1_000u128,
        &100_000u64,
    );
    client.request_emergency_withdraw(&admin, &pool_id, &token.address(), &100i128);

    env.ledger().set_timestamp(GRACE_PERIOD_SECS + 1);
    client.execute_emergency_withdraw(&pool_id);

    let key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
    assert!(!env.as_contract(&contract_id, || {
        env.storage().persistent().has(&key)
    }));
}
