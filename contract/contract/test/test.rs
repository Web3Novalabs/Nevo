#![cfg(test)]
extern crate alloc;

use crate::{
    base::types::{PoolConfig, PoolState},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::Client as TokenClient,
    Address, Env, String,
};

#[test]
fn test_create_pool_with_deposit() {
    let env = Env::default();
    env.mock_all_auths();

    // Register contract
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Setup admin and token
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let token = TokenClient::new(&env, &token_address);

    // Initialize the contract with a creation fee of 100
    let creation_fee = 100i128;
    client.initialize(&admin, &token_address, &creation_fee);

    // Setup creator and give them tokens
    let creator = Address::generate(&env);
    let initial_balance = 1000i128;
    token.mint(&creator, &initial_balance);

    // Advance ledger so timestamp is > 0
    env.ledger().with_mut(|li| li.timestamp = 1000);

    // Create a pool configuration
    let config = PoolConfig {
        name: String::from_str(&env, "Test Pool"),
        description: String::from_str(&env, "A test pool to verify deposit"),
        target_amount: 50_000,
        min_contribution: 100,
        is_private: false,
        duration: 86400, // 1 day
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: creator.clone(),
        application_deadline: env.ledger().timestamp(),
        milestones: soroban_sdk::Vec::new(&env),
    };

    // Call create_pool
    let pool_id = client.create_pool(&creator, &config);

    // Verify state variables match expected outcomes
    assert_eq!(pool_id, 1);
    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.name, config.name);
    assert_eq!(pool.description, config.description);
    assert_eq!(pool.target_amount, config.target_amount);

    // Verify token balances reflect the deposit transfer
    assert_eq!(token.balance(&creator), initial_balance - creation_fee);
    assert_eq!(token.balance(&contract_id), creation_fee);
}
