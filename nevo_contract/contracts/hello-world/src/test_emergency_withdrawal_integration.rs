#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String, Symbol,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

// ============= ISSUE #1092: INTEGRATION TESTS FOR EMERGENCY WITHDRAWAL FLOW =============

/// Test 1: Admin requests emergency withdrawal, grace period enforced, successful execution
#[test]
fn test_emergency_withdrawal_complete_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let withdrawal_amount = 500_000_000i128;
    let token = create_token(&env, withdrawal_amount, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Pool"),
        &String::from_str(&env, "Testing emergency withdrawal"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.request_emergency_withdraw(&admin, &pool_id, &token, &withdrawal_amount);

    let withdrawal_key = (Symbol::new(&env, "emergency_withdraw"), pool_id);
    let has_request = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(has_request);

    // Advance past grace period
    env.ledger().set_timestamp(GRACE_PERIOD_SECS + 1);

    client.execute_emergency_withdraw(&pool_id);

    let has_request_after = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(!has_request_after);
}

/// Test 2: Grace period is strictly enforced
#[test]
#[should_panic(expected = "Grace period not elapsed")]
fn test_emergency_withdrawal_grace_period_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 500_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Grace Period Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.request_emergency_withdraw(&admin, &pool_id, &token, &500_000_000i128);

    env.ledger().set_timestamp(GRACE_PERIOD_SECS - 1);

    client.execute_emergency_withdraw(&pool_id);
}

/// Test 3: Token balances are updated correctly after emergency withdrawal
#[test]
fn test_emergency_withdrawal_token_balances_correct() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let withdrawal_amount = 300_000_000i128;
    let token = create_token(&env, withdrawal_amount, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Balance Check Pool"),
        &String::from_str(&env, "Test balances"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.request_emergency_withdraw(&admin, &pool_id, &token, &withdrawal_amount);

    env.ledger().set_timestamp(GRACE_PERIOD_SECS + 1);

    client.execute_emergency_withdraw(&pool_id);

    // Contract should have 0 tokens
    let token_client = soroban_sdk::token::Client::new(&env, &token);
    let contract_balance = token_client.balance(&contract_id);
    assert_eq!(contract_balance, 0i128);

    // Admin should have received the tokens
    let admin_balance = token_client.balance(&admin);
    assert_eq!(admin_balance, withdrawal_amount);
}

/// Test 4: Emergency withdrawal for different amounts
#[test]
fn test_emergency_withdrawal_various_amounts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Amounts Pool"),
        &String::from_str(&env, "Test different amounts"),
        &1_000_000_000u128,
        &100_000u64,
    );

    // Test with small amount
    client.request_emergency_withdraw(&admin, &pool_id, &token, &1i128);
    env.ledger().set_timestamp(GRACE_PERIOD_SECS + 1);
    client.execute_emergency_withdraw(&pool_id);

    // Request again with larger amount
    client.request_emergency_withdraw(&admin, &pool_id, &token, &999_999_999i128);
    env.ledger().set_timestamp(GRACE_PERIOD_SECS * 2 + 1);
    client.execute_emergency_withdraw(&pool_id);
}

/// Test 5: Emergency withdrawal with non-zero starting timestamp
#[test]
fn test_emergency_withdrawal_nonzero_start_timestamp() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 500_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "NonZero Start Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    env.ledger().set_timestamp(1_000_000);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &500_000_000i128);

    env.ledger().set_timestamp(1_000_000 + GRACE_PERIOD_SECS + 1);

    client.execute_emergency_withdraw(&pool_id);

    let withdrawal_key = (Symbol::new(&env, "emergency_withdraw"), pool_id);
    let has_request = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(!has_request);
}

/// Test 6: Multiple pools can have independent emergency withdrawals
#[test]
fn test_emergency_withdrawal_multiple_pools_independent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = create_token(&env, 1_000_000_000i128, &contract_id);

    client.set_admin(&admin);

    let pool1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "First"),
        &1_000_000_000u128,
        &100_000u64,
    );
    let pool2 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Second"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.request_emergency_withdraw(&admin, &pool1, &token, &100_000_000i128);
    client.request_emergency_withdraw(&admin, &pool2, &token, &200_000_000i128);

    env.ledger().set_timestamp(GRACE_PERIOD_SECS + 1);

    client.execute_emergency_withdraw(&pool1);
    client.execute_emergency_withdraw(&pool2);

    let key1 = (Symbol::new(&env, "emergency_withdraw"), pool1);
    let key2 = (Symbol::new(&env, "emergency_withdraw"), pool2);
    let has1 = env.as_contract(&contract_id, || env.storage().persistent().has(&key1));
    let has2 = env.as_contract(&contract_id, || env.storage().persistent().has(&key2));
    assert!(!has1 && !has2);
}
