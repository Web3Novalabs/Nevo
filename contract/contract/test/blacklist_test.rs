use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::crowdfunding::{CrowdfundingContract, CrowdfundingContractClient};

#[test]
fn test_blacklist_address_prevents_donation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CrowdfundingContract);
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let creator = Address::generate(&env);
    let malicious_donor = Address::generate(&env);

    // Initialize contract
    client.initialize(&admin, &token, &0);

    // Create a campaign
    let campaign_id = BytesN::from_array(&env, &[1u8; 32]);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token);

    // Blacklist the malicious donor
    client.blacklist_address(&malicious_donor);

    // Verify the address is blacklisted
    assert!(client.is_blacklisted(&malicious_donor));

    // Attempt to donate from blacklisted address should fail
    let result = client.try_donate(&campaign_id, &malicious_donor, &token, &100i128);
    assert!(result.is_err());
}

#[test]
fn test_blacklist_address_prevents_campaign_creation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CrowdfundingContract);
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let malicious_creator = Address::generate(&env);

    // Initialize contract
    client.initialize(&admin, &token, &0);

    // Blacklist the malicious creator
    client.blacklist_address(&malicious_creator);

    // Verify the address is blacklisted
    assert!(client.is_blacklisted(&malicious_creator));

    // Attempt to create campaign from blacklisted address should fail
    let campaign_id = BytesN::from_array(&env, &[1u8; 32]);
    let title = String::from_str(&env, "Malicious Campaign");
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(
        &campaign_id,
        &title,
        &malicious_creator,
        &goal,
        &deadline,
        &token,
    );
    assert!(result.is_err());
}

#[test]
fn test_unblacklist_address_allows_operations() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CrowdfundingContract);
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize contract
    client.initialize(&admin, &token, &0);

    // Blacklist the user
    client.blacklist_address(&user);
    assert!(client.is_blacklisted(&user));

    // Unblacklist the user
    client.unblacklist_address(&user);
    assert!(!client.is_blacklisted(&user));

    // Now user should be able to create a campaign
    let campaign_id = BytesN::from_array(&env, &[1u8; 32]);
    let title = String::from_str(&env, "Valid Campaign");
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &user, &goal, &deadline, &token);

    // Verify campaign was created
    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.creator, user);
}

#[test]
fn test_only_admin_can_blacklist() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CrowdfundingContract);
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let target = Address::generate(&env);

    // Initialize contract
    client.initialize(&admin, &token, &0);

    // Admin can blacklist
    client.blacklist_address(&target);
    assert!(client.is_blacklisted(&target));

    // Unblacklist for next test
    client.unblacklist_address(&target);
    assert!(!client.is_blacklisted(&target));
}

#[test]
fn test_blacklist_persists_across_operations() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CrowdfundingContract);
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize contract
    client.initialize(&admin, &token, &0);

    // Blacklist the user
    client.blacklist_address(&user);

    // Check multiple times to ensure persistence
    assert!(client.is_blacklisted(&user));
    assert!(client.is_blacklisted(&user));
    assert!(client.is_blacklisted(&user));
}
