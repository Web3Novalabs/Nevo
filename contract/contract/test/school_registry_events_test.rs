#![cfg(test)]

use soroban_sdk::{
    symbol_short, testutils::{Address as _, Events}, token, Address, Env, IntoVal,
};

use crate::{
    base::errors::CrowdfundingError,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::StellarAssetClient<'a> {
    let token_address = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    token::StellarAssetClient::new(env, &token_address)
}

fn setup() -> (Env, CrowdfundingContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_client = create_token_contract(&env, &admin);
    let token = token_client.address.clone();

    client.initialize(&admin, &token, &1000);

    (env, client, admin, token)
}

#[test]
fn test_verify_cause_emits_school_registered_event() {
    let (env, client, admin, _token) = setup();
    let school = Address::generate(&env);

    // Verify the school
    client.verify_cause(&school);

    // Check that the school is verified
    assert!(client.is_cause_verified(&school));

    // Check that the correct event was emitted
    let events = env.events().all();
    let school_reg_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchReg")
            && event.topics.get(1).unwrap() == &school.into_val(&env)
    });

    assert!(school_reg_event.is_some());
    
    // Verify event data contains admin address
    let event = school_reg_event.unwrap();
    assert_eq!(event.data, admin.into_val(&env));
}

#[test]
fn test_reject_cause_emits_school_revoked_event() {
    let (env, client, admin, _token) = setup();
    let school = Address::generate(&env);

    // First verify the school
    client.verify_cause(&school);
    assert!(client.is_cause_verified(&school));

    // Then reject/revoke the school
    client.reject_cause(&school);

    // Check that the school is no longer verified
    assert!(!client.is_cause_verified(&school));

    // Check that the correct events were emitted
    let events = env.events().all();
    
    // Should have both SchReg and SchRev events
    let school_reg_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchReg")
    });
    let school_rev_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchRev")
            && event.topics.get(1).unwrap() == &school.into_val(&env)
    });

    assert!(school_reg_event.is_some());
    assert!(school_rev_event.is_some());
    
    // Verify revocation event data contains admin address
    let rev_event = school_rev_event.unwrap();
    assert_eq!(rev_event.data, admin.into_val(&env));
}

#[test]
fn test_verify_cause_unauthorized() {
    let (env, client, _admin, _token) = setup();
    let unauthorized = Address::generate(&env);
    let school = Address::generate(&env);

    env.mock_all_auths_allowing_non_root_auth();

    // Try to verify school as unauthorized user
    let result = client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &unauthorized,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &client.address,
                fn_name: "verify_cause",
                args: (school.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_verify_cause(&school);

    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
    
    // School should not be verified
    assert!(!client.is_cause_verified(&school));
    
    // No events should be emitted
    let events = env.events().all();
    let school_reg_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchReg")
    });
    assert!(school_reg_event.is_none());
}

#[test]
fn test_reject_cause_unauthorized() {
    let (env, client, admin, _token) = setup();
    let unauthorized = Address::generate(&env);
    let school = Address::generate(&env);

    // First verify the school as admin
    client.verify_cause(&school);
    assert!(client.is_cause_verified(&school));

    env.mock_all_auths_allowing_non_root_auth();

    // Try to reject school as unauthorized user
    let result = client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &unauthorized,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &client.address,
                fn_name: "reject_cause",
                args: (school.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_reject_cause(&school);

    assert_eq!(result, Err(Ok(CrowdfundingError::Unauthorized)));
    
    // School should still be verified
    assert!(client.is_cause_verified(&school));
    
    // Should only have SchReg event, no SchRev event
    let events = env.events().all();
    let school_rev_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchRev")
    });
    assert!(school_rev_event.is_none());
}

