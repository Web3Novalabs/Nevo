#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn create_pool(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Refund State Pool"),
        &String::from_str(env, "Refund state validation coverage"),
        &1_000_000_000u128,
        &200_000u64,
    )
}

#[test]
fn test_refund_state_setup_records_contribution_before_expiry_checks() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let pool_id = create_pool(&env, &client);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &250_000_000u128);

    assert_eq!(client.get_contribution(&pool_id, &donor), 250_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 250_000_000u128);
}