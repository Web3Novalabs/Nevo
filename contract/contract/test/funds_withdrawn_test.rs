#![cfg(test)]

use crate::{
    base::errors::CrowdfundingError,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{
    testutils::{Address as _, Events},
    token, Address, BytesN, Env, IntoVal, String, Symbol,
};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token_address, &1000);
    (client, admin, token_address)
}

#[test]
fn test_funds_withdrawn_event_emitted() {
    let env = Env::default();
    let (client, admin, token_address) = setup(&env);

    // Mint tokens to creator and create a campaign to accumulate platform fees
    let token_client = token::StellarAssetClient::new(&env, &token_address);
    let creator = Address::generate(&env);
    token_client.mint(&creator, &1000);

    let id = BytesN::from_array(&env, &[1; 32]);
    let deadline = env.ledger().timestamp() + 86400;
    client.create_campaign(
        &id,
        &String::from_str(&env, "Test"),
        &creator,
        &10000i128,
        &deadline,
        &token_address,
    );

    let receiver = Address::generate(&env);
    client.withdraw_platform_fees(&receiver, &1000i128);

    // Verify funds_withdrawn event was published
    let events = env.events().all();
    let found = events.iter().any(|(_, topics, _)| {
        if let Ok(sym) = topics.get::<Symbol>(0) {
            sym == Symbol::new(&env, "funds_withdrawn")
        } else {
            false
        }
    });
    assert!(found, "funds_withdrawn event was not emitted");
}

#[test]
fn test_funds_withdrawn_event_not_emitted_on_failure() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let receiver = Address::generate(&env);
    // No fees accumulated — should fail with InsufficientFees
    assert_eq!(
        client.try_withdraw_platform_fees(&receiver, &500),
        Err(Ok(CrowdfundingError::InsufficientFees))
    );

    let events = env.events().all();
    let found = events.iter().any(|(_, topics, _)| {
        if let Ok(sym) = topics.get::<Symbol>(0) {
            sym == Symbol::new(&env, "funds_withdrawn")
        } else {
            false
        }
    });
    assert!(!found, "funds_withdrawn event should not be emitted on failure");
}
