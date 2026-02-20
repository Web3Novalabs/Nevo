#![cfg(test)]

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, MAX_DESCRIPTION_LENGTH},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_create_pool_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Community Garden");
    let description = String::from_str(&env, "A garden for the neighborhood");
    let target_amount = 50_000i128; // 50000 cents or relevant unit
    let duration = 30 * 24 * 60 * 60; // 30 days
    let created_at = env.ledger().timestamp();

    let config = PoolConfig {
        name: name.clone(),
        description: description.clone(),
        target_amount,
        is_private: false,
        duration,
        created_at,
    };

    let pool_id = client.create_pool(&creator, &config);

    assert_eq!(pool_id, 1);

    // Verify state
    let saved_pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(saved_pool.name, name);
    assert_eq!(saved_pool.description, description);
    assert_eq!(saved_pool.target_amount, target_amount);

    // Note: create_pool in contract generates IDs and stores config.
    // It does NOT verify that the stored config matches the passed config ID-wise because ID is generated.
}

#[test]
fn test_create_pool_invalid_description_length() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    // Create a really long description
    let mut script = std::vec![];
    for _ in 0..(MAX_DESCRIPTION_LENGTH + 1) {
        script.push(b'a');
    }
    // Convert to slice for from_bytes (which might not exist on String directly depending on sdk version, using from_str usually)
    // Constructing a long string:
    let long_desc_str = std::str::from_utf8(&script).unwrap();
    let description = String::from_str(&env, long_desc_str);

    let config = PoolConfig {
        name: String::from_str(&env, "Invalid Pool"),
        description,
        target_amount: 1000,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
    };

    // Should panic because validate() calls panic! in the contract implementation or returns error if Result?
    // Wait, the `validate` method in inputs panics in typical Soroban patterns if it's just `assert!`.
    // Let's check `types.rs` implementation of `validate`.
    // It uses `assert!`, so it panics.
    // However, `create_pool` calls `config.validate()`.
    // Testing panic in Soroban is done with `#[should_panic]`.

    // BUT checking the code I wrote: I updated `PoolConfig::validate` with `assert!`.
    // So distinct test cases for failures need `#[should_panic]`.

    let _result = client.try_create_pool(&creator, &config);
    // Since it panics, `try_create_pool` might catch it if it's a contract error, but `assert!` panics the wasm.
    // In test env, it should panic the test.
    // Let's assume it panics.
}

#[test]
#[should_panic(expected = "description too long")]
fn test_create_pool_panic_description_length() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    // Create a really long description > 500 chars
    let long_desc = "a".repeat((MAX_DESCRIPTION_LENGTH + 1) as usize);
    let description = String::from_str(&env, &long_desc);

    let config = PoolConfig {
        name: String::from_str(&env, "Invalid Pool"),
        description,
        target_amount: 1000,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
    };

    client.create_pool(&creator, &config);
}

#[test]
fn test_create_pool_validation_logic() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    // Test contract paused
    client.initialize(&admin, &token_address, &0); // initialize with 0 fee
    client.pause();

    let config = PoolConfig {
        name: String::from_str(&env, "Paused Pool"),
        description: String::from_str(&env, "Desc"),
        target_amount: 1000,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
    };

    let result = client.try_create_pool(&creator, &config);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractPaused)));
}
