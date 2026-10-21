#![cfg(test)]
//! Tests for `request_emergency_withdraw` amount parameter validation — issue #1487.
//!
//! `request_emergency_withdraw` accepts an arbitrary `amount: i128` and stores it
//! directly in the `EmergencyWithdrawalRequest` without checking against
//! `pool.collected`, the contract's actual token balance, or `amount <= 0`.
//!
//! Covers:
//!   (1) Requesting an emergency withdrawal amount larger than `pool.collected`
//!       is accepted by `request_emergency_withdraw` (storing an unenforceable
//!       request), while `execute_emergency_withdraw`'s later token transfer
//!       is what eventually fails when the contract balance cannot cover it.
//!   (2) Requesting with `amount <= 0` (`0` and negative amounts), which has
//!       no explicit validation in `request_emergency_withdraw`.

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

fn setup_pool_with_admin(env: &Env, client: &ContractClient) -> (u32, Address, Address) {
    let admin = Address::generate(env);
    client.set_admin(&admin);

    let creator = Address::generate(env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(env, "Emergency Amount Test Pool"),
        &String::from_str(env, "Tests for issue #1487 amount validation"),
        &1_000_000_000u128,
        &100_000u64,
    );
    (pool_id, admin, creator)
}

// ── (1) Amount larger than pool.collected ───────────────────────────────────

/// `request_emergency_withdraw` accepts an amount exceeding `pool.collected`
/// (and the contract's actual token balance), storing an unenforceable
/// `EmergencyWithdrawalRequest`. When `execute_emergency_withdraw` is later
/// invoked after the grace period, the underlying token transfer fails and
/// the request remains in storage.
#[test]
fn test_request_emergency_withdraw_amount_exceeding_collected_accepted_then_execute_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, admin, _) = setup_pool_with_admin(&env, &client);
    let donor = Address::generate(&env);

    let collected_amount: i128 = 100_000_000;
    let token = create_token(&env, collected_amount, &donor);
    client.donate_with_token(&pool_id, &donor, &token, &collected_amount);

    assert_eq!(client.get_pool(&pool_id).3, collected_amount as u128);

    // Request an amount larger than both pool.collected and contract token balance.
    let excessive_amount: i128 = 500_000_000;
    env.ledger().set_timestamp(1_000);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &excessive_amount);

    // Confirm request_emergency_withdraw accepted and stored the unenforceable request.
    let withdrawal_key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
    let stored_request: EmergencyWithdrawalRequest = env
        .as_contract(&contract_id, || {
            env.storage().persistent().get(&withdrawal_key)
        })
        .expect("Request should be stored despite exceeding pool.collected");
    assert_eq!(stored_request.amount, excessive_amount);
    assert_eq!(stored_request.pool_id, pool_id);
    assert_eq!(stored_request.requested_by, admin);

    // Advance past the grace period and confirm execute_emergency_withdraw fails
    // on the token transfer due to insufficient contract token balance.
    env.ledger().set_timestamp(1_000 + GRACE_PERIOD_SECS + 1);
    let exec_result = client.try_execute_emergency_withdraw(&pool_id);
    assert!(
        exec_result.is_err(),
        "execute_emergency_withdraw must fail when requested amount exceeds contract token balance"
    );

    // Because execution panicked during token transfer, the request remains stored
    // and the contract's token balance is untouched.
    let still_stored = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(still_stored, "Unenforceable request remains in storage after failed execution");

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&contract_id), collected_amount);
    assert_eq!(token_client.balance(&admin), 0i128);
}

/// Even when `pool.collected` is 0, `request_emergency_withdraw` does not check
/// `pool.collected` and accepts any amount; if the contract happens to hold
/// enough tokens (e.g. transferred directly outside `donate_with_token`),
/// `execute_emergency_withdraw` succeeds because only the token transfer
/// enforces balance availability.
#[test]
fn test_request_emergency_withdraw_exceeding_zero_collected_succeeds_if_contract_holds_tokens() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, admin, _) = setup_pool_with_admin(&env, &client);
    assert_eq!(client.get_pool(&pool_id).3, 0u128);

    let unrecorded_balance: i128 = 250_000_000;
    let token = create_token(&env, unrecorded_balance, &contract_id);

    env.ledger().set_timestamp(2_000);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &unrecorded_balance);

    env.ledger().set_timestamp(2_000 + GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool_id);

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&admin), unrecorded_balance);
    assert_eq!(token_client.balance(&contract_id), 0i128);
}

// ── (2) Amount <= 0 (zero and negative amounts) ─────────────────────────────

/// `request_emergency_withdraw` has no explicit validation for `amount == 0`:
/// a zero-amount request is accepted and stored, and `execute_emergency_withdraw`
/// completes a 0-value token transfer after the grace period.
#[test]
fn test_request_emergency_withdraw_zero_amount_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, admin, _) = setup_pool_with_admin(&env, &client);
    let initial_balance: i128 = 100_000_000;
    let token = create_token(&env, initial_balance, &contract_id);

    env.ledger().set_timestamp(500);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &0i128);

    let withdrawal_key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
    let stored_request: EmergencyWithdrawalRequest = env
        .as_contract(&contract_id, || {
            env.storage().persistent().get(&withdrawal_key)
        })
        .expect("Zero-amount emergency withdrawal request should be stored");
    assert_eq!(stored_request.amount, 0i128);

    env.ledger().set_timestamp(500 + GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&pool_id);

    let has_request_after = env.as_contract(&contract_id, || {
        env.storage().persistent().has(&withdrawal_key)
    });
    assert!(!has_request_after);

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&contract_id), initial_balance);
    assert_eq!(token_client.balance(&admin), 0i128);
}

/// `request_emergency_withdraw` has no explicit validation for `amount < 0`:
/// a negative amount is accepted and stored in `EmergencyWithdrawalRequest`,
/// and only fails later during `execute_emergency_withdraw` when the Stellar
/// Asset Contract rejects a negative transfer amount.
#[test]
fn test_request_emergency_withdraw_negative_amount_accepted_then_execute_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, admin, _) = setup_pool_with_admin(&env, &client);
    let initial_balance: i128 = 100_000_000;
    let token = create_token(&env, initial_balance, &contract_id);

    let negative_amount: i128 = -50_000_000;
    env.ledger().set_timestamp(1_000);
    client.request_emergency_withdraw(&admin, &pool_id, &token, &negative_amount);

    let withdrawal_key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
    let stored_request: EmergencyWithdrawalRequest = env
        .as_contract(&contract_id, || {
            env.storage().persistent().get(&withdrawal_key)
        })
        .expect("Negative-amount emergency withdrawal request should be stored");
    assert_eq!(stored_request.amount, negative_amount);

    // Executing after the grace period fails in the token contract because
    // Stellar Asset Contract `transfer` rejects negative amounts.
    env.ledger().set_timestamp(1_000 + GRACE_PERIOD_SECS + 1);
    let exec_result = client.try_execute_emergency_withdraw(&pool_id);
    assert!(
        exec_result.is_err(),
        "execute_emergency_withdraw must fail when transferring a negative amount"
    );

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&contract_id), initial_balance);
    assert_eq!(token_client.balance(&admin), 0i128);
}
