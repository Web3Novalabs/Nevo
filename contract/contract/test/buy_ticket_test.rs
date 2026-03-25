#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token, &0);
    (client, admin, token)
}

fn create_pool(client: &CrowdfundingContractClient<'_>, env: &Env, token: &Address) -> u64 {
    let creator = Address::generate(env);
    let config = PoolConfig {
        name: soroban_sdk::String::from_str(env, "Event Pool"),
        description: soroban_sdk::String::from_str(env, "Test event"),
        target_amount: 1_000_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
    };
    // create_pool requires the token to match; use contribute pattern
    let _ = token; // token stored in contract state already
    client.create_pool(&creator, &config)
}

fn mint_and_buy(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token: &Address,
    pool_id: u64,
    price: i128,
) -> (Address, (i128, i128)) {
    let buyer = Address::generate(env);
    let token_client = token::StellarAssetClient::new(env, token);
    token_client.mint(&buyer, &price);
    let result = client.buy_ticket(&pool_id, &buyer, token, &price);
    (buyer, result)
}

// ── fee arithmetic ────────────────────────────────────────────────────────────

#[test]
fn test_buy_ticket_zero_fee_bps_full_amount_to_event_pool() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    // fee_bps = 0 (default) → all goes to event pool
    let price = 10_000i128;
    let (_, (event_amount, fee_amount)) = mint_and_buy(&env, &client, &token, pool_id, price);

    assert_eq!(event_amount, 10_000, "full price must go to event pool");
    assert_eq!(fee_amount, 0, "no platform fee when bps = 0");
    assert_eq!(event_amount + fee_amount, price, "split must sum to price");
}

#[test]
fn test_buy_ticket_250_bps_split() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    client.set_platform_fee_bps(&250); // 2.5%

    let price = 10_000i128;
    let (_, (event_amount, fee_amount)) = mint_and_buy(&env, &client, &token, pool_id, price);

    // 2.5% of 10_000 = 250
    assert_eq!(fee_amount, 250);
    assert_eq!(event_amount, 9_750);
    assert_eq!(event_amount + fee_amount, price);
}

#[test]
fn test_buy_ticket_500_bps_split() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    client.set_platform_fee_bps(&500); // 5%

    let price = 20_000i128;
    let (_, (event_amount, fee_amount)) = mint_and_buy(&env, &client, &token, pool_id, price);

    // 5% of 20_000 = 1_000
    assert_eq!(fee_amount, 1_000);
    assert_eq!(event_amount, 19_000);
    assert_eq!(event_amount + fee_amount, price);
}

#[test]
fn test_buy_ticket_1000_bps_split() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    client.set_platform_fee_bps(&1_000); // 10%

    let price = 50_000i128;
    let (_, (event_amount, fee_amount)) = mint_and_buy(&env, &client, &token, pool_id, price);

    // 10% of 50_000 = 5_000
    assert_eq!(fee_amount, 5_000);
    assert_eq!(event_amount, 45_000);
    assert_eq!(event_amount + fee_amount, price);
}

#[test]
fn test_buy_ticket_10000_bps_all_to_platform() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    client.set_platform_fee_bps(&10_000); // 100%

    let price = 5_000i128;
    let (_, (event_amount, fee_amount)) = mint_and_buy(&env, &client, &token, pool_id, price);

    assert_eq!(fee_amount, 5_000);
    assert_eq!(event_amount, 0);
    assert_eq!(event_amount + fee_amount, price);
}

#[test]
fn test_buy_ticket_rounding_floors_fee() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    client.set_platform_fee_bps(&333); // 3.33%

    // 3.33% of 100 = 3.33 → floors to 3
    let price = 100i128;
    let (_, (event_amount, fee_amount)) = mint_and_buy(&env, &client, &token, pool_id, price);

    assert_eq!(fee_amount, 3, "fee must floor (integer division)");
    assert_eq!(event_amount, 97);
    assert_eq!(event_amount + fee_amount, price);
}

#[test]
fn test_buy_ticket_accumulates_across_multiple_purchases() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    client.set_platform_fee_bps(&250); // 2.5%

    let price = 10_000i128;

    // Three separate buyers
    for _ in 0..3 {
        mint_and_buy(&env, &client, &token, pool_id, price);
    }

    // Each ticket: fee = 250, event = 9_750
    // After 3 tickets: event pool = 29_250, platform fees = 750
    let token_client = token::Client::new(&env, &token);
    let contract_balance = token_client.balance(&client.address);
    assert_eq!(
        contract_balance,
        price * 3,
        "contract holds all ticket revenue"
    );
}

// ── validation ────────────────────────────────────────────────────────────────

#[test]
fn test_buy_ticket_zero_price_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    let buyer = Address::generate(&env);
    let result = client.try_buy_ticket(&pool_id, &buyer, &token, &0);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_buy_ticket_pool_not_found_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let buyer = Address::generate(&env);
    let result = client.try_buy_ticket(&999u64, &buyer, &token, &1_000);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_buy_ticket_wrong_token_fails() {
    let env = Env::default();
    let (client, _, _token) = setup(&env);
    let pool_id = create_pool(&client, &env, &_token);

    // Register a different token
    let other_admin = Address::generate(&env);
    let other_token = env
        .register_stellar_asset_contract_v2(other_admin)
        .address();

    let buyer = Address::generate(&env);
    let result = client.try_buy_ticket(&pool_id, &buyer, &other_token, &1_000);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidToken)));
}

#[test]
fn test_buy_ticket_requires_buyer_auth() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    let buyer = Address::generate(&env);
    let token_client = token::StellarAssetClient::new(&env, &token);
    token_client.mint(&buyer, &10_000);

    // Verify buyer auth is recorded after a successful call
    client.buy_ticket(&pool_id, &buyer, &token, &10_000);

    let auths = env.auths();
    assert!(
        auths.iter().any(|(addr, _)| addr == &buyer),
        "buyer auth must be recorded"
    );
}
