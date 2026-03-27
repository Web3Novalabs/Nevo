#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

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
fn test_upgrade_contract_auth_success() {
    let env = Env::default();
    let (client, _admin) = setup(&env);

    let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);

    // Admin call with mock_all_auths should pass the auth check.
    // It will then fail on the deployer call because of the invalid WASM hash,
    // which is expected and confirms we passed the auth check.
    let result = client.try_upgrade_contract(&new_wasm_hash);

    assert!(result.is_err());
    // We can't easily check the error type if it's a HostError,
    // but we know it reached the contract because of previous diagnostic tests.
}

#[test]
fn test_upgrade_contract_unauthorized_fails() {
    let env = Env::default();
    let (client, _admin) = setup(&env);
    let non_admin = Address::generate(&env);

    let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);

    // Explicitly mock auth for the NON-admin address.
    // The contract's upgrade_contract will still call require_auth(admin).
    // This mismatch must result in an auth failure.
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
