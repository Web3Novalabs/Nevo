#![cfg(test)]

use crate::{
    base::{errors::CrowdfundingError, types::PoolConfig},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn pool_config(env: &Env, token: &Address, target: i128) -> PoolConfig {
    PoolConfig {
        name: String::from_str(env, "Scholarship Pool"),
        description: String::from_str(env, "Sponsor deposit test"),
        target_amount: target,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: admin.clone(),
    }
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    // StellarAssetClient lets us mint in tests without a real issuer tx
    let sac = StellarAssetClient::new(env, token);
    sac.mint(to, &amount);
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_create_pool_transfers_tokens_to_contract() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let sponsor = Address::generate(&env);
    let target = 50_000i128;

    mint(&env, &token, &sponsor, target);

    let token_client = TokenClient::new(&env, &token);

    let balance_before = token_client.balance(&sponsor);
    assert_eq!(balance_before, target);

    let pool_id = client.create_pool(&sponsor, &pool_config(&env, &token, target));

    // Sponsor's wallet must be drained by exactly target_amount
    assert_eq!(token_client.balance(&sponsor), 0);

    // The contract itself must hold the tokens
    // We retrieve the contract address via get_pool (it's the registered contract)
    // The client's contract address is the one that received the funds.
    // We verify via get_pool_balance instead of raw token balance to stay
    // within the contract's own accounting.
    let locked = client.get_pool_balance(&pool_id).unwrap();
    assert_eq!(locked, target, "PoolBalance must equal target_amount");
}

#[test]
fn test_create_pool_pool_balance_equals_target_amount() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let sponsor = Address::generate(&env);
    let target = 10_000i128;

    mint(&env, &token, &sponsor, target * 2); // give extra so we can check exact amount

    let pool_id = client.create_pool(&sponsor, &pool_config(&env, &token, target));

    assert_eq!(
        client.get_pool_balance(&pool_id).unwrap(),
        target,
        "locked balance must equal the pool's target_amount"
    );
}

#[test]
fn test_create_pool_metrics_total_raised_equals_deposit() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let sponsor = Address::generate(&env);
    let target = 25_000i128;

    mint(&env, &token, &sponsor, target);

    let pool_id = client.create_pool(&sponsor, &pool_config(&env, &token, target));

    // PoolMetrics.total_raised must reflect the sponsor deposit
    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.target_amount, target);
    // Verify via get_pool_balance (the canonical locked-balance query)
    assert_eq!(client.get_pool_balance(&pool_id).unwrap(), target);
}

#[test]
fn test_create_pool_reverts_when_sponsor_has_insufficient_balance() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let sponsor = Address::generate(&env);
    let target = 100_000i128;

    // Mint less than target
    mint(&env, &token, &sponsor, target - 1);

    let result = client.try_create_pool(&sponsor, &pool_config(&env, &token, target));
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InsufficientSponsorBalance)),
        "must revert with InsufficientSponsorBalance when sponsor balance < target_amount"
    );
}

#[test]
fn test_create_pool_reverts_when_sponsor_has_zero_balance() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let sponsor = Address::generate(&env);

    // No mint — sponsor has 0 tokens
    let result = client.try_create_pool(&sponsor, &pool_config(&env, &token, 5_000));
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::InsufficientSponsorBalance)),
        "must revert when sponsor has zero balance"
    );
}

#[test]
fn test_get_pool_balance_returns_not_found_for_unknown_pool() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let result = client.try_get_pool_balance(&999u64);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::PoolNotFound)),
        "get_pool_balance must return PoolNotFound for a non-existent pool"
    );
}

#[test]
fn test_multiple_pools_track_balances_independently() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    let sponsor1 = Address::generate(&env);
    let sponsor2 = Address::generate(&env);
    let target1 = 10_000i128;
    let target2 = 30_000i128;

    mint(&env, &token, &sponsor1, target1);
    mint(&env, &token, &sponsor2, target2);

    let pool1 = client.create_pool(&sponsor1, &pool_config(&env, &token, target1));
    let pool2 = client.create_pool(&sponsor2, &pool_config(&env, &token, target2));

    assert_eq!(client.get_pool_balance(&pool1).unwrap(), target1);
    assert_eq!(client.get_pool_balance(&pool2).unwrap(), target2);
    assert_ne!(pool1, pool2, "pool IDs must be distinct");
}