#[test]
fn test_reject_cause_not_verified_school() {
    let (env, client, _admin, _token) = setup();
    let school = Address::generate(&env);

    // Try to reject a school that was never verified
    let result = client.try_reject_cause(&school);
    
    // Should succeed (idempotent operation)
    assert_eq!(result, Ok(Ok(())));
    
    // School should remain unverified
    assert!(!client.is_cause_verified(&school));
    
    // Should emit SchRev event even for unverified school (for audit trail)
    let events = env.events().all();
    let school_rev_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchRev")
            && event.topics.get(1).unwrap() == &school.into_val(&env)
    });
    assert!(school_rev_event.is_some());
}

#[test]
fn test_multiple_schools_registry_events() {
    let (env, client, admin, _token) = setup();
    let school1 = Address::generate(&env);
    let school2 = Address::generate(&env);
    let school3 = Address::generate(&env);

    // Verify multiple schools
    client.verify_cause(&school1);
    client.verify_cause(&school2);
    client.verify_cause(&school3);

    // Reject one school
    client.reject_cause(&school2);

    // Check verification status
    assert!(client.is_cause_verified(&school1));
    assert!(!client.is_cause_verified(&school2));
    assert!(client.is_cause_verified(&school3));

    // Check events
    let events = env.events().all();
    
    // Should have 3 SchReg events and 1 SchRev event
    let school_reg_events: Vec<_> = events.iter().filter(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchReg")
    }).collect();
    let school_rev_events: Vec<_> = events.iter().filter(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchRev")
    }).collect();

    assert_eq!(school_reg_events.len(), 3);
    assert_eq!(school_rev_events.len(), 1);
    
    // Verify the revoked school event is for school2
    let rev_event = school_rev_events[0];
    assert_eq!(rev_event.topics.get(1).unwrap(), &school2.into_val(&env));
}

#[test]
fn test_school_registry_event_topics_format() {
    let (env, client, admin, _token) = setup();
    let school = Address::generate(&env);

    // Verify school
    client.verify_cause(&school);

    // Check event format matches specification
    let events = env.events().all();
    let school_reg_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchReg")
    }).unwrap();

    // Event should have format: (symbol_short!("SchReg"), school_addr)
    assert_eq!(school_reg_event.topics.len(), 2);
    assert_eq!(school_reg_event.topics.get(0).unwrap(), &symbol_short!("SchReg"));
    assert_eq!(school_reg_event.topics.get(1).unwrap(), &school.into_val(&env));
    assert_eq!(school_reg_event.data, admin.into_val(&env));
}

#[test]
fn test_school_revocation_event_topics_format() {
    let (env, client, admin, _token) = setup();
    let school = Address::generate(&env);

    // Verify then reject school
    client.verify_cause(&school);
    client.reject_cause(&school);

    // Check revocation event format
    let events = env.events().all();
    let school_rev_event = events.iter().find(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchRev")
    }).unwrap();

    // Event should have format: (symbol_short!("SchRev"), school_addr)
    assert_eq!(school_rev_event.topics.len(), 2);
    assert_eq!(school_rev_event.topics.get(0).unwrap(), &symbol_short!("SchRev"));
    assert_eq!(school_rev_event.topics.get(1).unwrap(), &school.into_val(&env));
    assert_eq!(school_rev_event.data, admin.into_val(&env));
}

#[test]
fn test_verify_same_school_multiple_times() {
    let (env, client, admin, _token) = setup();
    let school = Address::generate(&env);

    // Verify school multiple times
    client.verify_cause(&school);
    client.verify_cause(&school);
    client.verify_cause(&school);

    // Should still be verified
    assert!(client.is_cause_verified(&school));

    // Should emit multiple SchReg events (each call emits event)
    let events = env.events().all();
    let school_reg_events: Vec<_> = events.iter().filter(|event| {
        event.topics.get(0).unwrap() == &symbol_short!("SchReg")
            && event.topics.get(1).unwrap() == &school.into_val(&env)
    }).collect();

    assert_eq!(school_reg_events.len(), 3);
}

#[test]
fn test_contract_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);
    let school = Address::generate(&env);

    // Try to verify cause without initializing contract
    let result = client.try_verify_cause(&school);
    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));

    // Try to reject cause without initializing contract
    let result = client.try_reject_cause(&school);
    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));
}