#![cfg(test)]
//! Direct regression tests for `create_campaign_with_fee` — issue #1484.
//!
//! `create_campaign_with_fee` is documented as an alias for
//! `create_pool_with_fee`, but the existing tests only exercise the pool-named
//! entry point. These tests call `create_campaign_with_fee` itself and verify
//! the full path it delegates to: fee transfer, accumulated unclaimed fees,
//! event emission and the resulting campaign state.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::StellarAssetClient,
    Address, Env, IntoVal, String, Val,
};

// -- helpers ---------------------------------------------------------------

/// Register a Stellar asset contract, mint `amount` to `recipient`, and return
/// the token address.
fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let asset_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(asset_admin);
    StellarAssetClient::new(env, &token.address()).mint(recipient, &amount);
    token.address()
}

/// Current `token` balance of `account`.
fn token_balance(env: &Env, token: &Address, account: &Address) -> i128 {
    soroban_sdk::token::Client::new(env, token).balance(account)
}

/// Register the contract, configure a creation fee and fund a creator.
/// Returns the contract address, the admin, the creator and the fee token.
fn setup(env: &Env, fee: i128, creator_balance: i128) -> (Address, Address, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());

    let admin = Address::generate(env);
    let creator = Address::generate(env);
    let fee_token = create_token(env, creator_balance, &creator);

    let client = ContractClient::new(env, &contract_id);
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    (contract_id, admin, creator, fee_token)
}

// -- Test 1: successful creation, state and fee transfer -------------------

/// `create_campaign_with_fee` must create a campaign from the supplied
/// arguments and move exactly the configured creation fee from the creator to
/// the contract.
#[test]
fn test_create_campaign_with_fee_creates_pool_and_transfers_fee() {
    let env = Env::default();
    let (contract_id, admin, creator, fee_token) = setup(&env, 250_000_000, 1_000_000_000);
    let client = ContractClient::new(&env, &contract_id);

    let goal = 5_000_000_000u128;
    let deadline = 100_000u64;

    let pool_id = client.create_campaign_with_fee(
        &creator,
        &String::from_str(&env, "Campaign Alias Fund"),
        &String::from_str(&env, "Created through create_campaign_with_fee"),
        &goal,
        &deadline,
        &fee_token,
    );

    assert_eq!(pool_id, 1u32, "the first created campaign must get ID 1");
    assert_eq!(client.get_pool_count(), 1u32);
    assert_eq!(client.get_creation_fee(), 250_000_000);

    // Resulting state, read back through the campaign getters.
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator, "the creator must be the campaign sponsor");
    assert_eq!(pool.2, goal);
    assert_eq!(pool.3, 0u128, "a new campaign has collected nothing yet");
    assert_eq!(pool.5, deadline);
    assert_eq!(client.get_campaign_goal(&pool_id), goal);
    assert_eq!(client.get_total_raised(&pool_id), 0u128);

    let (title, description) = client.get_pool_metadata(&pool_id);
    assert_eq!(title, String::from_str(&env, "Campaign Alias Fund"));
    assert_eq!(
        description,
        String::from_str(&env, "Created through create_campaign_with_fee")
    );

    // Fee transfer.
    assert_eq!(
        token_balance(&env, &fee_token, &creator),
        1_000_000_000 - 250_000_000,
        "the creator must be debited exactly the creation fee"
    );
    assert_eq!(
        token_balance(&env, &fee_token, &contract_id),
        250_000_000,
        "the contract must hold exactly the creation fee"
    );

    // The configured fee is claimable by the admin that set it.
    assert_eq!(client.claim_protocol_fees(&admin, &fee_token), 250_000_000);
}

// -- Test 2: fee is accumulated as claimable protocol fees -----------------

