#![cfg(test)]

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{ApplicationStatus, Milestone, PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Vec};

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

fn create_pool(env: &Env, client: &CrowdfundingContractClient<'_>, token_address: &Address) -> (u64, Address) {
    let creator = Address::generate(env);
    let validator = Address::generate(env);
    let config = PoolConfig {
        name: String::from_str(env, "Milestone Test Pool"),
        description: String::from_str(env, "Pool for testing milestone functionality"),
        target_amount: 100_000i128,
        min_contribution: 0,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: env.ledger().timestamp(),
        token_address: token_address.clone(),
        validator: validator.clone(),
    };

    let pool_id = client.create_pool(&creator, &config);
    (pool_id, validator)
}

#[test]
fn test_milestone_functionality() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let (pool_id, validator) = create_pool(&env, &client, &token_address);
    let applicant = Address::generate(&env);

    // Apply for scholarship using the 2-parameter interface
    client.apply_for_scholarship(&pool_id, &applicant);

    // Approve the application
    client.approve_application(&(pool_id as u32), &applicant);

    // Verify application is approved
    let application = client.get_application(&pool_id, &applicant);
    assert_eq!(application.status, ApplicationStatus::Approved);

    // Test milestone creation using the new methods
    let unlock_date = env.ledger().timestamp() + 86400; // 1 day from now
    let milestone_amount = 5000i128;

    let result = client.try_add_milestone(&pool_id, &applicant, &unlock_date, &milestone_amount);
    assert_eq!(result, Ok(Ok(())));

    // Test milestone unlocking (should fail because date hasn't passed)
    let unlock_result = client.try_unlock_milestone(&pool_id, &applicant, &0u32);
    assert_eq!(unlock_result, Err(Ok(CrowdfundingError::InvalidAmount))); // Reusing error for date not passed

    // Test getting application details
    let app_details = client.get_application_details(&pool_id, &applicant);
    assert_eq!(app_details.milestones.len(), 1);
    assert_eq!(app_details.amount_claimed, 0);
    
    let milestone = app_details.milestones.get(0).unwrap();
    assert_eq!(milestone.unlock_date, unlock_date);
    assert_eq!(milestone.amount, milestone_amount);
    assert_eq!(milestone.unlocked, false);
}

#[test]
fn test_milestone_struct_properties() {
    let env = Env::default();
    
    // Test Milestone struct creation and properties
    let milestone = Milestone {
        unlock_date: 1700000000,
        unlocked: false,
        amount: 1000,
    };

    assert_eq!(milestone.unlock_date, 1700000000);
    assert_eq!(milestone.unlocked, false);
    assert_eq!(milestone.amount, 1000);
}