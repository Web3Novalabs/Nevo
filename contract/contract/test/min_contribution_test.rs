#![cfg(test)]

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_contribute_below_minimum_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    // Initialize contract so storage keys exist
    client.initialize(&admin, &token_address, &0);

    let creator = Address::generate(&env);
    let config = PoolConfig {
        name: String::from_str(&env, "Min Pool"),
        description: String::from_str(&env, "Pool with min"),
        target_amount: 1000,
        min_contribution: 5,
        is_private: false,
        duration: 86400,
        created_at: env.ledger().timestamp(),
    };

    let pool_id = client.create_pool(&creator, &config);

    let contributor = Address::generate(&env);

    // Attempt to contribute less than min (1 < 5)
    let result = client.try_contribute(&pool_id, &contributor, &token_address, &1i128, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}
