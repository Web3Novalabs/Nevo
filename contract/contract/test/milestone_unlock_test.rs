#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    token, Address, Env, String,
};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{MilestoneDetails, PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::StellarAssetClient<'a> {
    let token_address = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    token::StellarAssetClient::new(env, &token_address)
}

fn setup() -> (Env, CrowdfundingContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_client = create_token_contract(&env, &admin);
    let token = token_client.address.clone();

    client.initialize(&admin, &token, &1000);

    (env, client, admin, token)
}

fn create_pool_with_validator(
    client: &CrowdfundingContractClient<'_>,
    env: &Env,
    creator: &Address,
    validator: &Address,
    token: &Address,
) -> u64 {
    let config = PoolConfig {
        name: String::from_str(env, "Scholarship Pool"),
        description: String::from_str(env, "A pool for scholarship applications"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: validator.clone(),
    };
    client.create_pool(creator, &config)
}

#[test]
fn test_create_milestone_success() {
    let (env, client, admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    let milestone_index = 1;
    let description = String::from_str(&env, "Complete first semester with GPA >= 3.0");
    let unlock_time = env.ledger().timestamp() + 86_400; // 1 day from now
    
    let result = client.try_create_milestone(&pool_id, &milestone_index, &description, &unlock_time);
    assert_eq!(result, Ok(Ok(())));
    
    // Verify milestone was created
    let milestone = client.get_milestone(&pool_id, &milestone_index);
    assert_eq!(milestone.pool_id, pool_id);
    assert_eq!(milestone.milestone_index, milestone_index);
    assert_eq!(milestone.description, description);
    assert_eq!(milestone.unlock_time, unlock_time);
    assert_eq!(milestone.is_unlocked, false);
    assert_eq!(milestone.performance_override, false);
}

#[test]
fn test_create_milestone_unauthorized() {
    let (env, client, _admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    env.mock_all_auths_allowing_non_root_auth();
    
    let milestone_index = 1;
    let description = String::from_str(&env, "Complete first semester");
    let unlock_time = env.ledger().timestamp() + 86_400;
    
    // Try to create milestone as unauthorized user
    let result = client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &unauthorized,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &client.address,
                fn_name: "create_milestone",
                args: (pool_id, milestone_index, description.clone(), unlock_time).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_create_milestone(&pool_id, &milestone_index, &description, &unlock_time);
    
    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_create_milestone_pool_not_found() {
    let (env, client, _admin, _token) = setup();
    
    let nonexistent_pool_id = 999;
    let milestone_index = 1;
    let description = String::from_str(&env, "Complete first semester");
    let unlock_time = env.ledger().timestamp() + 86_400;
    
    let result = client.try_create_milestone(&nonexistent_pool_id, &milestone_index, &description, &unlock_time);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_unlock_performance_milestone_success() {
    let (env, client, admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    // Create a milestone
    let milestone_index = 1;
    let description = String::from_str(&env, "Complete first semester with GPA >= 3.0");
    let unlock_time = env.ledger().timestamp() + 86_400; // Future unlock time
    
    client.create_milestone(&pool_id, &milestone_index, &description, &unlock_time);
    
    // Unlock milestone via performance override
    let result = client.try_unlock_performance_milestone(&pool_id, &milestone_index, &validator);
    assert_eq!(result, Ok(Ok(())));
    
    // Verify milestone is unlocked
    let milestone = client.get_milestone(&pool_id, &milestone_index);
    assert_eq!(milestone.is_unlocked, true);
    assert_eq!(milestone.performance_override, true);
    assert_eq!(milestone.unlocked_by, Some(validator.clone()));
    assert!(milestone.unlocked_at.is_some());
}

#[test]
fn test_unlock_performance_milestone_unauthorized_validator() {
    let (env, client, admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    let unauthorized_validator = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    // Create a milestone
    let milestone_index = 1;
    let description = String::from_str(&env, "Complete first semester");
    let unlock_time = env.ledger().timestamp() + 86_400;
    
    client.create_milestone(&pool_id, &milestone_index, &description, &unlock_time);
    
    env.mock_all_auths_allowing_non_root_auth();
    
    // Try to unlock with unauthorized validator
    let result = client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &unauthorized_validator,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &client.address,
                fn_name: "unlock_performance_milestone",
                args: (pool_id, milestone_index, unauthorized_validator.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_unlock_performance_milestone(&pool_id, &milestone_index, &unauthorized_validator);
    
    assert_eq!(result, Err(Ok(CrowdfundingError::NotPoolValidator)));
}

#[test]
fn test_unlock_performance_milestone_not_found() {
    let (env, client, admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    let nonexistent_milestone_index = 999;
    
    let result = client.try_unlock_performance_milestone(&pool_id, &nonexistent_milestone_index, &validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::MilestoneNotFound)));
}

#[test]
fn test_unlock_performance_milestone_already_unlocked() {
    let (env, client, admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    // Create and unlock a milestone
    let milestone_index = 1;
    let description = String::from_str(&env, "Complete first semester");
    let unlock_time = env.ledger().timestamp() + 86_400;
    
    client.create_milestone(&pool_id, &milestone_index, &description, &unlock_time);
    client.unlock_performance_milestone(&pool_id, &milestone_index, &validator);
    
    // Try to unlock again
    let result = client.try_unlock_performance_milestone(&pool_id, &milestone_index, &validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::MilestoneAlreadyUnlocked)));
}

#[test]
fn test_get_milestone_not_found() {
    let (env, client, admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    let nonexistent_milestone_index = 999;
    
    let result = client.try_get_milestone(&pool_id, &nonexistent_milestone_index);
    assert_eq!(result, Err(Ok(CrowdfundingError::MilestoneNotFound)));
}

#[test]
fn test_unlock_performance_milestone_pool_not_found() {
    let (env, client, _admin, _token) = setup();
    let validator = Address::generate(&env);
    
    let nonexistent_pool_id = 999;
    let milestone_index = 1;
    
    let result = client.try_unlock_performance_milestone(&nonexistent_pool_id, &milestone_index, &validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_milestone_unlock_events() {
    let (env, client, admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    // Create milestone
    let milestone_index = 1;
    let description = String::from_str(&env, "Complete first semester");
    let unlock_time = env.ledger().timestamp() + 86_400;
    
    client.create_milestone(&pool_id, &milestone_index, &description, &unlock_time);
    
    // Unlock milestone
    client.unlock_performance_milestone(&pool_id, &milestone_index, &validator);
    
    // Verify events were emitted
    let events = env.events().all();
    
    // Check for milestone_created event
    let milestone_created_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &soroban_sdk::Symbol::new(&env, "milestone_created")
    });
    assert!(milestone_created_event.is_some());
    
    // Check for milestone_unlocked event
    let milestone_unlocked_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &soroban_sdk::Symbol::new(&env, "milestone_unlocked")
    });
    assert!(milestone_unlocked_event.is_some());
}

#[test]
fn test_multiple_milestones_same_pool() {
    let (env, client, admin, token) = setup();
    let creator = Address::generate(&env);
    let validator = Address::generate(&env);
    
    let pool_id = create_pool_with_validator(&client, &env, &creator, &validator, &token);
    
    // Create multiple milestones
    for i in 1..=3 {
        let description = String::from_str(&env, &format!("Milestone {}", i));
        let unlock_time = env.ledger().timestamp() + (i as u64 * 86_400);
        
        client.create_milestone(&pool_id, &i, &description, &unlock_time);
    }
    
    // Unlock milestone 2 via performance override
    client.unlock_performance_milestone(&pool_id, &2, &validator);
    
    // Verify milestone 2 is unlocked, others are not
    let milestone1 = client.get_milestone(&pool_id, &1);
    let milestone2 = client.get_milestone(&pool_id, &2);
    let milestone3 = client.get_milestone(&pool_id, &3);
    
    assert_eq!(milestone1.is_unlocked, false);
    assert_eq!(milestone2.is_unlocked, true);
    assert_eq!(milestone2.performance_override, true);
    assert_eq!(milestone3.is_unlocked, false);
}