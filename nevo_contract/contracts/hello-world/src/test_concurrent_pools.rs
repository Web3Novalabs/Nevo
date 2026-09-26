#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _, Address, Env, String,
};

// ============= ISSUE #1094: STRESS TESTS FOR MULTIPLE CONCURRENT POOLS =============

/// Test 1: Create 100 pools successfully
#[test]
fn test_stress_create_100_pools() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for i in 0..100u32 {
        let pool_id = client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "Desc"),
            &((i as u128 + 1) * 1_000_000u128),
            &100_000u64,
        );
        assert_eq!(pool_id, i + 1);
    }

    assert_eq!(client.get_pool_count(), 100);
}

/// Test 2: Independent state management across pools
#[test]
fn test_stress_independent_state_management() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for _i in 0..20u32 {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "Desc"),
            &1_000_000_000u128,
            &100_000u64,
        );
    }

    let states = [
        PoolState::Active,
        PoolState::Paused,
        PoolState::Completed,
        PoolState::Cancelled,
        PoolState::Disbursed,
    ];

    for i in 1..=20u32 {
        let state = &states[((i - 1) % 5) as usize];
        client.set_pool_state(&i, state);
    }

    // Active pools (1, 6, 11, 16) should accept donations
    let donor = Address::generate(&env);
    client.donate(&1, &donor, &100_000u128);
    assert_eq!(client.get_total_raised(&1), 100_000u128);
}

/// Test 3: Contribution tracking per pool
#[test]
fn test_stress_contribution_tracking_per_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for _i in 0..50u32 {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "Desc"),
            &10_000_000_000u128,
            &100_000u64,
        );
    }

    for i in 1..=50u32 {
        let donor = Address::generate(&env);
        let amount = (i as u128) * 1_000_000u128;
        client.donate(&i, &donor, &amount);

        assert_eq!(client.get_total_raised(&i), amount);
        assert_eq!(client.get_contribution(&i, &donor), amount);
    }

    assert_eq!(client.get_total_raised(&1), 1_000_000u128);
    assert_eq!(client.get_total_raised(&25), 25_000_000u128);
    assert_eq!(client.get_total_raised(&50), 50_000_000u128);
}

/// Test 4: State transitions work for all 100 pools
#[test]
fn test_stress_state_transitions_for_all_pools() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for _i in 0..100u32 {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "Desc"),
            &1_000_000_000u128,
            &100_000u64,
        );
    }

    for i in 1..=100u32 {
        client.set_pool_state(&i, &PoolState::Disbursed);
        client.close_pool(&i);

        let pool = client.get_pool(&i);
        assert_eq!(pool.4, true);
    }

    for i in 1..=100u32 {
        let pool = client.get_pool(&i);
        assert_eq!(pool.4, true);
    }
}

/// Test 5: Resource usage with many pools and donations
#[test]
fn test_stress_resource_usage_many_pools_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    for _i in 0..10u32 {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "Desc"),
            &100_000_000_000u128,
            &100_000u64,
        );
    }

    for i in 1..=10u32 {
        for _j in 0..10u32 {
            let donor = Address::generate(&env);
            client.donate(&i, &donor, &1_000_000u128);
        }
        assert_eq!(client.get_total_raised(&i), 10_000_000u128);
    }

    assert_eq!(client.get_pool_count(), 10);
}
