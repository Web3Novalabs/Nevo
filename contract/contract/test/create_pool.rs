#![cfg(test)]

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, MAX_DESCRIPTION_LENGTH},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::StellarAssetClient,
    Address, Env, String, Symbol,
};

/// Helper: register the contract, initialize it, and return (client, admin, token_address).
fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token_address, &0);
    (client, admin, token_address)
}

/// Mint `amount` tokens to `to` using the SAC test helper.
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

#[test]
fn test_create_pool_success() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let creator = Address::generate(&env);
    let config = PoolConfig {
        name: String::from_str(&env, "Community Garden"),
        description: String::from_str(&env, "A garden for the neighborhood"),
        target_amount: 50_000i128,
        min_contribution: 0,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: creator.clone(),
    };

    mint(&env, &token_address, &creator, config.target_amount);
    let pool_id = client.create_pool(&creator, &config);
    assert_eq!(pool_id, 1);

    let saved = client.get_pool(&pool_id).unwrap();
    assert_eq!(saved.name, config.name);
    assert_eq!(saved.description, config.description);
    assert_eq!(saved.target_amount, config.target_amount);
    assert_eq!(saved.token_address, token_address);
    assert_eq!(saved.validator, config.validator);
}

#[test]
fn test_create_pool_invalid_token_fails() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let creator = Address::generate(&env);
    let wrong_token_admin = Address::generate(&env);
    let wrong_token = env
        .register_stellar_asset_contract_v2(wrong_token_admin)
        .address();

    let config = PoolConfig {
        name: String::from_str(&env, "Bad Token Pool"),
        description: String::from_str(&env, "This should fail"),
        target_amount: 10_000i128,
        min_contribution: 0,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address: wrong_token,
        validator: creator.clone(),
    };

    let result = client.try_create_pool(&creator, &config);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidToken)));
}

#[test]
#[should_panic(expected = "description too long")]
fn test_create_pool_panic_description_length() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let creator = Address::generate(&env);
    let long_desc = "a".repeat((MAX_DESCRIPTION_LENGTH + 1) as usize);

    let config = PoolConfig {
        name: String::from_str(&env, "Invalid Pool"),
        description: String::from_str(&env, &long_desc),
        target_amount: 1000,
        min_contribution: 0,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address,
        validator: creator.clone(),
    };

    client.create_pool(&creator, &config);
}

#[test]
fn test_create_pool_invalid_description_length() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let creator = Address::generate(&env);
    let long_desc = "a".repeat((MAX_DESCRIPTION_LENGTH + 1) as usize);

    let config = PoolConfig {
        name: String::from_str(&env, "Invalid Pool"),
        description: String::from_str(&env, &long_desc),
        target_amount: 1000,
        min_contribution: 0,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address,
    };

    let _result = client.try_create_pool(&creator, &config);
}

#[test]
fn test_create_pool_validation_logic() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    client.pause();

    let creator = Address::generate(&env);
    // No mint needed — the paused check fires before the balance check
    let config = PoolConfig {
        name: String::from_str(&env, "Paused Pool"),
        description: String::from_str(&env, "Desc"),
        target_amount: 1000,
        min_contribution: 0,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
        token_address,
    };

    let result = client.try_create_pool(&creator, &config);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractPaused)));
}

#[test]
fn test_create_pool_emits_event_created() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    client.initialize(&admin, &token_address, &0);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Community Garden");
    let description = String::from_str(&env, "A garden for the neighborhood");
    let target_amount = 50_000i128;
    let duration = 30 * 24 * 60 * 60u64;
    let created_at = env.ledger().timestamp();

    let config = PoolConfig {
        name: name.clone(),
        description: description.clone(),
        target_amount,
        min_contribution: 0,
        is_private: false,
        duration,
        created_at,
        token_address: token_address.clone(),
    };

    mint(&env, &token_address, &creator, target_amount);
    let _pool_id = client.create_pool(&creator, &config);
    let deadline = created_at + duration;

    let all_events = env.events().all();
    let event_created_symbol = Symbol::new(&env, "event_created");

    let found = all_events.iter().any(|(_contract, topics, data)| {
        if topics.is_empty() {
            return false;
        }
        use soroban_sdk::FromVal;
        let sym = Symbol::from_val(&env, &topics.get(0).unwrap());
        if sym != event_created_symbol {
            return false;
        }
        use soroban_sdk::TryFromVal;
        let decoded: Result<(String, i128, u64), _> =
            <(String, i128, u64)>::try_from_val(&env, &data);
        match decoded {
            Ok((n, amt, dl)) => n == name && amt == target_amount && dl == deadline,
            Err(_) => false,
        }
    });

    assert!(found, "event_created was not emitted by create_pool");
}
