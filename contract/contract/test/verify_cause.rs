#![cfg(test)]

use soroban_sdk::{
    symbol_short, testutils::Address as _, testutils::Events, Address, Env, IntoVal,
};

use crate::crowdfunding::{CrowdfundingContract, CrowdfundingContractClient};

fn create_client() -> (Env, CrowdfundingContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);
    (env, client)
}

fn setup(env: &Env, client: &CrowdfundingContractClient) -> (Address, Address) {
    let admin = Address::generate(env);
    let token = Address::generate(env);
    client.initialize(&admin, &token, &1000);
    (admin, token)
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
    assert!(client.is_cause_verified(&cause));
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

#[test]
fn test_verify_cause_emits_school_registered_event() {
    let (env, client) = create_client();
    let (_, _) = setup(&env, &client);
    let cause = Address::generate(&env);

    client.verify_cause(&cause);

    let events = env.events().all();
    let found = events.iter().any(|(_, topics, data)| {
        if topics.is_empty() {
            return false;
        }
        use soroban_sdk::FromVal;
        let sym = soroban_sdk::Symbol::from_val(&env, &topics.get(0).unwrap());
        if sym != symbol_short!("SchReg") {
            return false;
        }
        let emitted_cause = Address::from_val(&env, &topics.get(1).unwrap());
        let _ = data;
        emitted_cause == cause
    });
    assert!(found, "SchReg event not emitted by verify_cause");
}

#[test]
fn test_reject_cause_removes_verification_and_emits_school_revoked_event() {
    let (env, client) = create_client();
    let (_, _) = setup(&env, &client);
    let cause = Address::generate(&env);

    client.verify_cause(&cause);
    assert!(client.is_cause_verified(&cause));

    client.reject_cause(&cause);
    assert!(!client.is_cause_verified(&cause));

    let events = env.events().all();
    let found = events.iter().any(|(_, topics, data)| {
        if topics.is_empty() {
            return false;
        }
        use soroban_sdk::FromVal;
        let sym = soroban_sdk::Symbol::from_val(&env, &topics.get(0).unwrap());
        if sym != symbol_short!("SchRev") {
            return false;
        }
        let emitted_cause = Address::from_val(&env, &topics.get(1).unwrap());
        let _ = data;
        emitted_cause == cause
    });
    assert!(found, "SchRev event not emitted by reject_cause");
}

#[test]
fn test_reject_cause_on_unverified_address_emits_event() {
    let (env, client) = create_client();
    let (_, _) = setup(&env, &client);
    let cause = Address::generate(&env);

    // reject without prior verify — should still fire the event
    client.reject_cause(&cause);
    assert!(!client.is_cause_verified(&cause));

    let events = env.events().all();
    let found = events.iter().any(|(_, topics, _)| {
        if topics.is_empty() {
            return false;
        }
        use soroban_sdk::FromVal;
        let sym = soroban_sdk::Symbol::from_val(&env, &topics.get(0).unwrap());
        sym == symbol_short!("SchRev")
    });
    assert!(found, "SchRev event not emitted on unverified reject");
}
