#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, BytesN, Env, String};

use crate::{
    base::errors::CrowdfundingError,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

#[test]
fn test_batch_claim_three_campaigns_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let creator3 = Address::generate(&env);

    let campaign_id1 = BytesN::from_array(&env, &[1u8; 32]);
    let campaign_id2 = BytesN::from_array(&env, &[2u8; 32]);
    let campaign_id3 = BytesN::from_array(&env, &[3u8; 32]);

    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 1000;

    // Create campaigns
    client.create_campaign(
        &campaign_id1,
        &String::from_str(&env, "Campaign 1"),
        &creator1,
        &goal,
        &deadline,
        &token_address,
    );
    client.create_campaign(
        &campaign_id2,
        &String::from_str(&env, "Campaign 2"),
        &creator2,
        &goal,
        &deadline,
        &token_address,
    );
    client.create_campaign(
        &campaign_id3,
        &String::from_str(&env, "Campaign 3"),
        &creator3,
        &goal,
        &deadline,
        &token_address,
    );

    // Fund all campaigns
    let donor = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token_address);
    token_client.mint(&donor, &(goal * 3));

    client.donate(&campaign_id1, &donor, &token_address, &goal);
    client.donate(&campaign_id2, &donor, &token_address, &goal);
    client.donate(&campaign_id3, &donor, &token_address, &goal);

    // Batch claim all 3 campaigns
    let mut campaign_ids = soroban_sdk::Vec::new(&env);
    campaign_ids.push_back(campaign_id1.clone());
    campaign_ids.push_back(campaign_id2.clone());
    campaign_ids.push_back(campaign_id3.clone());

    let results = client.batch_claim_campaign_funds(&campaign_ids);

    // All should succeed
    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap(), Ok(()));
    assert_eq!(results.get(1).unwrap(), Ok(()));
    assert_eq!(results.get(2).unwrap(), Ok(()));

    // Verify creators received funds
    assert_eq!(token_client.balance(&creator1), goal);
    assert_eq!(token_client.balance(&creator2), goal);
    assert_eq!(token_client.balance(&creator3), goal);
}

#[test]
fn test_batch_claim_with_partial_failures() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let creator3 = Address::generate(&env);

    let campaign_id1 = BytesN::from_array(&env, &[1u8; 32]);
    let campaign_id2 = BytesN::from_array(&env, &[2u8; 32]);
    let campaign_id3 = BytesN::from_array(&env, &[3u8; 32]);

    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 1000;

    // Create 3 campaigns
    client.create_campaign(
        &campaign_id1,
        &String::from_str(&env, "Campaign 1"),
        &creator1,
        &goal,
        &deadline,
        &token_address,
    );
    client.create_campaign(
        &campaign_id2,
        &String::from_str(&env, "Campaign 2"),
        &creator2,
        &goal,
        &deadline,
        &token_address,
    );
    client.create_campaign(
        &campaign_id3,
        &String::from_str(&env, "Campaign 3"),
        &creator3,
        &goal,
        &deadline,
        &token_address,
    );

    // Fund only campaigns 1 and 3
    let donor = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token_address);
    token_client.mint(&donor, &(goal * 2));

    client.donate(&campaign_id1, &donor, &token_address, &goal);
    client.donate(&campaign_id3, &donor, &token_address, &goal);

    // Batch claim all 3 campaigns
    let mut campaign_ids = soroban_sdk::Vec::new(&env);
    campaign_ids.push_back(campaign_id1.clone());
    campaign_ids.push_back(campaign_id2.clone());
    campaign_ids.push_back(campaign_id3.clone());

    let results = client.batch_claim_campaign_funds(&campaign_ids);

    // Campaign 1 and 3 succeed, campaign 2 fails
    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap(), Ok(()));
    assert_eq!(
        results.get(1).unwrap(),
        Err(CrowdfundingError::CampaignExpired)
    );
    assert_eq!(results.get(2).unwrap(), Ok(()));

    // Verify only creators 1 and 3 received funds
    assert_eq!(token_client.balance(&creator1), goal);
    assert_eq!(token_client.balance(&creator2), 0);
    assert_eq!(token_client.balance(&creator3), goal);
}

#[test]
fn test_batch_claim_already_claimed() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let creator = Address::generate(&env);
    let campaign_id1 = BytesN::from_array(&env, &[1u8; 32]);
    let campaign_id2 = BytesN::from_array(&env, &[2u8; 32]);
    let campaign_id3 = BytesN::from_array(&env, &[3u8; 32]);

    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 1000;

    // Create and fund 3 campaigns
    for (id, name) in [
        (&campaign_id1, "Campaign 1"),
        (&campaign_id2, "Campaign 2"),
        (&campaign_id3, "Campaign 3"),
    ] {
        client.create_campaign(
            id,
            &String::from_str(&env, name),
            &creator,
            &goal,
            &deadline,
            &token_address,
        );

        let donor = Address::generate(&env);
        let token_client = token::StellarAssetClient::new(&env, &token_address);
        token_client.mint(&donor, &goal);
        client.donate(id, &donor, &token_address, &goal);
    }

    // Claim campaign 2 individually first
    client.claim_campaign_funds(&campaign_id2);

    // Batch claim all 3 campaigns
    let mut campaign_ids = soroban_sdk::Vec::new(&env);
    campaign_ids.push_back(campaign_id1.clone());
    campaign_ids.push_back(campaign_id2.clone());
    campaign_ids.push_back(campaign_id3.clone());

    let results = client.batch_claim_campaign_funds(&campaign_ids);

    // Campaign 1 and 3 succeed, campaign 2 fails (already claimed)
    assert_eq!(results.len(), 3);
    assert_eq!(results.get(0).unwrap(), Ok(()));
    assert_eq!(
        results.get(1).unwrap(),
        Err(CrowdfundingError::CampaignAlreadyFunded)
    );
    assert_eq!(results.get(2).unwrap(), Ok(()));
}

#[test]
fn test_single_claim_campaign_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let creator = Address::generate(&env);
    let campaign_id = BytesN::from_array(&env, &[1u8; 32]);
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 1000;

    client.create_campaign(
        &campaign_id,
        &String::from_str(&env, "Test Campaign"),
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    // Fund the campaign
    let donor = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token_address);
    token_client.mint(&donor, &goal);
    client.donate(&campaign_id, &donor, &token_address, &goal);

    // Claim funds
    client.claim_campaign_funds(&campaign_id);

    // Verify creator received funds
    assert_eq!(token_client.balance(&creator), goal);

    // Verify cannot claim again
    let result = client.try_claim_campaign_funds(&campaign_id);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignAlreadyFunded)));
}

#[test]
fn test_claim_unsuccessful_campaign_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let creator = Address::generate(&env);
    let campaign_id = BytesN::from_array(&env, &[1u8; 32]);
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 1000;

    client.create_campaign(
        &campaign_id,
        &String::from_str(&env, "Test Campaign"),
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    // Don't fund the campaign
    let result = client.try_claim_campaign_funds(&campaign_id);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignExpired)));
}
