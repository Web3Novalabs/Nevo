#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

// ============= ISSUE #1294: CAMPAIGN LIST MANAGEMENT TESTS =============

/// Test: get_all_campaigns returns an empty list initially
#[test]
fn test_get_all_campaigns_empty_initially() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 0);
}

/// Test: a single created campaign appears in the returned list
#[test]
fn test_get_all_campaigns_single() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Single Campaign"),
        &String::from_str(&env, "Only one campaign"),
        &1_000_000_000u128,
        &200_000u64,
    );

    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 1);
    assert_eq!(campaigns.get(0).unwrap(), pool_id);
}

/// Test: multiple created campaigns are all listed
#[test]
fn test_get_all_campaigns_multiple() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Campaign One"),
        &String::from_str(&env, "First campaign"),
        &1_000_000_000u128,
        &200_000u64,
    );
    let id2 = client.create_pool(
        &creator,
        &String::from_str(&env, "Campaign Two"),
        &String::from_str(&env, "Second campaign"),
        &2_000_000_000u128,
        &200_000u64,
    );
    let id3 = client.create_pool(
        &creator,
        &String::from_str(&env, "Campaign Three"),
        &String::from_str(&env, "Third campaign"),
        &3_000_000_000u128,
        &200_000u64,
    );

    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 3);
    assert!(campaigns.contains(id1));
    assert!(campaigns.contains(id2));
    assert!(campaigns.contains(id3));
}

/// Test: campaign order is preserved in the returned list
#[test]
fn test_get_all_campaigns_order_preserved() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Order One"),
        &String::from_str(&env, "First in order"),
        &1_000_000_000u128,
        &200_000u64,
    );
    let id2 = client.create_pool(
        &creator,
        &String::from_str(&env, "Order Two"),
        &String::from_str(&env, "Second in order"),
        &1_000_000_000u128,
        &200_000u64,
    );
    let id3 = client.create_pool(
        &creator,
        &String::from_str(&env, "Order Three"),
        &String::from_str(&env, "Third in order"),
        &1_000_000_000u128,
        &200_000u64,
    );

    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 3);
    assert_eq!(campaigns.get(0).unwrap(), id1);
    assert_eq!(campaigns.get(1).unwrap(), id2);
    assert_eq!(campaigns.get(2).unwrap(), id3);
}

/// Test: list updates with new campaigns
#[test]
fn test_get_all_campaigns_updates_with_new_campaigns() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    assert_eq!(client.get_all_campaigns().len(), 0);

    let id1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Update One"),
        &String::from_str(&env, "First added"),
        &1_000_000_000u128,
        &200_000u64,
    );
    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 1);
    assert_eq!(campaigns.get(0).unwrap(), id1);

    let id2 = client.create_pool(
        &creator,
        &String::from_str(&env, "Update Two"),
        &String::from_str(&env, "Second added"),
        &1_000_000_000u128,
        &200_000u64,
    );
    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 2);
    assert_eq!(campaigns.get(0).unwrap(), id1);
    assert_eq!(campaigns.get(1).unwrap(), id2);
}

// ============= ISSUE #1090: INTEGRATION TESTS FOR CAMPAIGN LIFECYCLE =============

/// Test 1: Create campaign (pool) with a creation fee
#[test]
fn test_campaign_create_with_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &500_000i128);

    assert_eq!(client.get_creation_fee(), 500_000i128);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Campaign with Fee"),
        &String::from_str(&env, "Testing fee integration"),
        &10_000_000_000u128,
        &200_000u64,
    );

    assert_eq!(pool_id, 1);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.2, 10_000_000_000u128);
}

/// Test 2: Multiple users donate to the same campaign
#[test]
fn test_campaign_multiple_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Community Fundraiser"),
        &String::from_str(&env, "Multiple donors"),
        &1_000_000_000_000u128,
        &200_000u64,
    );

    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);

    client.donate(&pool_id, &donor1, &100_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 100_000_000u128);

    client.donate(&pool_id, &donor2, &200_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 300_000_000u128);

    client.donate(&pool_id, &donor3, &50_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 350_000_000u128);
}

/// Test 3: Campaign reaches its funding goal
#[test]
fn test_campaign_reaches_goal() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = 500_000_000u128;
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Goal Reached Campaign"),
        &String::from_str(&env, "Reach the goal"),
        &goal,
        &200_000u64,
    );

    let donor1 = Address::generate(&env);
    client.donate(&pool_id, &donor1, &300_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 300_000_000u128);

    let donor2 = Address::generate(&env);
    client.donate(&pool_id, &donor2, &200_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), goal);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, goal);
    assert_eq!(pool.4, false);
}

/// Test 4: Donations to a closed campaign fail
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_campaign_donations_fail_after_close() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closed Campaign"),
        &String::from_str(&env, "Will be closed"),
        &1_000_000_000u128,
        &200_000u64,
    );

    client.donate(&pool_id, &Address::generate(&env), &500_000_000u128);
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
}

/// Test 5: All balances and metrics correct throughout lifecycle
#[test]
fn test_campaign_balances_correct_throughout() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = 1_000_000_000u128;
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Lifecycle Campaign"),
        &String::from_str(&env, "Full lifecycle test"),
        &goal,
        &200_000u64,
    );

    // Initial state
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 0u128);
    assert_eq!(pool.4, false);

    // First donation
    let donor1 = Address::generate(&env);
    client.donate(&pool_id, &donor1, &400_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 400_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor1), 400_000_000u128);

    // Second donation
    let donor2 = Address::generate(&env);
    client.donate(&pool_id, &donor2, &300_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 700_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor2), 300_000_000u128);

    // Third donation from first donor
    client.donate(&pool_id, &donor1, &300_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), goal);
    assert_eq!(client.get_contribution(&pool_id, &donor1), 700_000_000u128);

    // Goal reached - verify metrics
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, goal);
    assert_eq!(pool.2, goal);
    assert_eq!(pool.4, false);
}

/// Test 6: Donations to a Paused campaign fail
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_campaign_donations_fail_when_paused() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Paused Campaign"),
        &String::from_str(&env, "Will be paused"),
        &1_000_000_000u128,
        &200_000u64,
    );

    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
    client.set_pool_state(&pool_id, &PoolState::Paused);

    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
}

/// Test 7: Donations to a Cancelled campaign fail
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_campaign_donations_fail_when_cancelled() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Cancelled Campaign"),
        &String::from_str(&env, "Will be cancelled"),
        &1_000_000_000u128,
        &200_000u64,
    );

    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
    client.set_pool_state(&pool_id, &PoolState::Cancelled);

    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
}

/// Test 8: Token-based donations throughout lifecycle
#[test]
fn test_campaign_token_donations_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = 500_000_000u128;
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Token Campaign"),
        &String::from_str(&env, "Token-based donations"),
        &goal,
        &200_000u64,
    );

    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    // Mint tokens to donors, not contract
    let token = create_token(&env, 1_000_000_000i128, &donor1);
    let sac = StellarAssetClient::new(&env, &token);
    sac.mint(&donor2, &500_000_000i128);

    client.donate_with_token(&pool_id, &donor1, &token, &200_000_000i128);
    assert_eq!(client.get_total_raised(&pool_id), 200_000_000u128);

    client.donate_with_token(&pool_id, &donor2, &token, &300_000_000i128);
    assert_eq!(client.get_total_raised(&pool_id), goal);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, goal);
}
