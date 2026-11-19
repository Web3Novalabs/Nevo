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

/// Test 6 (Issue #1302): a real resource-usage assertion, using the SDK's
/// cost-estimate budget instead of just re-checking correctness under a
/// "resource" label (as Test 5 above and its campaign-stress counterpart do).
///
/// `env.cost_estimate().budget()` resets before every *top-level* contract
/// invocation, so it cannot report a cumulative total across 100 calls —
/// instead this compares the cost of a single `create_pool` / `donate` call
/// made when the contract is empty against the same call made once 99-100
/// other pools already exist. If per-pool operations are correctly isolated
/// by storage key (as they should be), the cost should not grow with the
/// total pool count; a large multiplier here would indicate an accidental
/// O(n) or worse scan somewhere in the write path.
#[test]
fn test_stress_budget_scaling_100_pools() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    // Baseline: cost of creating the very first pool in an empty contract.
    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
        &100_000u64,
    );
    let first_create_cpu = env.cost_estimate().budget().cpu_instruction_cost();
    let first_create_mem = env.cost_estimate().budget().memory_bytes_cost();

    // Baseline: cost of the first donation to that first pool.
    let donor1 = Address::generate(&env);
    client.donate(&1, &donor1, &1_000_000u128);
    let first_donate_cpu = env.cost_estimate().budget().cpu_instruction_cost();

    // Fill up to 99 pools.
    for _ in 0..98u32 {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Pool"),
            &String::from_str(&env, "Desc"),
            &1_000_000_000u128,
            &100_000u64,
        );
    }

    // Cost of creating the 100th pool, with 99 already stored.
    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
        &100_000u64,
    );
    let last_create_cpu = env.cost_estimate().budget().cpu_instruction_cost();
    let last_create_mem = env.cost_estimate().budget().memory_bytes_cost();

    assert_eq!(client.get_pool_count(), 100);

    // Cost of donating to the brand-new 100th pool, with 100 pools total in
    // the contract -- should cost about the same as donating to pool #1,
    // proving per-pool storage operations don't degrade as pool count grows.
    let donor100 = Address::generate(&env);
    client.donate(&100, &donor100, &1_000_000u128);
    let last_donate_cpu = env.cost_estimate().budget().cpu_instruction_cost();

    assert!(first_create_cpu > 0, "budget should report nonzero CPU cost");
    assert!(first_donate_cpu > 0, "budget should report nonzero CPU cost");
    assert!(last_create_cpu > 0, "budget should report nonzero CPU cost");
    assert!(last_donate_cpu > 0, "budget should report nonzero CPU cost");

    assert!(
        last_create_cpu <= first_create_cpu.saturating_mul(10),
        "creating pool #100 cost {} CPU instructions vs {} for pool #1 -- \
         create_pool may not scale O(1) with the existing pool count",
        last_create_cpu,
        first_create_cpu
    );
    assert!(
        last_create_mem <= first_create_mem.saturating_mul(10),
        "creating pool #100 cost {} bytes of memory vs {} for pool #1 -- \
         create_pool memory usage may not scale O(1) with the existing pool count",
        last_create_mem,
        first_create_mem
    );
    assert!(
        last_donate_cpu <= first_donate_cpu.saturating_mul(10),
        "donating to a fresh pool with 100 pools total cost {} CPU instructions \
         vs {} with only 1 pool total -- donate() cost may depend on total pool \
         count instead of being isolated per pool",
        last_donate_cpu,
        first_donate_cpu
    );
}