/// The creation fee must be recorded as accumulated unclaimed fees, which is
/// what `claim_protocol_fees` later pays out to the admin.
#[test]
fn test_create_campaign_with_fee_accumulates_unclaimed_fees() {
    let env = Env::default();
    let (contract_id, admin, creator, fee_token) = setup(&env, 100_000_000, 1_000_000_000);
    let client = ContractClient::new(&env, &contract_id);

    client.create_campaign_with_fee(
        &creator,
        &String::from_str(&env, "First Campaign"),
        &String::from_str(&env, "Pays a creation fee"),
        &1_000_000_000u128,
        &100_000u64,
        &fee_token,
    );
    client.create_campaign_with_fee(
        &creator,
        &String::from_str(&env, "Second Campaign"),
        &String::from_str(&env, "Pays another creation fee"),
        &1_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    let claimable = client.claim_protocol_fees(&admin, &fee_token);
    assert_eq!(
        claimable, 200_000_000,
        "both creation fees must accumulate as claimable protocol fees"
    );
    assert_eq!(
        token_balance(&env, &fee_token, &admin),
        200_000_000,
        "claimed fees must be transferred to the admin"
    );
}

// -- Test 3: event emission -----------------------------------------------

/// A successful `create_campaign_with_fee` must emit a single fee-payment event
/// (`fee_paid`) and then the `pool_crtd` event for the new campaign, in that order.
///
/// `env.events().all()` reports the events published by the last top-level
/// invocation only; events from the fee-token contract are filtered out so the
/// comparison covers exactly the events this contract published.
#[test]
fn test_create_campaign_with_fee_emits_fee_and_pool_created_events() {
    let env = Env::default();
    let (contract_id, _admin, creator, fee_token) = setup(&env, 100_000_000, 1_000_000_000);
    let client = ContractClient::new(&env, &contract_id);

    let fee: i128 = 100_000_000;
    let goal = 2_000_000_000u128;
    let title = String::from_str(&env, "Event Campaign");
    let description = String::from_str(&env, "Checking emitted events");

    let pool_id = client.create_campaign_with_fee(
        &creator,
        &title,
        &description,
        &goal,
        &100_000u64,
        &fee_token,
    );

    let pool_created_topics: Vec<Val> =
        soroban_sdk::vec![&env, POOL_CREATED.into_val(&env), pool_id.into_val(&env)];
    let fee_paid_topics: Vec<Val> = soroban_sdk::vec![&env, FEE_PAID.into_val(&env)];

    let expected: Vec<(Address, Vec<Val>, Val)> = soroban_sdk::vec![
        &env,
        (
            contract_id.clone(),
            fee_paid_topics,
            (creator.clone(), fee).into_val(&env),
        ),
        (
            contract_id.clone(),
            pool_created_topics,
            (creator, goal, title, description).into_val(&env),
        ),
    ];

    assert_eq!(
        env.events().all().filter_by_contract(&contract_id),
        expected,
        "create_campaign_with_fee must emit fee_paid and pool_crtd"
    );
}

// -- Test 4: zero fee skips the token transfer -----------------------------

/// With a zero creation fee the alias must still create the campaign without
/// requiring or moving any tokens.
#[test]
fn test_create_campaign_with_fee_zero_fee_creates_campaign_without_transfer() {
    let env = Env::default();
    let (contract_id, admin, creator, fee_token) = setup(&env, 0, 0);
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = client.create_campaign_with_fee(
        &creator,
        &String::from_str(&env, "Free Campaign"),
        &String::from_str(&env, "No creation fee configured"),
        &1_000_000_000u128,
        &100_000u64,
        &fee_token,
    );

    assert_eq!(pool_id, 1u32);
    assert_eq!(
        token_balance(&env, &fee_token, &contract_id),
        0,
        "no tokens may be transferred when the creation fee is zero"
    );
    assert_eq!(
        client.try_claim_protocol_fees(&admin, &fee_token),
        Err(Ok(ContractError::NoUnclaimedFees.into())),
        "a zero-fee campaign must not leave claimable fees behind"
    );
}

// -- Test 5: inherited validation behaviour -------------------------------

/// The alias must inherit `create_pool`'s validation: a zero application
/// deadline is rejected and no campaign is created.
#[test]
fn test_create_campaign_with_fee_rejects_zero_deadline() {
    let env = Env::default();
    let (contract_id, _admin, creator, fee_token) = setup(&env, 100_000_000, 1_000_000_000);
    let client = ContractClient::new(&env, &contract_id);

    let res = client.try_create_campaign_with_fee(
        &creator,
        &String::from_str(&env, "No Deadline"),
        &String::from_str(&env, "Must be rejected"),
        &1_000_000_000u128,
        &0u64,
        &fee_token,
    );

    assert_eq!(
        res,
        Err(Ok(ContractError::InvalidPoolDeadline.into())),
        "a zero application deadline must be rejected by create_campaign_with_fee"
    );
    assert_eq!(client.get_pool_count(), 0u32, "no campaign may be created");
}
