#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env, Symbol, TryFromVal, testutils::Events};

use crate::{
    base::types::PoolConfig,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

#[test]
fn test_event_repro() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(token_admin).address();
    client.initialize(&admin, &token, &0);

    let creator = Address::generate(&env);
    let config = PoolConfig {
        name: soroban_sdk::String::from_str(&env, "Event Pool"),
        description: soroban_sdk::String::from_str(&env, "Test event"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: 0,
        token_address: token.clone(),
    };
    client.create_pool(&creator, &config);

    let all_events = env.events().all();
    assert!(all_events.len() > 0, "No events after create_pool. Count: {}", all_events.len());
}
