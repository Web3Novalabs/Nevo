#![cfg(test)]

use soroban_sdk::{Address, Env, String};

use crate::{
    base::types::PoolMetadata,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup(env: &Env) -> CrowdfundingContractClient<'_> {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    client.initialize(&admin, &token_address, &0);
    client
}

fn create_pool(env: &Env, client: &CrowdfundingContractClient<'_>) -> u64 {
    let creator = Address::generate(env);
    let name = String::from_str(env, "Event Pool");
    let metadata = PoolMetadata {
        description: String::from_str(env, "A test event pool"),
        external_url: String::from_str(env, ""),
        image_hash: String::from_str(env, ""),
    };
    client.save_pool(
        &name,
        &metadata,
        &creator,
        &10_000i128,
        &(env.ledger().timestamp() + 86400),
        &None::<u32>,
        &None::<soroban_sdk::Vec<Address>>,
    )
}

#[test]
fn test_has_ticket_returns_false_when_no_ticket_issued() {
    let env = Env::default();
    let client = setup(&env);
    let pool_id = create_pool(&env, &client);
    let user = Address::generate(&env);

    assert!(!client.has_ticket(&pool_id, &user));
}

#[test]
fn test_has_ticket_returns_true_after_issue() {
    let env = Env::default();
    let client = setup(&env);
    let pool_id = create_pool(&env, &client);
    let user = Address::generate(&env);

    client.issue_ticket(&pool_id, &user);

    assert!(client.has_ticket(&pool_id, &user));
}

#[test]
fn test_has_ticket_is_per_user() {
    let env = Env::default();
    let client = setup(&env);
    let pool_id = create_pool(&env, &client);

    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);

    client.issue_ticket(&pool_id, &user_a);

    assert!(client.has_ticket(&pool_id, &user_a));
    assert!(!client.has_ticket(&pool_id, &user_b));
}

#[test]
fn test_has_ticket_is_per_pool() {
    let env = Env::default();
    let client = setup(&env);

    let pool_a = create_pool(&env, &client);
    let pool_b = create_pool(&env, &client);
    let user = Address::generate(&env);

    client.issue_ticket(&pool_a, &user);

    assert!(client.has_ticket(&pool_a, &user));
    assert!(!client.has_ticket(&pool_b, &user));
}

#[test]
fn test_issue_ticket_fails_for_nonexistent_pool() {
    let env = Env::default();
    let client = setup(&env);
    let user = Address::generate(&env);

    use crate::base::errors::CrowdfundingError;
    let result = client.try_issue_ticket(&9999, &user);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}
