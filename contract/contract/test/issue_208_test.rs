#![cfg(test)]

//! Issue #208 — Prevent withdrawing more fees than exist.
//!
//! Covers:
//!   - Failure: withdraw 101 when only 100 platform fees collected → InsufficientPlatformFees
//!   - Failure: withdraw 101 when only 100 event fees collected    → InsufficientEventFees
//!   - Success: withdraw exactly 100 → state decrements correctly
//!   - Isolation: pool-contribution funds are never touched by a fee withdrawal

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, StorageKey},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

// ---------------------------------------------------------------------------
// Shared setup
// ---------------------------------------------------------------------------

struct Fixture<'a> {
    env: Env,
    client: CrowdfundingContractClient<'a>,
    admin: Address,
    token: Address,
}

fn setup() -> Fixture<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token, &0);

    // SAFETY: the Env outlives the test function; the 'static bound is
    // satisfied because Env::default() is owned and moved into the struct.
    let fixture = Fixture {
        env,
        client: unsafe { core::mem::transmute(client) },
        admin,
        token,
    };
    fixture
}

/// Seed exactly `fee_amount` units into the `PlatformFees` counter and mint
/// the same amount to the contract address so the token balance matches.
fn seed_platform_fees(f: &Fixture<'_>, fee_amount: i128) {
    StellarAssetClient::new(&f.env, &f.token).mint(&f.client.address, &fee_amount);
    f.env.as_contract(&f.client.address, || {
        f.env
            .storage()
            .instance()
            .set(&StorageKey::PlatformFees, &fee_amount);
    });
}

/// Seed exactly `fee_amount` units into the `EventFeeTreasury` counter and
/// mint the same amount to the contract address.
fn seed_event_fees(f: &Fixture<'_>, fee_amount: i128) {
    StellarAssetClient::new(&f.env, &f.token).mint(&f.client.address, &fee_amount);
    f.env.as_contract(&f.client.address, || {
        f.env
            .storage()
            .instance()
            .set(&StorageKey::EventFeeTreasury, &fee_amount);
    });
}

/// Read the current `PlatformFees` counter from contract storage.
fn read_platform_fees(f: &Fixture<'_>) -> i128 {
    f.env.as_contract(&f.client.address, || {
        f.env
            .storage()
            .instance()
            .get::<_, i128>(&StorageKey::PlatformFees)
            .unwrap_or(0)
    })
}

/// Read the current `EventFeeTreasury` counter from contract storage.
fn read_event_fees(f: &Fixture<'_>) -> i128 {
    f.env.as_contract(&f.client.address, || {
        f.env
            .storage()
            .instance()
            .get::<_, i128>(&StorageKey::EventFeeTreasury)
            .unwrap_or(0)
    })
}

// ---------------------------------------------------------------------------
// Failure cases
// ---------------------------------------------------------------------------

/// Withdraw 101 when only 100 platform fees exist → InsufficientPlatformFees.
#[test]
fn test_platform_fee_withdrawal_exceeds_balance_fails() {
    let f = setup();
    seed_platform_fees(&f, 100);

    let receiver = Address::generate(&f.env);
    let result = f.client.try_withdraw_platform_fees(&receiver, &101);

    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InsufficientPlatformFees)),
        "withdrawing 101 when only 100 collected must return InsufficientPlatformFees"
    );
}

/// Withdraw 101 when only 100 event fees exist → InsufficientEventFees.
#[test]
fn test_event_fee_withdrawal_exceeds_balance_fails() {
    let f = setup();
    seed_event_fees(&f, 100);

    let receiver = Address::generate(&f.env);
    let result = f.client.try_withdraw_event_fees(&f.admin, &receiver, &101);

    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InsufficientEventFees)),
        "withdrawing 101 when only 100 collected must return InsufficientEventFees"
    );
}

/// Withdraw exactly the full balance → must also fail with one unit over.
#[test]
fn test_platform_fee_withdrawal_one_over_exact_balance_fails() {
    let f = setup();
    seed_platform_fees(&f, 100);

    // Drain the full balance first
    let receiver = Address::generate(&f.env);
    f.client.withdraw_platform_fees(&receiver, &100);

    // Now the counter is 0 — even 1 unit must fail
    let result = f.client.try_withdraw_platform_fees(&receiver, &1);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InsufficientPlatformFees)),
        "any withdrawal after full drain must fail"
    );
}

// ---------------------------------------------------------------------------
// Success cases
// ---------------------------------------------------------------------------

/// Withdraw exactly 100 when 100 platform fees exist → succeeds, counter → 0.
#[test]
fn test_platform_fee_withdrawal_exact_amount_succeeds() {
    let f = setup();
    seed_platform_fees(&f, 100);

    let receiver = Address::generate(&f.env);
    let token_client = TokenClient::new(&f.env, &f.token);

    let receiver_before = token_client.balance(&receiver);
    f.client.withdraw_platform_fees(&receiver, &100);
    let receiver_after = token_client.balance(&receiver);

    assert_eq!(
        receiver_after - receiver_before,
        100,
        "receiver must gain exactly 100 tokens"
    );
    assert_eq!(
        read_platform_fees(&f),
        0,
        "PlatformFees counter must be 0 after full withdrawal"
    );
}

