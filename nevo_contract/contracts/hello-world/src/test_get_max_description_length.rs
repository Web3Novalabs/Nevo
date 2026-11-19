#![cfg(test)]

use super::*;
use soroban_sdk::Env;

#[test]
fn test_get_max_description_length_returns_500() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_max_description_length(), 500u32);
}
