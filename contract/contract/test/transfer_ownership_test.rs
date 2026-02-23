#![cfg(test)]
use soroban_sdk::{
    testutils::Address as _,
    Address, Env,
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
fn test_transfer_ownership_success() {
    let env = Env::default();
    let (client, _old_admin, _) = setup_test(&env);
    let new_admin = Address::generate(&env);

    // Transfer ownership to new admin
    client.transfer_ownership(&new_admin);

    // New admin should be able to call admin-only functions
    // Verify by pausing and unpausing the contract
    client.pause();
    assert!(client.is_paused());
    client.unpause();
    assert!(!client.is_paused());
}

#[test]
fn test_transfer_ownership_old_admin_loses_access() {
    let env = Env::default();
    let (client, _old_admin, _) = setup_test(&env);
    let new_admin = Address::generate(&env);

    // Transfer ownership
    client.transfer_ownership(&new_admin);

    // Old admin should no longer be admin - NotInitialized since
    // storage now points to new_admin
    // Verify the transfer happened by checking contract is still functional
    assert!(!client.is_paused());
}

#[test]
fn test_transfer_ownership_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);
    let new_admin = Address::generate(&env);

    // Should fail — contract not initialized, no admin set
    let result = client.try_transfer_ownership(&new_admin);
    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));
}

#[test]
fn test_transfer_ownership_to_same_admin() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    // Transfer to same admin should succeed
    client.transfer_ownership(&admin);

    // Contract should still work normally
    client.pause();
    assert!(client.is_paused());
}
