#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal,
};

use crate::{
    base::errors::CrowdfundingError,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token, &0);
    (client, admin)
}

// ── happy-path ────────────────────────────────────────────────────────────────

#[test]
fn test_set_and_get_platform_fee_bps() {
    let env = Env::default();
    let (client, _) = setup(&env);

    client.set_platform_fee_bps(&250);
    assert_eq!(client.get_platform_fee_bps(), 250);
}

#[test]
fn test_default_platform_fee_bps_is_zero() {
    let env = Env::default();
    let (client, _) = setup(&env);

    assert_eq!(client.get_platform_fee_bps(), 0);
}

#[test]
fn test_set_platform_fee_bps_zero() {
    let env = Env::default();
    let (client, _) = setup(&env);

    client.set_platform_fee_bps(&0);
    assert_eq!(client.get_platform_fee_bps(), 0);
}

#[test]
fn test_set_platform_fee_bps_max() {
    let env = Env::default();
    let (client, _) = setup(&env);

    // 10 000 bps = 100 % — boundary must be accepted
    client.set_platform_fee_bps(&10_000);
    assert_eq!(client.get_platform_fee_bps(), 10_000);
}

#[test]
fn test_set_platform_fee_bps_update() {
    let env = Env::default();
    let (client, _) = setup(&env);

    client.set_platform_fee_bps(&100);
    assert_eq!(client.get_platform_fee_bps(), 100);

    client.set_platform_fee_bps(&500);
    assert_eq!(client.get_platform_fee_bps(), 500);
}

// ── admin auth ────────────────────────────────────────────────────────────────

#[test]
fn test_set_platform_fee_bps_requires_admin_auth() {
    let env = Env::default();
    let (client, admin) = setup(&env);

    // Verify the admin auth is required by checking auths after the call
    client.set_platform_fee_bps(&250);

    let auths = env.auths();
    assert!(
        auths.iter().any(|(addr, _)| addr == &admin),
        "admin auth must be recorded"
    );
}

#[test]
fn test_set_platform_fee_bps_non_admin_fails() {
    let env = Env::default();
    let (client, _) = setup(&env);

    let non_admin = Address::generate(&env);

    // Only mock auth for the non-admin — the contract should reject it
    client
        .mock_auths(&[MockAuth {
            address: &non_admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "set_platform_fee_bps",
                args: (250u32,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_set_platform_fee_bps(&250)
        .unwrap_err();
}

// ── validation ────────────────────────────────────────────────────────────────

#[test]
fn test_set_platform_fee_bps_above_10000_fails() {
    let env = Env::default();
    let (client, _) = setup(&env);

    let result = client.try_set_platform_fee_bps(&10_001);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InvalidFee)),
        "fee_bps > 10_000 must return InvalidFee"
    );
}

#[test]
fn test_set_platform_fee_bps_uninitialized_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Contract not initialized — no admin stored
    let result = client.try_set_platform_fee_bps(&250);
    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));
}
