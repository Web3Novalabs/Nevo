#![cfg(test)]
//! Regression tests for the pool-existence checks in `get_donor_count` and
//! `get_contribution` — issue #1482.
//!
//! Both getters read the whole `Pool` record only to decide whether the pool
//! exists, which is wasteful: the existence question can be answered with
//! `Storage::has`. These tests lock in the behaviour that must be preserved
//! (a nonexistent pool panics with `ContractError::PoolNotFound` instead of
//! silently reporting zero), show that the existence check and the full read
//! always agree, and measure the relative cost of the two reads so the
//! optimisation is backed by an actual measurement.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// -- Missing-pool behaviour ----------------------------------------------

/// `get_donor_count` must reject an ID that has never been created with
/// `ContractError::PoolNotFound` (encoded as `Error(Contract, #1)`), not
/// report a donor count of zero.
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_get_donor_count_missing_pool_panics_with_pool_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Pool IDs are handed out sequentially starting at 1; nothing was created.
    let _ = client.get_donor_count(&1u32);
}

/// `get_contribution` must reject an ID that has never been created with
/// `ContractError::PoolNotFound` (encoded as `Error(Contract, #1)`), not
/// report a contribution of zero.
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_get_contribution_missing_pool_panics_with_pool_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    // Pool IDs are handed out sequentially starting at 1; nothing was created.
    let _ = client.get_contribution(&1u32, &donor);
}

/// The `try_` form pins the error variant itself, so these tests cannot pass
/// because of an unrelated panic, and they prove neither getter silently
/// returns zero for a nonexistent pool.
#[test]
fn test_missing_pool_getters_return_pool_not_found_instead_of_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);

    assert_eq!(
        client.try_get_donor_count(&1u32),
        Err(Ok(ContractError::PoolNotFound.into())),
        "get_donor_count must fail with PoolNotFound for a pool that was never created"
    );
    assert_eq!(
        client.try_get_contribution(&1u32, &donor),
        Err(Ok(ContractError::PoolNotFound.into())),
        "get_contribution must fail with PoolNotFound for a pool that was never created"
    );
}

/// The existence check must also reject an ID far beyond any pool the contract
/// could have handed out, i.e. the failure is about the key being absent and
/// not about the value happening to be `1`.
#[test]
fn test_missing_pool_getters_reject_far_future_pool_id() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);

    assert_eq!(
        client.try_get_donor_count(&9_999u32),
        Err(Ok(ContractError::PoolNotFound.into()))
    );
    assert_eq!(
        client.try_get_contribution(&9_999u32, &donor),
        Err(Ok(ContractError::PoolNotFound.into()))
    );
}

/// The getters keep working normally for a pool that does exist, so the
/// missing-pool tests above cannot be satisfied by breaking the happy path.
#[test]
fn test_getters_sterve_existing_pool_after_existence_check() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Existence Check Pool"),
        &String::from_str(&env, "Pool that does exist"),
        &10_000_000_000u128,
        &100_000u64,
    );

    // An existing pool with no donations yet reports zero, not an error.
    assert_eq!(client.get_donor_count(&pool_id), 0u32);
    assert_eq!(client.get_contribution(&pool_id, &donor), 0u128);

    client.donate(&pool_id, &donor, &100_000_000u128);

    assert_eq!(client.get_donor_count(&pool_id), 1u32);
    assert_eq!(client.get_contribution(&pool_id, &donor), 100_000_000u128);
}

// -- Resource cost: full Pool read vs. existence check ---------------------

/// Measures the CPU and memory the host metered while reading the pool record,
/// from an `as_contract` frame that performs exactly that read. The host budget
/// accumulates over the lifetime of the `Env`, so the cost of the surrounding
/// frame is measured separately and subtracted. Both figures are deterministic
/// host counters, not wall-clock timings.
fn measure_read(env: &Env, contract_id: &Address, pool_id: u32, full_read: bool) -> (u64, u64) {
    env.as_contract(contract_id, || {});
    let baseline_cpu = env.cost_estimate().budget().cpu_instruction_cost();
    let baseline_mem = env.cost_estimate().budget().memory_bytes_cost();

    env.as_contract(contract_id, || {
        if full_read {
            // The read the getters performed before the optimisation: load and
            // deserialise the whole `Pool` record just to test existence.
            assert!(
                env.storage()
                    .persistent()
                    .get::<_, Pool>(&pool_id)
                    .is_some(),
                "the pool under test must exist"
            );
        } else {
            // The proposed replacement: ask only whether the key is present.
            assert!(
                env.storage().persistent().has(&pool_id),
                "the pool under test must exist"
            );
        }
    });
    let read_cpu = env.cost_estimate().budget().cpu_instruction_cost();
    let read_mem = env.cost_estimate().budget().memory_bytes_cost();

    (
        read_cpu.saturating_sub(baseline_cpu),
        read_mem.saturating_sub(baseline_mem),
    )
}

/// Metering comparison backing the `Storage::has` optimisation: reading and
/// deserialising the full `Pool` record costs strictly more CPU instructions
/// and memory than asking whether the key exists.
///
/// The measurement uses the Soroban SDK cost-estimate budget
/// (`env.cost_estimate().budget()`), the same API the rest of the test suite
/// relies on, so no timing or benchmarking framework is involved.
#[test]
fn test_existence_check_is_cheaper_than_full_pool_read() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metering Pool"),
        &String::from_str(&env, "Pool used for read-cost metering"),
        &10_000_000_000u128,
        &100_000u64,
    );

    let (full_read_cpu, full_read_mem) = measure_read(&env, &contract_id, pool_id, true);
    let (existence_cpu, existence_mem) = measure_read(&env, &contract_id, pool_id, false);

    assert!(
        full_read_cpu > 0,
        "the full read must consume CPU instructions"
    );
    assert!(
        existence_cpu > 0,
        "the existence check must consume CPU instructions"
    );
    assert!(
        existence_cpu < full_read_cpu,
        "existence check cost {} CPU instructions, full Pool read cost {} -- \
         an existence check is not cheaper, so it must not replace the read",
        existence_cpu,
        full_read_cpu
    );
    assert!(
        existence_mem < full_read_mem,
        "existence check cost {} memory bytes, full Pool read cost {} -- \
         an existence check is not cheaper, so it must not replace the read",
        existence_mem,
        full_read_mem
    );
}

/// The two read strategies must agree about which pools exist; the existence
/// check is only a safe replacement for the full read if they always match.
#[test]
fn test_existence_check_agrees_with_full_pool_read() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let existing = client.create_pool(
        &creator,
        &String::from_str(&env, "Existing Pool"),
        &String::from_str(&env, "Pool that does exist"),
        &10_000_000_000u128,
        &100_000u64,
    );

    env.as_contract(&contract_id, || {
        assert!(env
            .storage()
            .persistent()
            .get::<_, Pool>(&existing)
            .is_some());
        assert!(env.storage().persistent().has(&existing));

        let missing = existing + 1;
        assert!(env
            .storage()
            .persistent()
            .get::<_, Pool>(&missing)
            .is_none());
        assert!(!env.storage().persistent().has(&missing));
    });
}
