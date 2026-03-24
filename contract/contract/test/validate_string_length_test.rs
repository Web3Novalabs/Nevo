#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::{
    base::errors::{CrowdfundingError, SecondCrowdfundingError},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
    interfaces::second_crowdfunding::SecondCrowdfundingTrait,
};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrowdfundingContract);
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token = Address::generate(env);
    client.initialize(&admin, &token, &0);

    (client, token)
}

fn string_of_len(env: &Env, len: usize) -> String {
    String::from_str(env, &"a".repeat(len))
}

// ── create_campaign ────────────────────────────────────────────────────────────

#[test]
fn test_campaign_title_within_limit_succeeds() {
    let env = Env::default();
    let (client, token) = setup(&env);

    let creator = Address::generate(&env);
    let campaign_id = BytesN::from_array(&env, &[1u8; 32]);
    let title = string_of_len(&env, 200); // exactly at the limit
    let deadline = env.ledger().timestamp() + 86_400;

    // Via Soroban client: string validation maps to CrowdfundingError::InvalidTitle
    let result =
        client.try_create_campaign(&campaign_id, &title, &creator, &1000, &deadline, &token);
    assert!(result.is_ok(), "title of 200 chars should be accepted");

    // Via SecondCrowdfundingTrait directly: validates string length only
    let trait_result = <CrowdfundingContract as SecondCrowdfundingTrait>::create_campaign_checked(
        env.clone(),
        campaign_id,
        title,
        creator,
        1000,
        deadline,
        token,
    );
    assert!(
        trait_result.is_ok(),
        "SecondCrowdfundingTrait: title of 200 chars should be accepted"
    );
}

#[test]
fn test_campaign_title_exceeds_limit_returns_error() {
    let env = Env::default();
    let (client, token) = setup(&env);

    let creator = Address::generate(&env);
    let campaign_id = BytesN::from_array(&env, &[2u8; 32]);
    let title = string_of_len(&env, 201); // one over the limit
    let deadline = env.ledger().timestamp() + 86_400;

    // Via Soroban client: SecondCrowdfundingError is mapped to CrowdfundingError::InvalidTitle
    let client_result =
        client.try_create_campaign(&campaign_id, &title, &creator, &1000, &deadline, &token);
    assert_eq!(
        client_result,
        Err(Ok(CrowdfundingError::InvalidTitle)),
        "title of 201 chars should return CrowdfundingError::InvalidTitle via client"
    );

    // Via SecondCrowdfundingTrait directly: returns StringTooLong without remapping
    let trait_result = <CrowdfundingContract as SecondCrowdfundingTrait>::create_campaign_checked(
        env.clone(),
        campaign_id,
        title,
        creator,
        1000,
        deadline,
        token,
    );
    assert_eq!(
        trait_result,
        Err(SecondCrowdfundingError::StringTooLong),
        "title of 201 chars should return StringTooLong via SecondCrowdfundingTrait"
    );
}

#[test]
fn test_campaign_title_much_longer_than_limit_returns_error() {
    let env = Env::default();
    let (client, token) = setup(&env);

    let creator = Address::generate(&env);
    let campaign_id = BytesN::from_array(&env, &[3u8; 32]);
    let title = string_of_len(&env, 500); // well over the limit
    let deadline = env.ledger().timestamp() + 86_400;

    // Via Soroban client: SecondCrowdfundingError is mapped to CrowdfundingError::InvalidTitle
    let client_result =
        client.try_create_campaign(&campaign_id, &title, &creator, &1000, &deadline, &token);
    assert_eq!(
        client_result,
        Err(Ok(CrowdfundingError::InvalidTitle)),
        "title of 500 chars should return CrowdfundingError::InvalidTitle via client"
    );

    // Via SecondCrowdfundingTrait directly: returns StringTooLong without remapping
    let trait_result = <CrowdfundingContract as SecondCrowdfundingTrait>::create_campaign_checked(
        env.clone(),
        campaign_id,
        title,
        creator,
        1000,
        deadline,
        token,
    );
    assert_eq!(
        trait_result,
        Err(SecondCrowdfundingError::StringTooLong),
        "title of 500 chars should return StringTooLong via SecondCrowdfundingTrait"
    );
}
