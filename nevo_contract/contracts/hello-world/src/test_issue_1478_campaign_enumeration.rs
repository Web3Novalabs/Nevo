#![cfg(test)]

//! Issue #1478: `get_pool_count` / `get_all_campaigns` boundaries and O(n) cost.
//!
//! Today `get_pool_count` is the highest allocated pool id (contiguous, no
//! delete path). `get_all_campaigns` walks `1..=count` and keeps ids that
//! exist in persistent storage. These tests lock that behavior and record a
//! reproducible host-budget baseline for a 500-entry enumeration.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> ContractClient<'_> {
    let contract_id = env.register(Contract, ());
    ContractClient::new(env, &contract_id)
}

fn create_named_pool(env: &Env, client: &ContractClient<'_>, title: &str) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, title),
        &String::from_str(env, "Enumeration test"),
        &1_000_000u128,
        &100_000u64,
    )
}

/// Zero boundary: no pools, empty enumeration, no false positives.
#[test]
fn test_pool_count_zero_enumeration_is_empty() {
    let env = Env::default();
    let client = setup(&env);

    assert_eq!(client.get_pool_count(), 0u32);
    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 0);
}

/// One boundary: exactly one campaign is counted and listed.
#[test]
fn test_pool_count_one_enumeration_returns_only_that_pool() {
    let env = Env::default();
    let client = setup(&env);

    let pool_id = create_named_pool(&env, &client, "Only Pool");

    assert_eq!(client.get_pool_count(), 1u32);
    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 1);
    assert_eq!(campaigns.get(0).unwrap(), pool_id);
    assert_eq!(pool_id, 1u32);
}

/// Large-count correctness plus host-budget baseline.
///
/// A 500-entry `get_all_campaigns` exceeds the current Soroban test
/// transaction footprint (`total footprint ledger entries: 502 > 400`).
/// This test uses 350 entries, the largest representative count that stays
/// under that host limit. A 500-entry run remains a follow-up once
/// pagination or a higher footprint is available.
///
/// Environment: Soroban SDK test `Env` (native, not wasm),
/// `env.cost_estimate().budget()` CPU/memory counters after the
/// `get_all_campaigns` invocation. This is a reproducible baseline, not a
/// hard performance guarantee. Exact instruction counts are not asserted.
#[test]
fn test_get_all_campaigns_large_count_cost_baseline() {
    let env = Env::default();
    let client = setup(&env);
    let n = 350u32;

    for i in 0..n {
        create_named_pool(&env, &client, &format!("Pool {i}"));
    }

    assert_eq!(client.get_pool_count(), n);

    let campaigns = client.get_all_campaigns();
    let cpu = env.cost_estimate().budget().cpu_instruction_cost();
    let mem = env.cost_estimate().budget().memory_bytes_cost();

    assert_eq!(campaigns.len(), n);
    for i in 1..=n {
        assert_eq!(
            campaigns.get(i - 1).unwrap(),
            i,
            "enumeration must be contiguous ids 1..=count with no gaps today"
        );
    }

    assert!(
        cpu > 0 && mem > 0,
        "get_all_campaigns must consume metered CPU and memory (O(n) storage reads)"
    );
    eprintln!(
        "get_all_campaigns baseline n={n}: cpu_instructions={cpu} memory_bytes={mem} (Soroban test Env)"
    );
}
