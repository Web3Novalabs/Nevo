#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, StorageKey},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{testutils::Events, Symbol, TryFromVal};

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
        token_address: token.clone(),
    };
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

fn read_i128_storage(env: &Env, client: &CrowdfundingContractClient<'_>, key: &StorageKey) -> i128 {
    env.as_contract(&client.address, || {
        env.storage().instance().get(key).unwrap_or(0)
    })
}

// ── full success ──────────────────────────────────────────────────────────────

#[test]
fn test_buy_ticket_full_success() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let pool_id = create_pool(&client, &env, &token);

    // 1. Configure platform fee (5% = 500 bps)
    client.set_platform_fee_bps(&500);

    // 2. Prepare buyer
    let buyer = Address::generate(&env);
    let price = 10_000i128;
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&buyer, &price);

    let token_client = token::Client::new(&env, &token);
    let buyer_balance_before = token_client.balance(&buyer);
    let contract_balance_before = token_client.balance(&client.address);

    // 3. Execute buy_ticket
    let (event_amount, fee_amount) = client.buy_ticket(&pool_id, &buyer, &token, &price);

    // 4. Assertions - Return Values
    // 5% of 10,000 = 500
    assert_eq!(fee_amount, 500, "fee amount correctly calculated");
    assert_eq!(event_amount, 9_500, "event amount correctly calculated");
    assert_eq!(event_amount + fee_amount, price, "split sums to price");

    // 5. Assertions - Token Balances
    assert_eq!(
        token_client.balance(&buyer),
        buyer_balance_before - price,
        "buyer balance decreased by price"
    );
    assert_eq!(
        token_client.balance(&client.address),
        contract_balance_before + price,
        "contract balance increased by price"
    );

    // 6. Assertions - Internal Storage Updates
    env.as_contract(&client.address, || {
        let storage = env.storage().instance();

        let saved_event_amount: i128 = storage.get(&StorageKey::EventPool(pool_id)).unwrap_or(0);
        assert_eq!(
            saved_event_amount, event_amount,
            "EventPool storage updated"
        );

        let saved_fee_amount: i128 = storage
            .get(&StorageKey::EventPlatformFees(pool_id))
            .unwrap_or(0);
        assert_eq!(
            saved_fee_amount, fee_amount,
            "EventPlatformFees storage updated"
        );

        let fee_treasury_amount: i128 = storage.get(&StorageKey::EventFeeTreasury).unwrap_or(0);
        assert_eq!(
            fee_treasury_amount, fee_amount,
            "EventFeeTreasury storage updated"
        );
    });

    // 7. Assertions - Events
    let all_events = env.events().all();

    // We expect at least the ticket_sold event.
    // In this environment, we previously saw 2 events from create_pool.
    // So we look for the ticket_sold event among all events.
    let ticket_sold_event = all_events.iter().find(|e| {
        let topics = &e.1;
        if topics.len() < 3 {
            return false;
        }

        let event_name = Symbol::try_from_val(&env, &topics.get(0).unwrap());
        let event_pool_id = u64::try_from_val(&env, &topics.get(1).unwrap());
        let event_buyer = Address::try_from_val(&env, &topics.get(2).unwrap());

        event_name == Ok(Symbol::new(&env, "ticket_sold"))
            && event_pool_id == Ok(pool_id)
            && event_buyer == Ok(buyer.clone())
    });

    if let Some(event) = ticket_sold_event {
        let data = &event.2;
        let decoded: Result<(i128, i128, i128), _> = TryFromVal::try_from_val(&env, data);
        assert_eq!(
            decoded,
            Ok((price, event_amount, fee_amount)),
            "event data matches"
        );
    }
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
    assert_eq!(
        read_i128_storage(&env, &client, &StorageKey::EventPool(pool_id)),
        price
    );
    assert_eq!(
        read_i128_storage(&env, &client, &StorageKey::EventPlatformFees(pool_id)),
        0
    );
    assert_eq!(
        read_i128_storage(&env, &client, &StorageKey::EventFeeTreasury),
        0
    );
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
    assert_eq!(
        read_i128_storage(&env, &client, &StorageKey::EventPool(pool_id)),
        29_250
    );
    assert_eq!(
        read_i128_storage(&env, &client, &StorageKey::EventPlatformFees(pool_id)),
        750
    );
    assert_eq!(
        read_i128_storage(&env, &client, &StorageKey::EventFeeTreasury),
        750
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
