#![cfg(test)]

use crate::crowdfunding::{CrowdfundingContract, CrowdfundingContractClient};
use soroban_sdk::token;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, IntoVal, String};

fn create_client() -> (Env, CrowdfundingContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_withdraw_platform_fees_success() {
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
    token_client.mint(&creator, &2000);

    let id = BytesN::from_array(&env, &[1; 32]);
    let title = String::from_str(&env, "Campaign 1");
    let goal = 10000;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&id, &title, &creator, &goal, &deadline, &token_address);

    // Now platform fees should be 1000.
    // Withdraw 500 to a specific address.
    let receiver = Address::generate(&env);
    client.withdraw_platform_fees(&receiver, &500);

    assert_eq!(standard_token_client.balance(&receiver), 500);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
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

    // Unmock auths to test failure. We expect an Auth error.
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &non_admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &client.address,
            fn_name: "withdraw_platform_fees",
            args: (&receiver, &500i128).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.withdraw_platform_fees(&receiver, &500);
}
