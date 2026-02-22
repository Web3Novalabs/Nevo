#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

use crate::{
    base::errors::CrowdfundingError,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    (client, admin, token_address)
}

#[test]
fn test_renounce_admin_success() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    // Initial state: admin exists and can perform admin actions
    assert!(!client.is_paused());
    client.pause();
    assert!(client.is_paused());
    client.unpause();
    assert!(!client.is_paused());

    // Renounce admin
    client.renounce_admin();

    // Verify admin is removed by trying an admin action
    let result = client.try_pause();
    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));
}

#[test]
fn test_renounce_admin_unauthorized() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);
    let _non_admin = Address::generate(&env);

    // unauthorized call is covered by require_auth in renounce_admin
    // which is tested below via mock_auths
}

#[test]
#[should_panic]
fn test_renounce_admin_requires_admin_auth() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);
    let non_admin = Address::generate(&env);

    // Use specific mock_auths to ensure non_admin is the one calling
    client
        .mock_auths(&[MockAuth {
            address: &non_admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "renounce_admin",
                args: ().into_val(&env),
                sub_invokes: &[],
            },
        }])
        .renounce_admin();
}

#[test]
fn test_renounce_admin_already_renounced() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    client.renounce_admin();

    // Try to renounce again
    let result = client.try_renounce_admin();
    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));
}
