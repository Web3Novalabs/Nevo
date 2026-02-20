#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal};

use crate::crowdfunding::{CrowdfundingContract, CrowdfundingContractClient};
use crate::interfaces::crowdfunding::CrowdfundingTrait;

fn create_client() -> (Env, CrowdfundingContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, CrowdfundingContract);
    let client = CrowdfundingContractClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_verify_cause_success() {
    let (env, client) = create_client();

    let admin = Address::generate(&env);
    let cause = Address::generate(&env);
    let token = Address::generate(&env);
    let creation_fee = 1000;

    client.initialize(&admin, &token, &creation_fee);

    // Verify the cause
    client.verify_cause(&cause);

    // Check if it is verified
    assert_eq!(client.is_cause_verified(&cause), true);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_verify_cause_unauthorized() {
    let (env, client) = create_client();

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let cause = Address::generate(&env);
    let token = Address::generate(&env);
    let creation_fee = 1000;

    client.initialize(&admin, &token, &creation_fee);

    // Unmock auth to test failing auth
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &non_admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &client.address,
            fn_name: "verify_cause",
            args: (&cause,).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    // This should panic because non_admin is authenticating, not admin, but `verify_cause` requires admin auth.
    client.verify_cause(&cause);
}

#[test]
#[should_panic(expected = "Error(Contract, #28)")]
fn test_verify_cause_not_initialized() {
    let (env, client) = create_client();

    let cause = Address::generate(&env);

    // Should panic with NotInitialized
    client.verify_cause(&cause);
}
