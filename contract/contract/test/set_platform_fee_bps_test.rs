#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, Symbol,
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
    let _ = client
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

// ── PlatformFeeUpdated event ──────────────────────────────────────────────────

#[test]
fn test_platform_fee_updated_event_emitted_with_old_and_new_fee() {
    let env = Env::default();
    let (client, _) = setup(&env);

    // Set an initial fee so old_fee is non-zero on the second call.
    client.set_platform_fee_bps(&100);

    // Clear events accumulated during setup and first call.
    let _ = env.events().all();

    client.set_platform_fee_bps(&500);

    let all_events = env.events().all();

    // Find the platform_fee_updated event among all emitted events.
    let fee_updated = all_events.iter().find(|(_, topics, _)| {
        if let Ok(sym) = Symbol::try_from_val(&env, &topics.get(0).unwrap()) {
            sym == Symbol::new(&env, "platform_fee_updated")
        } else {
            false
        }
    });

    assert!(
        fee_updated.is_some(),
        "platform_fee_updated event must be emitted"
    );

    let (_, _, data) = fee_updated.unwrap();
    let (old_fee, new_fee): (u32, u32) = soroban_sdk::FromVal::from_val(&env, &data);
    assert_eq!(old_fee, 100, "old_fee_bps must be the previously set value");
    assert_eq!(new_fee, 500, "new_fee_bps must be the newly set value");
}

#[test]
fn test_platform_fee_updated_event_old_fee_is_zero_on_first_set() {
    let env = Env::default();
    let (client, _) = setup(&env);

    let _ = env.events().all();

    client.set_platform_fee_bps(&250);

    let all_events = env.events().all();
    let fee_updated = all_events.iter().find(|(_, topics, _)| {
        if let Ok(sym) = Symbol::try_from_val(&env, &topics.get(0).unwrap()) {
            sym == Symbol::new(&env, "platform_fee_updated")
        } else {
            false
        }
    });

    assert!(
        fee_updated.is_some(),
        "platform_fee_updated event must be emitted on first set"
    );

    let (_, _, data) = fee_updated.unwrap();
    let (old_fee, new_fee): (u32, u32) = soroban_sdk::FromVal::from_val(&env, &data);
    assert_eq!(old_fee, 0, "old_fee_bps must be 0 when no fee was set before");
    assert_eq!(new_fee, 250, "new_fee_bps must match the value passed in");
}
