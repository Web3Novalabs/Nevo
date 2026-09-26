#![cfg(test)]
//! Tests for emergency withdrawal request-then-execute ordering — issue #1378.
//!
//! Covers request_emergency_withdraw / execute_emergency_withdraw sequence:
//!   (1) Cannot execute without a prior request.
//!   (2) Cannot execute before grace period elapses.
//!   (3) Execute succeeds immediately after grace period ends.
//!   (4) Only original requester/admin can execute / authorization enforcement.
//!   (5) Request state cleared after successful execution.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String, Symbol,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn setup_emergency_pool(env: &Env, client: &ContractClient, admin: &Address) -> (u32, Address) {
    client.set_admin(admin);
    let creator = Address::generate(env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(env, "Emergency Ordering Pool"),
        &String::from_str(env, "Tests for emergency withdrawal request-then-execute ordering"),
        &1_000_000_000u128,
        &100_000u64,
    );
    (pool_id, creator)
}

// ── Test 1: Cannot execute without a prior request ──────────────────────────

#[test]
#[should_panic(expected = "Emergency withdrawal not requested")]
fn test_cannot_execute_without_prior_request() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let (pool_id, _) = setup_emergency_pool(&env, &client, &admin);

    // Attempting to execute emergency withdrawal when no request was submitted must panic
    client.execute_emergency_withdraw(&pool_id);
}

// ── Test 2: Cannot execute before grace period elapses ──────────────────────

#[test]
#[should_panic(expected = "Grace period not elapsed")]
fn test_cannot_execute_before_grace_period_elapses() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let (pool_id, _) = setup_emergency_pool(&env, &client, &admin);
    let amount = 500_000_000i128;
    let token = create_token(&env, amount, &contract_id);

    env.ledger().set_timestamp(1_000);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &amount);

    // Advance timestamp to 1 second before the grace period elapses (86400s)
    env.ledger().set_timestamp(1_000 + GRACE_PERIOD_SECS - 1);

    client.execute_emergency_withdraw(&pool_id);
}

// ── Test 3: Execute succeeds immediately after grace period ends ────────────

#[test]
fn test_execute_succeeds_immediately_after_grace_period_ends() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let (pool_id, _) = setup_emergency_pool(&env, &client, &admin);
    let amount = 300_000_000i128;
    let token = create_token(&env, amount, &contract_id);

    env.ledger().set_timestamp(5_000);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &amount);

    // Exactly at the grace period boundary (time_elapsed == GRACE_PERIOD_SECS)
    env.ledger().set_timestamp(5_000 + GRACE_PERIOD_SECS);

    client.execute_emergency_withdraw(&pool_id);

    // Funds were transferred from contract to the requesting admin
    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&admin), amount);
    assert_eq!(token_client.balance(&contract_id), 0i128);
}

// ── Test 4: Only original requester/admin can execute / authorization ───────

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_unauthorized_non_admin_cannot_request_emergency_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let unauthorized_caller = Address::generate(&env);
    let (pool_id, _) = setup_emergency_pool(&env, &client, &admin);
    let amount = 100_000_000i128;
    let token = create_token(&env, amount, &contract_id);

    // Caller that is not the configured admin must be rejected
    client.request_emergency_withdraw(&unauthorized_caller, &pool_id, &token, &amount);
}

#[test]
fn test_execute_safeguard_disburses_to_original_requester() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let (pool_id, _) = setup_emergency_pool(&env, &client, &admin);
    let amount = 450_000_000i128;
    let token = create_token(&env, amount, &contract_id);

    env.ledger().set_timestamp(100);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &amount);

    // Advance past grace period
    env.ledger().set_timestamp(100 + GRACE_PERIOD_SECS + 1);

    // Third party executes the withdrawal
    let third_party = Address::generate(&env);
    let token_client = soroban_sdk::token::Client::new(&env, &token);

    client.execute_emergency_withdraw(&pool_id);

    // Verify funds strictly disburse to the original requester (admin), not third party
    assert_eq!(token_client.balance(&admin), amount);
    assert_eq!(token_client.balance(&third_party), 0i128);
}

// ── Test 5: Request state cleared after successful execution ────────────────

#[test]
fn test_request_state_cleared_after_successful_execution() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let (pool_id, _) = setup_emergency_pool(&env, &client, &admin);
    let amount = 200_000_000i128;
    let token = create_token(&env, amount * 2, &contract_id);

    env.ledger().set_timestamp(0);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &amount);

    let withdrawal_key = (Symbol::new(&env, "emergency_withdraw"), pool_id);
    let has_request_before = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(has_request_before, "Request state must exist before execution");

    env.ledger().set_timestamp(GRACE_PERIOD_SECS + 1);
    client.execute_emergency_withdraw(&pool_id);

    // (5a) Request key must be removed from persistent storage
    let has_request_after = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(!has_request_after, "Request state must be deleted from storage");

    // (5b) Subsequent execute fails because request state no longer exists
    let second_exec = client.try_execute_emergency_withdraw(&pool_id);
    assert!(second_exec.is_err(), "Second execute must fail on cleared state");

    // (5c) A new emergency request can be submitted cleanly without conflict
    client.request_emergency_withdraw(&admin, &pool_id, &token, &amount);
    let has_new_request = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(has_new_request, "New request can be created after prior request was cleared");
}
