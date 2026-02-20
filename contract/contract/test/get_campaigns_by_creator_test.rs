#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::crowdfunding::{CrowdfundingContract, CrowdfundingContractClient};

fn create_test_campaign_id(env: &Env, seed: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    BytesN::from_array(env, &bytes)
}

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    // Initialize with 0 fee by default
    client.initialize(&admin, &token_address, &0);

    (client, admin, token_address)
}

#[test]
fn test_get_campaigns_by_creator_empty() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaigns = client.get_campaigns_by_creator(&creator);

    assert_eq!(campaigns.len(), 0);
}

#[test]
fn test_get_campaigns_by_creator_single_campaign() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_address);

    let campaigns = client.get_campaigns_by_creator(&creator);

    assert_eq!(campaigns.len(), 1);
    assert_eq!(campaigns.get(0).unwrap(), campaign_id);
}

#[test]
fn test_get_campaigns_by_creator_multiple_campaigns() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaign_id_1 = create_test_campaign_id(&env, 1);
    let campaign_id_2 = create_test_campaign_id(&env, 2);
    let campaign_id_3 = create_test_campaign_id(&env, 3);

    let title1 = String::from_str(&env, "Campaign 1");
    let title2 = String::from_str(&env, "Campaign 2");
    let title3 = String::from_str(&env, "Campaign 3");

    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    // Create three campaigns with the same creator
    client.create_campaign(
        &campaign_id_1,
        &title1,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );
    client.create_campaign(
        &campaign_id_2,
        &title2,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );
    client.create_campaign(
        &campaign_id_3,
        &title3,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    let campaigns = client.get_campaigns_by_creator(&creator);

    assert_eq!(campaigns.len(), 3);
    assert_eq!(campaigns.get(0).unwrap(), campaign_id_1);
    assert_eq!(campaigns.get(1).unwrap(), campaign_id_2);
    assert_eq!(campaigns.get(2).unwrap(), campaign_id_3);
}

#[test]
fn test_get_campaigns_by_creator_different_creators() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let campaign_id_1 = create_test_campaign_id(&env, 1);
    let campaign_id_2 = create_test_campaign_id(&env, 2);
    let campaign_id_3 = create_test_campaign_id(&env, 3);

    let title1 = String::from_str(&env, "Creator1 Campaign 1");
    let title2 = String::from_str(&env, "Creator1 Campaign 2");
    let title3 = String::from_str(&env, "Creator2 Campaign 1");

    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    // Create two campaigns with creator1
    client.create_campaign(
        &campaign_id_1,
        &title1,
        &creator1,
        &goal,
        &deadline,
        &token_address,
    );
    client.create_campaign(
        &campaign_id_2,
        &title2,
        &creator1,
        &goal,
        &deadline,
        &token_address,
    );

    // Create one campaign with creator2
    client.create_campaign(
        &campaign_id_3,
        &title3,
        &creator2,
        &goal,
        &deadline,
        &token_address,
    );

    // Verify creator1 has 2 campaigns
    let creator1_campaigns = client.get_campaigns_by_creator(&creator1);
    assert_eq!(creator1_campaigns.len(), 2);
    assert_eq!(creator1_campaigns.get(0).unwrap(), campaign_id_1);
    assert_eq!(creator1_campaigns.get(1).unwrap(), campaign_id_2);

    // Verify creator2 has 1 campaign
    let creator2_campaigns = client.get_campaigns_by_creator(&creator2);
    assert_eq!(creator2_campaigns.len(), 1);
    assert_eq!(creator2_campaigns.get(0).unwrap(), campaign_id_3);

    // Verify total campaigns is 3
    let all_campaigns = client.get_all_campaigns();
    assert_eq!(all_campaigns.len(), 3);
}

#[test]
fn test_get_campaigns_by_creator_isolation() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let creator3 = Address::generate(&env);

    let campaign_id_1 = create_test_campaign_id(&env, 1);

    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    // Create one campaign with creator1
    client.create_campaign(
        &campaign_id_1,
        &title,
        &creator1,
        &goal,
        &deadline,
        &token_address,
    );

    // Verify creator1 has 1 campaign
    let creator1_campaigns = client.get_campaigns_by_creator(&creator1);
    assert_eq!(creator1_campaigns.len(), 1);

    // Verify creator2 has 0 campaigns
    let creator2_campaigns = client.get_campaigns_by_creator(&creator2);
    assert_eq!(creator2_campaigns.len(), 0);

    // Verify creator3 has 0 campaigns
    let creator3_campaigns = client.get_campaigns_by_creator(&creator3);
    assert_eq!(creator3_campaigns.len(), 0);
}
