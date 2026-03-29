#![cfg(test)]

use crate::{
    base::errors::CrowdfundingError,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::token;
use soroban_sdk::{
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal, String, Symbol, TryFromVal,
};

fn create_client() -> (Env, CrowdfundingContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_withdraw_platform_fees_end_to_end() {
    let (env, client) = create_client();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);
    let standard_token_client = token::Client::new(&env, &token_address);

    let creation_fee = 1000;
    client.initialize(&admin, &token_address, &creation_fee);

    let creator = Address::generate(&env);
    token_client.mint(&creator, &5000);

    let id_1 = BytesN::from_array(&env, &[1; 32]);
    let id_2 = BytesN::from_array(&env, &[2; 32]);
    let title_1 = String::from_str(&env, "Campaign 1");
    let title_2 = String::from_str(&env, "Campaign 2");
    let goal = 10000;
    let deadline = env.ledger().timestamp() + 86400;

    // Two campaign creations charge creation fees into the platform fee pool.
    client.create_campaign(&id_1, &title_1, &creator, &goal, &deadline, &token_address);
    client.create_campaign(&id_2, &title_2, &creator, &goal, &deadline, &token_address);

    assert_eq!(standard_token_client.balance(&client.address), 2000);

    // Withdraw part of fees.
    let receiver = Address::generate(&env);
    client.withdraw_platform_fees(&receiver, &750);

    assert_eq!(standard_token_client.balance(&receiver), 750);
    assert_eq!(standard_token_client.balance(&client.address), 1250);

    // Withdraw the remaining amount.
    client.withdraw_platform_fees(&receiver, &1250);

    assert_eq!(standard_token_client.balance(&receiver), 2000);
    assert_eq!(standard_token_client.balance(&client.address), 0);

    // Nothing left to withdraw.
    assert_eq!(
        client.try_withdraw_platform_fees(&receiver, &1),
        Err(Ok(CrowdfundingError::InsufficientFees))
    );
}

#[test]
fn test_withdraw_platform_fees_invalid_amount() {
    let (env, client) = create_client();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token_address, &0);

    let receiver = Address::generate(&env);

    assert_eq!(
        client.try_withdraw_platform_fees(&receiver, &0),
        Err(Ok(CrowdfundingError::InvalidAmount))
    );
    assert_eq!(
        client.try_withdraw_platform_fees(&receiver, &-10),
        Err(Ok(CrowdfundingError::InvalidAmount))
    );
}

#[test]
fn test_withdraw_platform_fees_insufficient_fees() {
    let (env, client) = create_client();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    client.initialize(&admin, &token_address, &1000);

    let creator = Address::generate(&env);
    token_admin_client.mint(&creator, &1000);

    let id = BytesN::from_array(&env, &[9; 32]);
    let title = String::from_str(&env, "Campaign");
    let goal = 1000;
    let deadline = env.ledger().timestamp() + 86400;
    client.create_campaign(&id, &title, &creator, &goal, &deadline, &token_address);

    let receiver = Address::generate(&env);
    assert_eq!(
        client.try_withdraw_platform_fees(&receiver, &1001),
        Err(Ok(CrowdfundingError::InsufficientFees))
    );
}

#[test]
fn test_withdraw_platform_fees_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let receiver = Address::generate(&env);
    assert_eq!(
        client.try_withdraw_platform_fees(&receiver, &100),
        Err(Ok(CrowdfundingError::NotInitialized))
    );
}

#[test]
fn test_withdraw_platform_fees_unauthorized() {
    let (env, client) = create_client();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let client_token = token::StellarAssetClient::new(&env, &token_address);
    let creation_fee = 1000;

    client.initialize(&admin, &token_address, &creation_fee);

    let creator = Address::generate(&env);
    client_token.mint(&creator, &2000);

    let id = BytesN::from_array(&env, &[1; 32]);
    let title = String::from_str(&env, "Campaign 1");
    let goal = 10000;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&id, &title, &creator, &goal, &deadline, &token_address);

    let receiver = Address::generate(&env);
    let non_admin = Address::generate(&env);

    // Mock a non-admin signer; auth check should fail.
    let _ = client
        .mock_auths(&[MockAuth {
            address: &non_admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "withdraw_platform_fees",
                args: (receiver.clone(), 500i128).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_withdraw_platform_fees(&receiver, &500)
        .unwrap_err();
}

#[test]
fn test_withdraw_platform_fees_emits_event() {
    let (env, client) = create_client();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = token::StellarAssetClient::new(&env, &token_address);

    client.initialize(&admin, &token_address, &1000);

    let creator = Address::generate(&env);
    token_client.mint(&creator, &1000);

    let id = BytesN::from_array(&env, &[1; 32]);
    let title = String::from_str(&env, "Campaign");
    let goal = 10000;
    let deadline = env.ledger().timestamp() + 86400;
    client.create_campaign(&id, &title, &creator, &goal, &deadline, &token_address);

    let receiver = Address::generate(&env);
    let amount: i128 = 500;
    client.withdraw_platform_fees(&receiver, &amount);

    let all_events = env.events().all();
    let event = all_events.iter().find(|e| {
        let topics = &e.1;
        if topics.len() < 2 {
            return false;
        }
        let name = Symbol::try_from_val(&env, &topics.get(0).unwrap());
        let to = Address::try_from_val(&env, &topics.get(1).unwrap());
        name == Ok(Symbol::new(&env, "platform_fees_withdrawn")) && to == Ok(receiver.clone())
    });

    assert!(event.is_some(), "platform_fees_withdrawn event not emitted");

    let data = &event.unwrap().2;
    let decoded: Result<i128, _> = TryFromVal::try_from_val(&env, data);
    assert_eq!(
        decoded,
        Ok(amount),
        "event data should contain withdrawn amount"
    );
}
