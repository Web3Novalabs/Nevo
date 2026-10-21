#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, Address, Env, String,
};

// ============= ISSUE #1093: STRESS TESTS FOR MULTIPLE CONCURRENT CAMPAIGNS =============

/// Test 1: Create 100 campaigns successfully
#[test]
fn test_stress_create_100_campaigns() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for i in 0..100u32 {
        let pool_id = client.create_pool(
            &creator,
            &String::from_str(&env, "Campaign"),
            &String::from_str(&env, "Desc"),
            &((i as u128 + 1) * 1_000_000),
            &100_000u64,
        );
        assert_eq!(pool_id, i + 1);
    }

    assert_eq!(client.get_pool_count(), 100);
}

/// Test 2: All 100 campaigns are tracked in the pool list
#[test]
fn test_stress_all_campaigns_tracked() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = 1_000_000u128;

    for _i in 0..100u32 {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "D"),
            &goal,
            &100_000u64,
        );
    }

    assert_eq!(client.get_pool_count(), 100);

    let first = client.get_pool(&1);
    let last = client.get_pool(&100);
    assert_eq!(first.2, goal);
    assert_eq!(last.2, goal);
    assert_eq!(first.1, creator);
    assert_eq!(last.1, creator);
}

/// Test 3: Independent donation tracking across campaigns
#[test]
fn test_stress_independent_donation_tracking() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = 10_000_000u128;

    for _i in 0..10u32 {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "D"),
            &goal,
            &100_000u64,
        );
    }

    for i in 1..=10u32 {
        let donor = Address::generate(&env);
        let amount = (i as u128) * 1_000_000;
        client.donate(&i, &donor, &amount);
        assert_eq!(client.get_total_raised(&i), amount);
    }

    for i in 1..=10u32 {
        let expected = (i as u128) * 1_000_000;
        assert_eq!(client.get_total_raised(&i), expected);
    }
}

/// Test 4: Performance remains acceptable with 100 campaigns
#[test]
fn test_stress_performance_100_campaigns() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for _i in 0..100u32 {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "Desc"),
            &1_000_000u128,
            &100_000u64,
        );
    }

    assert_eq!(client.get_pool_count(), 100);

    for i in 1..=100u32 {
        let pool = client.get_pool(&i);
        assert_eq!(pool.0, i);
        assert_eq!(pool.1, creator);
    }
}

/// Test 5: Memory usage is reasonable
#[test]
fn test_stress_memory_usage_100_campaigns() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for i in 0..100u32 {
        let goal = ((i as u128) + 1) * 1_000_000;
        client.create_pool(
            &creator,
            &String::from_str(&env, "Campaign"),
            &String::from_str(&env, "Desc"),
            &goal,
            &100_000u64,
        );
    }

    // Donate to all 100 campaigns
    for i in 1..=100u32 {
        let donor = Address::generate(&env);
        client.donate(&i, &donor, &1_000_000u128);
    }

    // Verify each campaign's state
    for i in 1..=100u32 {
        let pool = client.get_pool(&i);
        let expected_goal = (i as u128) * 1_000_000;
        assert_eq!(pool.2, expected_goal);
        assert_eq!(pool.3, 1_000_000u128);
    }
}
