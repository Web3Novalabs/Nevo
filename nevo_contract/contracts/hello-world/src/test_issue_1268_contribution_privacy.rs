//! Tests for private vs public pool contributions — issue #1268
//!
//! Covers:
//!   1. Public contribution (is_private=false) emits event with correct privacy flag.
//!   2. Private contribution (is_private=true) emits event with privacy flag set.
//!   3. Verify event emission contains all required fields including privacy status.
//!   4. Multiple contributions preserve independent privacy flags.

#![cfg(test)]

use super::*;
use soroban_sdk::{
    symbol_short,
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env, String, Symbol, TryIntoVal,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// (1) Public contribution (is_private = false) emits event with correct privacy flag.
/// In the Nevo architecture, public pool participation / application via `apply_to_pool`
/// publishes an `APPLICATION_SUBMITTED` event with `privacy_flag = false`.
#[test]
fn test_public_contribution_emits_event_with_privacy_flag_false() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Public Contribution Pool"),
        &String::from_str(&env, "Testing public contribution privacy flag"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let events_before = env.events().all().len();

    let app_data = String::from_str(&env, "Public student application data");
    client.apply_to_pool(&pool_id, &student, &app_data);

    let events = env.events().all();
    assert_eq!(
        events.len(),
        events_before + 1,
        "A public contribution must emit exactly one new event"
    );

    let event = events.last().unwrap();
    // Verify topic: topic[0] is APPLICATION_SUBMITTED ("app_sub"), topic[1] is pool_id
    assert_eq!(event.1.len(), 2, "Event must have exactly 2 topics");
    let topic_sym: Symbol = event.1.get(0).unwrap().try_into_val(&env).unwrap();
    let topic_id: u32 = event.1.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(topic_sym, symbol_short!("app_sub"));
    assert_eq!(topic_id, pool_id);

    // Verify data: (student, app_count, privacy_flag)
    let (emitted_student, app_count, is_private): (Address, u32, bool) =
        event.2.try_into_val(&env).unwrap();
    assert_eq!(emitted_student, student);
    assert_eq!(app_count, 1);
    assert!(
        !is_private,
        "Public contribution event must have privacy_flag = false"
    );
}

/// (2) Private contribution (is_private = true) emits event with privacy flag set.
/// In the Nevo architecture, token-backed donations via `donate_with_token`
/// publish a `CONTRIBUTION` event with `privacy_flag = true` to preserve contributor anonymity.
#[test]
fn test_private_contribution_emits_event_with_privacy_flag_true() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let amount: i128 = 250_000_000;
    let token = create_token(&env, amount, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Private Contribution Pool"),
        &String::from_str(&env, "Testing private contribution privacy flag"),
        &1_000_000_000u128,
        &100_000u64,
    );

    let events_before = env.events().all().len();

    client.donate_with_token(&pool_id, &donor, &token, &amount);

    let events = env.events().all();
    assert_eq!(
        events.len(),
        events_before + 1,
        "A private contribution must emit exactly one new event"
    );

    let event = events.last().unwrap();
    // Verify topic: topic[0] is CONTRIBUTION ("contrib"), topic[1] is pool_id
    assert_eq!(event.1.len(), 2, "Event must have exactly 2 topics");
    let topic_sym: Symbol = event.1.get(0).unwrap().try_into_val(&env).unwrap();
    let topic_id: u32 = event.1.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(topic_sym, symbol_short!("contrib"));
    assert_eq!(topic_id, pool_id);

    // Verify data: (donor, amount, new_collected, privacy_flag)
    let (emitted_donor, emitted_amount, new_collected, is_private): (Address, i128, u128, bool) =
        event.2.try_into_val(&env).unwrap();
    assert_eq!(emitted_donor, donor);
    assert_eq!(emitted_amount, amount);
    assert_eq!(new_collected, amount as u128);
    assert!(
        is_private,
        "Private contribution event must have privacy_flag = true"
    );
}

