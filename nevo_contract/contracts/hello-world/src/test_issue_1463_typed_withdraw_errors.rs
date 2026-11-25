#![cfg(test)]
//! Tests for issue #1463: `withdraw_unallocated_funds` must raise typed
//! `ContractError` variants for the insolvency and zero-surplus cases, not
//! string panics.
//!
//! Coverage:
//!   1. `locked > collected` → `ContractError::Insolvency` (#37)
//!   2. `locked == collected` → `ContractError::NoSurplusToWithdraw` (#38)

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    StellarAssetClient::new(env, &token.address()).mint(recipient, &amount);
    token.address()
}

/// Write an `Application` record and the matching application index so that
/// `withdraw_unallocated_funds` sees `approved_amount` as locked funds.
fn seed_application(
    env: &Env,
    contract_id: &Address,
    pool_id: u32,
    index: u32,
    student: &Address,
    status: ApplicationStatus,
    approved_amount: i128,
    amount_claimed: i128,
) {
    env.as_contract(contract_id, || {
        let app_key = (Symbol::new(env, APPLICATION_PREFIX), pool_id, index);
        env.storage().persistent().set(
            &app_key,
            &(index, student.clone(), String::from_str(env, "app")),
        );

        let claim_key = (
            Symbol::new(env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage().persistent().set(
            &claim_key,
            &Application {
                pool_id,
                student: student.clone(),
                application_data: String::from_str(env, "app"),
                status,
                approved_amount,
                amount_claimed,
            },
        );

        let count_key = (Symbol::new(env, APPLICATION_COUNT_PREFIX), pool_id);
        env.storage().persistent().set(&count_key, &index);
    });
}

/// Register the contract, create a pool, and donate `collected` into it in
/// a freshly-minted token. Returns (client, contract_id, pool_id, sponsor, token).
fn funded_pool(
    env: &Env,
    collected: i128,
) -> (ContractClient<'_>, Address, u32, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let sponsor = Address::generate(env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(env, "Issue #1463 Pool"),
        &String::from_str(env, "typed withdraw errors"),
        &1_000_000_000i128,
        &200_000u64,
    );

    let token = create_token(env, collected, &sponsor);
    client.set_pool_token(&pool_id, &token);
    client.donate_with_token(&pool_id, &sponsor, &token, &collected);

    (client, contract_id, pool_id, sponsor, token)
}

// ── (1) Insolvency: locked > collected ──────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #37)")]
fn insolvency_panics_with_typed_error() {
    let env = Env::default();
    let (client, contract_id, pool_id, _sponsor, token) = funded_pool(&env, 500);

    // Approved for 10_000 while only 500 is collected → locked exceeds collected.
    seed_application(
        &env,
        &contract_id,
        pool_id,
        1,
        &Address::generate(&env),
        ApplicationStatus::Approved,
        10_000,
        0,
    );

    client.withdraw_unallocated_funds(&pool_id, &token);
}

// ── (2) Zero surplus: locked == collected ───────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #38)")]
fn no_surplus_panics_with_typed_error() {
    let env = Env::default();
    let (client, contract_id, pool_id, _sponsor, token) = funded_pool(&env, 500);

    // Fully allocated: approved == collected → surplus == 0.
    seed_application(
        &env,
        &contract_id,
        pool_id,
        1,
        &Address::generate(&env),
        ApplicationStatus::Approved,
        500,
        0,
    );

    client.withdraw_unallocated_funds(&pool_id, &token);
}