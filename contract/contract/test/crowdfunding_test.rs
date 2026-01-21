#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String,
};

use crate::{
    base::errors::CrowdfundingError,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn create_test_campaign_id(env: &Env, seed: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    BytesN::from_array(env, &bytes)
}

#[test]
fn test_create_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Save the Whales");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline);
}

#[test]
fn test_get_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 2);
    let title = String::from_str(&env, "Build a School");
    let goal = 500_000i128;
    let deadline = env.ledger().timestamp() + 172800;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.id, campaign_id);
    assert_eq!(campaign.title, title);
    assert_eq!(campaign.creator, creator);
    assert_eq!(campaign.goal, goal);
    assert_eq!(campaign.deadline, deadline);
}

#[test]
fn test_get_nonexistent_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let campaign_id = create_test_campaign_id(&env, 99);

    let result = client.try_get_campaign(&campaign_id);

    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignNotFound)));
}

#[test]
fn test_create_campaign_with_empty_title() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 3);
    let title = String::from_str(&env, "");
    let goal = 100_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidTitle)));
}

#[test]
fn test_create_campaign_with_zero_goal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 4);
    let title = String::from_str(&env, "Zero Goal Campaign");
    let goal = 0i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidGoal)));
}

#[test]
fn test_create_campaign_with_negative_goal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 5);
    let title = String::from_str(&env, "Negative Goal Campaign");
    let goal = -100i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidGoal)));
}

#[test]
fn test_create_campaign_with_past_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 6);
    let title = String::from_str(&env, "Past Deadline Campaign");
    let goal = 100_000i128;
    let deadline = 500;

    let result = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDeadline)));
}

#[test]
fn test_create_duplicate_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 7);
    let title = String::from_str(&env, "Duplicate Campaign");
    let goal = 100_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    let result2 = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result2, Err(Ok(CrowdfundingError::CampaignAlreadyExists)));
}

#[test]
fn test_multiple_campaigns() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let campaign_id_1 = create_test_campaign_id(&env, 8);
    let campaign_id_2 = create_test_campaign_id(&env, 9);

    let title1 = String::from_str(&env, "Campaign One");
    let title2 = String::from_str(&env, "Campaign Two");

    let goal1 = 100_000i128;
    let goal2 = 200_000i128;

    let deadline1 = env.ledger().timestamp() + 86400;
    let deadline2 = env.ledger().timestamp() + 172800;

    client.create_campaign(&campaign_id_1, &title1, &creator1, &goal1, &deadline1);
    client.create_campaign(&campaign_id_2, &title2, &creator2, &goal2, &deadline2);

    let campaign1 = client.get_campaign(&campaign_id_1);
    let campaign2 = client.get_campaign(&campaign_id_2);

    assert_eq!(campaign1.title, title1);
    assert_eq!(campaign1.goal, goal1);

    assert_eq!(campaign2.title, title2);
    assert_eq!(campaign2.goal, goal2);
}
