#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

use crate::{
    base::errors::CrowdfundingError,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

// Import the compiled WASM of this same contract to use as the "new" version
// in the upgrade integration test.
mod upgraded_contract {
    soroban_sdk::contractimport!(file = "../target/wasm32v1-none/release/hello_world.wasm");
}

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token = Address::generate(env);

    client.initialize(&admin, &token, &0);
    (client, admin)
}

/// Integration test: proves the full upgrade path works end-to-end.
/// Uploads a real WASM binary, calls upgrade_contract, and verifies the
/// contract remains functional (storage intact) after the upgrade.
#[test]
fn test_upgrade_contract_succeeds_with_valid_wasm() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    client.initialize(&admin, &token, &0);

    // Upload the contract's own compiled WASM — gives a valid on-ledger hash.
    let new_wasm_hash: BytesN<32> = env.deployer().upload_contract_wasm(upgraded_contract::WASM);

    // Upgrade must succeed: admin is authorized and WASM hash is valid.
    client.upgrade_contract(&new_wasm_hash);

    // Contract is still callable after upgrade — storage is preserved.
    let result = client.try_get_pool_remaining_time(&999u64);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_upgrade_contract_not_initialized_fails() {
    let env = Env::default();
    env.mock_all_auths();

    // Register without calling initialize — no Admin in storage.
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
