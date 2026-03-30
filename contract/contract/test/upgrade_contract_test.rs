#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

use crate::base::errors::CrowdfundingError;
use crate::crowdfunding::{CrowdfundingContract, CrowdfundingContractClient};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token = Address::generate(env);

    client.initialize(&admin, &token, &0);
    (client, admin)
}

#[test]
fn test_upgrade_contract_not_initialized_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
    let result = client.try_upgrade_contract(&new_wasm_hash);

    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));
}

#[test]
fn test_upgrade_contract_unauthorized_fails() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let non_admin = Address::generate(&env);

    let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);

    let result = client
        .mock_auths(&[MockAuth {
            address: &non_admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "upgrade_contract",
                args: (new_wasm_hash.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_upgrade_contract(&new_wasm_hash);

    assert!(result.is_err(), "Unauthorized call should fail");
}
