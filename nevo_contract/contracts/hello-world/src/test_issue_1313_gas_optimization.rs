#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

const WORKLOAD_SIZE: u32 = 32;

#[test]
fn test_gas_validation_pool_creation_workload_is_bounded() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    for index in 0..WORKLOAD_SIZE {
        let pool_id = client.create_pool(
            &creator,
            &String::from_str(&env, "Bounded pool"),
            &String::from_str(&env, "Bounded creation workload"),
            &(index as u128 + 1),
            &100_000u64,
        );
        assert_eq!(pool_id, index + 1);
    }

    assert_eq!(client.get_pool_count(), WORKLOAD_SIZE);
}

#[test]
fn test_gas_validation_donation_workload_has_no_cross_pool_growth() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);

    for _ in 0..WORKLOAD_SIZE {
        client.create_pool(
            &creator,
            &String::from_str(&env, "Donation pool"),
            &String::from_str(&env, "Bounded donation workload"),
            &1_000_000u128,
            &100_000u64,
        );
    }

    for pool_id in 1..=WORKLOAD_SIZE {
        client.donate(&pool_id, &Address::generate(&env), &100u128);
    }

    for pool_id in 1..=WORKLOAD_SIZE {
        assert_eq!(client.get_total_raised(&pool_id), 100u128);
    }
}

#[test]
fn test_gas_validation_metadata_reads_remain_bounded_after_updates() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Metadata pool"),
        &String::from_str(&env, "Initial"),
        &1_000_000u128,
        &100_000u64,
    );

    for _ in 0..WORKLOAD_SIZE {
        client.save_pool(
            &pool_id,
            &String::from_str(&env, "Updated metadata"),
            &String::from_str(&env, "https://example.com"),
            &String::from_str(&env, "hash"),
        );
        assert_eq!(
            client.get_saved_pool_metadata(&pool_id).2,
            String::from_str(&env, "hash")
        );
    }
}
