#![cfg(test)]
extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::base::types::{PoolConfig, PoolState};
use crate::crowdfunding::{CrowdfundingContract, CrowdfundingContractClient};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (
    soroban_sdk::token::Client<'a>,
    soroban_sdk::token::StellarAssetClient<'a>,
) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        soroban_sdk::token::Client::new(env, &contract_address.address()),
        soroban_sdk::token::StellarAssetClient::new(env, &contract_address.address()),
    )
}

#[test]
fn test_fund_edu_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    // Use an advanced timestamp so we don't hit 0 issues.
    env.ledger().with_mut(|l| l.timestamp = 100_000);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Identitites
    let admin = Address::generate(&env);
    let sponsor = Address::generate(&env);
    let student = Address::generate(&env);
    // In this MVP, the contract admin acts as the default validator since they can call
    // `verify_cause`. Alternatively, an admin verifies causes directly.
    let validator = admin.clone();

    // 1. Setup Token and Contract
    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token_contract(&env, &token_admin);
    let token_address = token.address.clone();

    // Initialize the CrowdfundingContract
    client.initialize(&admin, &token_address, &0i128);

    // Give the sponsor some funds to start
    let deposit_amount = 50_000_000i128; // 50 USDC for example
    token_admin_client.mint(&sponsor, &deposit_amount);

    assert_eq!(token.balance(&sponsor), deposit_amount);
    assert_eq!(token.balance(&student), 0);

    // 2. Sponsor Creates Pool
    let config = PoolConfig {
        name: String::from_str(&env, "STEM 2026 Q1"),
        description: String::from_str(&env, "Education fund"),
        target_amount: deposit_amount,
        min_contribution: 0,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: admin.clone(),
    };

    let pool_id = client.create_pool(&sponsor, &config);

    // Sponsor funds the pool
    client.contribute(&pool_id, &sponsor, &token_address, &deposit_amount, &false);
    assert_eq!(token.balance(&sponsor), 0);
    assert_eq!(token.balance(&contract_id), deposit_amount); // Pool owns funds

    // 3. Student Applies
    // Student registers by making a 0-amount contribution
    client.contribute(&pool_id, &student, &token_address, &0i128, &false);

    // 4. Approve
    // Validator (Admin in this case) reviews and approves the student off-chain, records on-chain
    client.verify_cause(&student);
    assert!(client.is_cause_verified(&student));

    // 5. Claim
    // Student executes `claim_pool_funds` after being approved
    client.claim_pool_funds(&pool_id, &student);

    // Assert balances post-claim
    assert_eq!(token.balance(&contract_id), 0);
    assert_eq!(token.balance(&student), deposit_amount);

    // 6. Sponsor (or Admin) can now close the disbursed pool
    client.close_pool(&pool_id, &sponsor);
    assert!(client.is_closed(&pool_id));
}
