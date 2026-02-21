#![cfg(test)]

use crate::{
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

#[test]
fn test_get_campaigns_by_creator_multiple_users() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    
    client.initialize(&admin, &token_address, &0);

    // Create campaigns with two different creators
    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let campaign_id1 = BytesN::from_array(&env, &[1u8; 32]);
    let campaign_id2 = BytesN::from_array(&env, &[2u8; 32]);
    let campaign_id3 = BytesN::from_array(&env, &[3u8; 32]);

    let title1 = String::from_str(&env, "Campaign 1");
    let title2 = String::from_str(&env, "Campaign 2");
    let title3 = String::from_str(&env, "Campaign 3");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    // Creator 1 creates 2 campaigns
    client.create_campaign(
        &campaign_id1,
        &title1,
        &creator1,
        &goal,
        &deadline,
        &token_address,
    );

    client.create_campaign(
        &campaign_id2,
        &title2,
        &creator1,
        &goal,
        &deadline,
        &token_address,
    );

    // Creator 2 creates 1 campaign
    client.create_campaign(
        &campaign_id3,
        &title3,
        &creator2,
        &goal,
        &deadline,
        &token_address,
    );

    // Verify creator1 has 2 campaigns
    let campaigns1 = client.get_campaigns_by_creator(&creator1);
    assert_eq!(campaigns1.len(), 2);

    // Verify creator2 has 1 campaign
    let campaigns2 = client.get_campaigns_by_creator(&creator2);
    assert_eq!(campaigns2.len(), 1);
}