/// Withdraw a partial amount → counter decrements by exactly that amount.
#[test]
fn test_platform_fee_partial_withdrawal_decrements_counter() {
    let f = setup();
    seed_platform_fees(&f, 100);

    let receiver = Address::generate(&f.env);
    f.client.withdraw_platform_fees(&receiver, &60);

    assert_eq!(
        read_platform_fees(&f),
        40,
        "PlatformFees counter must reflect the remaining 40"
    );
}

/// Withdraw exactly 100 event fees → succeeds, treasury counter → 0.
#[test]
fn test_event_fee_withdrawal_exact_amount_succeeds() {
    let f = setup();
    seed_event_fees(&f, 100);

    let receiver = Address::generate(&f.env);
    let token_client = TokenClient::new(&f.env, &f.token);

    let receiver_before = token_client.balance(&receiver);
    f.client.withdraw_event_fees(&f.admin, &receiver, &100);
    let receiver_after = token_client.balance(&receiver);

    assert_eq!(
        receiver_after - receiver_before,
        100,
        "receiver must gain exactly 100 tokens"
    );
    assert_eq!(
        read_event_fees(&f),
        0,
        "EventFeeTreasury counter must be 0 after full withdrawal"
    );
}

// ---------------------------------------------------------------------------
// State isolation — pool funds must never be touched
// ---------------------------------------------------------------------------

/// Seed 100 platform fees + 500 pool-contribution funds.
/// Withdraw all 100 fees → pool funds remain exactly 500.
#[test]
fn test_platform_fee_withdrawal_does_not_touch_pool_funds() {
    let f = setup();

    // Create a pool and contribute 500 to it so the contract holds real pool funds
    let creator = Address::generate(&f.env);
    let contributor = Address::generate(&f.env);
    let pool_config = PoolConfig {
        name: String::from_str(&f.env, "Isolation Pool"),
        description: String::from_str(&f.env, "Testing fund isolation"),
        target_amount: 10_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: f.env.ledger().timestamp(),
        token_address: f.token.clone(),
    };
    let pool_id = f.client.create_pool(&creator, &pool_config);

    let contribution_amount = 500i128;
    StellarAssetClient::new(&f.env, &f.token).mint(&contributor, &contribution_amount);
    f.client.contribute(
        &pool_id,
        &contributor,
        &f.token,
        &contribution_amount,
        &false,
    );

    // Separately seed 100 platform fees on top
    seed_platform_fees(&f, 100);

    let token_client = TokenClient::new(&f.env, &f.token);
    let contract_balance_before = token_client.balance(&f.client.address);
    // Contract holds: 500 (pool) + 100 (fees) = 600
    assert_eq!(contract_balance_before, 600);

    // Withdraw all 100 platform fees
    let receiver = Address::generate(&f.env);
    f.client.withdraw_platform_fees(&receiver, &100);

    // Contract must still hold exactly the 500 pool-contribution funds
    let contract_balance_after = token_client.balance(&f.client.address);
    assert_eq!(
        contract_balance_after, 500,
        "pool-contribution funds must be untouched after fee withdrawal"
    );

    // PlatformFees counter must be 0
    assert_eq!(read_platform_fees(&f), 0);

    // Attempting to withdraw 1 more must fail — pool funds are off-limits
    let result = f.client.try_withdraw_platform_fees(&receiver, &1);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InsufficientPlatformFees)),
        "cannot withdraw pool-contribution funds via withdraw_platform_fees"
    );
}

/// End-to-end: fees collected via buy_ticket flow into EventFeeTreasury
/// and are withdrawable; pool funds remain isolated.
#[test]
fn test_event_fee_withdrawal_via_buy_ticket_flow() {
    let f = setup();

    // Set 10% platform fee
    f.client.set_platform_fee_bps(&1_000);

    let creator = Address::generate(&f.env);
    let pool_config = PoolConfig {
        name: String::from_str(&f.env, "Ticket Pool"),
        description: String::from_str(&f.env, "End-to-end event fee test"),
        target_amount: 100_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: f.env.ledger().timestamp(),
        token_address: f.token.clone(),
    };
    let pool_id = f.client.create_pool(&creator, &pool_config);

    // Buyer purchases a ticket for 1_000 → fee = 100, event = 900
    let buyer = Address::generate(&f.env);
    StellarAssetClient::new(&f.env, &f.token).mint(&buyer, &1_000);
    f.client.buy_ticket(&pool_id, &buyer, &f.token, &1_000);

    // EventFeeTreasury must now hold 100
    assert_eq!(read_event_fees(&f), 100);

    // Withdraw 101 → must fail
    let receiver = Address::generate(&f.env);
    assert_eq!(
        f.client.try_withdraw_event_fees(&f.admin, &receiver, &101),
        Err(Ok(CrowdfundingError::InsufficientEventFees)),
        "cannot withdraw more than collected event fees"
    );

    // Withdraw exactly 100 → must succeed
    f.client.withdraw_event_fees(&f.admin, &receiver, &100);
    assert_eq!(read_event_fees(&f), 0);

    // EventPool (creator's share = 900) must be untouched
    let event_pool_balance: i128 = f.env.as_contract(&f.client.address, || {
        f.env
            .storage()
            .instance()
            .get::<_, i128>(&StorageKey::EventPool(pool_id))
            .unwrap_or(0)
    });
    assert_eq!(
        event_pool_balance, 900,
        "event pool (creator funds) must be untouched after fee withdrawal"
    );
}