/// (3) Verify event emission contains all required fields including privacy status.
/// Both private token contribution and public pool participation events must have
/// standard 2-topic structure (symbol, pool_id) and all required data fields.
#[test]
fn test_contribution_events_contain_all_required_fields_and_privacy_status() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let student = Address::generate(&env);

    let amount: i128 = 500_000_000;
    let token = create_token(&env, amount, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "All Fields Pool"),
        &String::from_str(&env, "Testing event fields integrity"),
        &2_000_000_000u128,
        &100_000u64,
    );

    // 1. Submit public contribution / application
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Student application"),
    );

    // 2. Submit private contribution
    client.donate_with_token(&pool_id, &donor, &token, &amount);

    let events = env.events().all();
    let contract_events: soroban_sdk::Vec<_> = events
        .iter()
        .filter(|(c, _, _)| *c == contract_id)
        .collect();

    // Must have pool creation, application, and donation events
    assert!(contract_events.len() >= 3);

    // Check the application event (second contract event)
    let app_event = contract_events.get(1).unwrap();
    assert_eq!(app_event.1.len(), 2, "Application event must have 2 topics");
    let app_topic_sym: Symbol = app_event.1.get(0).unwrap().try_into_val(&env).unwrap();
    let app_topic_id: u32 = app_event.1.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(app_topic_sym, symbol_short!("app_sub"));
    assert_eq!(app_topic_id, pool_id);

    let (app_student, app_seq, app_privacy): (Address, u32, bool) =
        app_event.2.try_into_val(&env).unwrap();
    assert_eq!(app_student, student, "Student address must match");
    assert_eq!(app_seq, 1, "App sequence must be 1");
    assert_eq!(
        app_privacy, false,
        "Public application privacy flag must be false"
    );

    // Check the contribution event (third contract event)
    let contrib_event = contract_events.get(2).unwrap();
    assert_eq!(
        contrib_event.1.len(),
        2,
        "Contribution event must have 2 topics"
    );
    let contrib_topic_sym: Symbol = contrib_event.1.get(0).unwrap().try_into_val(&env).unwrap();
    let contrib_topic_id: u32 = contrib_event.1.get(1).unwrap().try_into_val(&env).unwrap();
    assert_eq!(contrib_topic_sym, symbol_short!("contrib"));
    assert_eq!(contrib_topic_id, pool_id);

    let (c_donor, c_amount, c_collected, c_privacy): (Address, i128, u128, bool) =
        contrib_event.2.try_into_val(&env).unwrap();
    assert_eq!(c_donor, donor, "Donor address must match");
    assert_eq!(c_amount, amount, "Contribution amount must match");
    assert_eq!(c_collected, amount as u128, "New collected total must match");
    assert_eq!(
        c_privacy, true,
        "Private contribution privacy flag must be true"
    );
}

/// Multiple contributions across public and private channels preserve correct privacy flags.
#[test]
fn test_multiple_contributions_preserve_independent_privacy_flags() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_1 = Address::generate(&env);
    let donor_2 = Address::generate(&env);
    let student_1 = Address::generate(&env);
    let student_2 = Address::generate(&env);

    let token_1 = create_token(&env, 100_000_000, &donor_1);
    let token_2 = create_token(&env, 200_000_000, &donor_2);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Mixed Privacy Pool"),
        &String::from_str(&env, "Testing multiple events with varying privacy"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Public contribution 1
    client.apply_to_pool(&pool_id, &student_1, &String::from_str(&env, "App 1"));
    let events = env.events().all();
    let last = events.last().unwrap();
    let (_, _, priv_1): (Address, u32, bool) = last.2.try_into_val(&env).unwrap();
    assert!(!priv_1, "First public contribution flag must be false");

    // Private contribution 1
    client.donate_with_token(&pool_id, &donor_1, &token_1, &100_000_000);
    let events = env.events().all();
    let last = events.last().unwrap();
    let (_, _, _, priv_2): (Address, i128, u128, bool) = last.2.try_into_val(&env).unwrap();
    assert!(priv_2, "First private contribution flag must be true");

    // Public contribution 2
    client.apply_to_pool(&pool_id, &student_2, &String::from_str(&env, "App 2"));
    let events = env.events().all();
    let last = events.last().unwrap();
    let (_, _, priv_3): (Address, u32, bool) = last.2.try_into_val(&env).unwrap();
    assert!(!priv_3, "Second public contribution flag must be false");

    // Private contribution 2
    client.donate_with_token(&pool_id, &donor_2, &token_2, &200_000_000);
    let events = env.events().all();
    let last = events.last().unwrap();
    let (_, _, _, priv_4): (Address, i128, u128, bool) = last.2.try_into_val(&env).unwrap();
    assert!(priv_4, "Second private contribution flag must be true");
}
