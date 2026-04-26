#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::{
    contract::{FundEduContract, FundEduContractClient, FundEduError},
    types::ScholarshipPool,
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn setup() -> (Env, FundEduContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(FundEduContract, ());
    let client = FundEduContractClient::new(&env, &contract_id);
    (env, client)
}

fn setup_with_admin() -> (Env, FundEduContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(FundEduContract, ());
    let client = FundEduContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin).unwrap();
    (env, client, admin)
}

// ── initialize ────────────────────────────────────────────────────────────────

#[test]
fn initialize_succeeds() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    assert!(client.initialize(&admin).is_ok());
}

#[test]
fn initialize_twice_returns_already_initialized() {
    let (env, client, _) = setup_with_admin();
    let other = Address::generate(&env);
    let result = client.try_initialize(&other);
    assert_eq!(result, Err(Ok(FundEduError::AlreadyInitialized)));
}

// ── upgrade ───────────────────────────────────────────────────────────────────

#[test]
fn upgrade_without_initialize_returns_unauthorized() {
    let (env, client) = setup();
    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let result = client.try_upgrade(&hash);
    assert_eq!(result, Err(Ok(FundEduError::Unauthorized)));
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn upgrade_non_admin_panics() {
    let (env, client, _admin) = setup_with_admin();
    // Remove mock auths so the non-admin call fails auth
    let env2 = Env::default();
    let contract_id = env2.register(FundEduContract, ());
    let client2 = FundEduContractClient::new(&env2, &contract_id);
    // Initialize with a real admin (mocked)
    env2.mock_all_auths();
    let admin = Address::generate(&env2);
    client2.initialize(&admin).unwrap();
    // Now clear auths and try upgrade as a different address — should panic
    env2.set_auths(&[]);
    let hash = BytesN::from_array(&env2, &[1u8; 32]);
    client2.upgrade(&hash).unwrap();
}

// ── success cases ─────────────────────────────────────────────────────────────

#[test]
fn create_pool_success_returns_pool_id() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let token = Address::generate(&env);

    let pool_id = client
        .create_pool(
            &sponsor,
            &String::from_str(&env, "STEM 2026"),
            &50_000_000i128,
            &token,
        )
        .unwrap();

    assert_eq!(pool_id, 0);
}

#[test]
fn create_pool_success_persists_correct_data() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let token = Address::generate(&env);
    let name = String::from_str(&env, "Arts Fund");

    let pool_id = client
        .create_pool(&sponsor, &name, &10_000i128, &token)
        .unwrap();

    let pool: ScholarshipPool = client.get_pool(&pool_id).expect("pool must exist");

    assert_eq!(pool.name, name);
    assert_eq!(pool.sponsor, sponsor);
    assert_eq!(pool.target_amount, 10_000i128);
    assert_eq!(pool.token_address, token);
    assert!(pool.is_active);
}

#[test]
fn create_pool_increments_pool_id() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let token = Address::generate(&env);

    let id0 = client
        .create_pool(
            &sponsor,
            &String::from_str(&env, "Pool A"),
            &1_000i128,
            &token,
        )
        .unwrap();

    let id1 = client
        .create_pool(
            &sponsor,
            &String::from_str(&env, "Pool B"),
            &2_000i128,
            &token,
        )
        .unwrap();

    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
}

// ── failure cases ─────────────────────────────────────────────────────────────

#[test]
fn create_pool_zero_amount_returns_invalid_funding() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let token = Address::generate(&env);

    let result = client.try_create_pool(
        &sponsor,
        &String::from_str(&env, "Bad Pool"),
        &0i128,
        &token,
    );

    assert_eq!(result, Err(Ok(FundEduError::InvalidFunding)));
}

#[test]
fn create_pool_negative_amount_returns_invalid_funding() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let token = Address::generate(&env);

    let result = client.try_create_pool(
        &sponsor,
        &String::from_str(&env, "Negative Pool"),
        &-1i128,
        &token,
    );

    assert_eq!(result, Err(Ok(FundEduError::InvalidFunding)));
}

#[test]
fn create_pool_empty_name_returns_invalid_sponsor() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let token = Address::generate(&env);

    let result = client.try_create_pool(
        &sponsor,
        &String::from_str(&env, ""),
        &5_000i128,
        &token,
    );

    assert_eq!(result, Err(Ok(FundEduError::InvalidSponsor)));
}

#[test]
fn get_pool_returns_none_for_unknown_id() {
    let (_env, client) = setup();
    assert!(client.get_pool(&99).is_none());
}

#[test]
fn test_claim_funds_success() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let student = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let token = soroban_sdk::token::Client::new(&env, &token_id);

    // Create pool
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Scholarship 2026"),
        &100_000,
        &token_id,
    );

    // Manually set an approved application in storage
    let app = crate::types::Application {
        student: student.clone(),
        requested_amount: 50_000,
        total_granted: 50_000,
        amount_claimed: 0,
        status: crate::types::ApplicationStatus::Approved,
        milestone_index: 0,
    };

    env.as_contract(&client.address, || {
        crate::storage::set_application(&env, pool_id, student.clone(), &app);
    });

    // Fund the contract with tokens
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    token_admin_client.mint(&client.address, &50_000);

    // Claim part of the funds
    client.claim_funds(&pool_id, &student, &20_000);

    // Verify state
    let updated_app = env.as_contract(&client.address, || {
        crate::storage::get_application(&env, pool_id, student.clone()).unwrap()
    });
    assert_eq!(updated_app.amount_claimed, 20_000);
    assert_eq!(token.balance(&student), 20_000);

    // Claim the rest
    client.claim_funds(&pool_id, &student, &30_000);
    assert_eq!(token.balance(&student), 50_000);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")] // ExceedsGrant
fn test_claim_funds_exceeds_grant() {
    let (env, client) = setup();
    let sponsor = Address::generate(&env);
    let student = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Scholarship 2026"),
        &100_000,
        &token_id,
    );

    let app = crate::types::Application {
        student: student.clone(),
        requested_amount: 50_000,
        total_granted: 50_000,
        amount_claimed: 0,
        status: crate::types::ApplicationStatus::Approved,
        milestone_index: 0,
    };

    env.as_contract(&client.address, || {
        crate::storage::set_application(&env, pool_id, student.clone(), &app);
    });

    client.claim_funds(&pool_id, &student, &60_000);
}
