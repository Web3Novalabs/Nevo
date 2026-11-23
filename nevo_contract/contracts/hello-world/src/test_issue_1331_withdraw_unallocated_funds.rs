#![cfg(test)]
//! withdraw_unallocated_funds authorization and amount — issue #1331.
//!
//! Covers withdraw_unallocated_funds:
//!   (1) The pool sponsor can invoke it successfully.
//!   (2) Any other caller fails authorization.
//!   (3) Only the unallocated portion is withdrawn, not allocated/claimed funds.
//!   (4) The surplus is transferred to the caller in the pool token.
//!   (5) It fails when no unallocated funds remain.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token::{StellarAssetClient, TokenClient},
    Address, Env, IntoVal, String, Symbol,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

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
            &(index, student.clone(), String::from_str(env, "application")),
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
                application_data: String::from_str(env, "application"),
                status,
                approved_amount,
                amount_claimed,
            },
        );

        let count_key = (Symbol::new(env, APPLICATION_COUNT_PREFIX), pool_id);
        env.storage().persistent().set(&count_key, &index);
    });
}

/// A pool with `collected` donated and the same amount of `token` held by the
/// contract. Returns (client, contract_id, pool_id, sponsor, token).
fn funded_pool(env: &Env, collected: u128) -> (ContractClient<'_>, Address, u32, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let sponsor = Address::generate(env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(env, "Unallocated Funds"),
        &String::from_str(env, "Tests for withdraw_unallocated_funds"),
        &1_000_000_000u128,
        &200_000u64,
    );
    client.donate(&pool_id, &sponsor, &collected);

    let token = create_token(env, collected as i128, &contract_id);
    (client, contract_id, pool_id, sponsor, token)
}

// ── (1) The pool sponsor can invoke it ──────────────────────────────────────

#[test]
fn test_sponsor_can_withdraw_unallocated_funds() {
    let env = Env::default();
    let (client, contract_id, pool_id, sponsor, token) = funded_pool(&env, 100_000_000);

    client
        .mock_auths(&[MockAuth {
            address: &sponsor,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "withdraw_unallocated_funds",
                args: (&pool_id, &token).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .withdraw_unallocated_funds(&pool_id, &token);

    assert_eq!(client.get_pool(&pool_id).3, 0u128);
}

// ── (2) Unauthorized callers fail ───────────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_non_sponsor_cannot_withdraw_unallocated_funds() {
    let env = Env::default();
    let (client, contract_id, pool_id, _sponsor, token) = funded_pool(&env, 100_000_000);
    let attacker = Address::generate(&env);

    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "withdraw_unallocated_funds",
                args: (&pool_id, &token).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .withdraw_unallocated_funds(&pool_id, &token);
}

#[test]
fn test_failed_unauthorized_withdrawal_leaves_funds_untouched() {
    let env = Env::default();
    let (client, contract_id, pool_id, _sponsor, token) = funded_pool(&env, 100_000_000);
    let attacker = Address::generate(&env);

    let result = client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "withdraw_unallocated_funds",
                args: (&pool_id, &token).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_withdraw_unallocated_funds(&pool_id, &token);

    assert!(result.is_err());
    assert_eq!(client.get_pool(&pool_id).3, 100_000_000u128);
    assert_eq!(TokenClient::new(&env, &token).balance(&contract_id), 100_000_000i128);
    assert_eq!(TokenClient::new(&env, &token).balance(&attacker), 0i128);
}

// ── (3) + (4) Only the unallocated portion moves, to the caller ─────────────

#[test]
fn test_withdraws_only_unallocated_portion_and_transfers_it_to_sponsor() {
    let env = Env::default();
    let (client, contract_id, pool_id, sponsor, token) = funded_pool(&env, 100_000_000);

    // Approved: 80M approved, 20M already claimed -> 60M still allocated.
    seed_application(&env, &contract_id, pool_id, 1, &Address::generate(&env), ApplicationStatus::Approved, 80_000_000, 20_000_000);
    // Pending: 10M reserved for a pending decision.
    seed_application(&env, &contract_id, pool_id, 2, &Address::generate(&env), ApplicationStatus::Pending, 10_000_000, 0);
    // Rejected: never locks funds.
    seed_application(&env, &contract_id, pool_id, 3, &Address::generate(&env), ApplicationStatus::Rejected, 50_000_000, 0);

    let token_client = TokenClient::new(&env, &token);
    assert_eq!(token_client.balance(&sponsor), 0);

    client.withdraw_unallocated_funds(&pool_id, &token);

    // 100M collected - (60M + 10M) allocated = 30M surplus.
    assert_eq!(token_client.balance(&sponsor), 30_000_000i128, "sponsor receives the surplus");
    assert_eq!(token_client.balance(&contract_id), 70_000_000i128, "allocated funds stay in the contract");
    assert_eq!(client.get_pool(&pool_id).3, 70_000_000u128, "collected drops by the surplus only");
}

// ── (5) Nothing left to withdraw ────────────────────────────────────────────

#[test]
#[should_panic(expected = "No surplus to withdraw")]
fn test_fails_when_all_funds_are_allocated() {
    let env = Env::default();
    let (client, contract_id, pool_id, _sponsor, token) = funded_pool(&env, 60_000_000);

    seed_application(&env, &contract_id, pool_id, 1, &Address::generate(&env), ApplicationStatus::Approved, 60_000_000, 0);

    client.withdraw_unallocated_funds(&pool_id, &token);
}

#[test]
#[should_panic(expected = "No surplus to withdraw")]
fn test_second_withdrawal_fails_once_surplus_is_drained() {
    let env = Env::default();
    let (client, _contract_id, pool_id, _sponsor, token) = funded_pool(&env, 100_000_000);

    client.withdraw_unallocated_funds(&pool_id, &token);
    client.withdraw_unallocated_funds(&pool_id, &token);
}
