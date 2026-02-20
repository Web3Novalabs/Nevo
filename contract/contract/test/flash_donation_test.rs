#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    (client, admin, token_address)
}

#[test]
fn test_flash_donation_protection() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    // Set initial sequence
    env.ledger().with_mut(|li| li.sequence = 100);

    // 1. Create a pool
    let name = String::from_str(&env, "Flash Test");
    let target = 10_000i128;
    let duration = 3600; // 1 hour
    let pool_config = PoolConfig {
        name,
        description: String::from_str(&env, "Desc"),
        target_amount: target,
        is_private: false,
        duration,
        created_at: env.ledger().timestamp(),
    };
    let pool_id = client.create_pool(&creator, &pool_config);

    // Give tokens to donor
    let token_admin = Address::generate(&env); // Not used but we need some admin logic if mocked
    let token = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    token.mint(&donor, &5000);

    // 2. Donate
    client.contribute(&pool_id, &donor, &token_address, &1000, &false);

    // 3. Attempt refund in the SAME ledger sequence
    // First, expire the pool to allow refund
    env.ledger().with_mut(|li| {
        li.timestamp += duration + 700000; // Pass deadline + 7 days grace period
    });

    let result = client.try_refund(&pool_id, &donor);
    assert_eq!(result, Err(Ok(CrowdfundingError::FlashDonationDetected)));

    // 4. Advance ledger sequence and try again
    env.ledger().with_mut(|li| {
        li.sequence += 1;
    });

    let result_success = client.try_refund(&pool_id, &donor);
    assert!(result_success.is_ok());
}
