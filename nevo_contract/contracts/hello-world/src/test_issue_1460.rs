#![cfg(test)]

//! Issue #1460 — `claim_funds` takes a caller-supplied `token_address` and
//! never validates it against `POOL_TOKEN_PREFIX`, unlike `donate_with_token`
//! and `refund_donation` which both enforce the stored pool token.

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn approved(env: &Env) -> String {
    String::from_str(env, "Approved")
}

// =============================================================================
// Test A — claim_funds accepts any token, ignoring the configured pool token
// =============================================================================

/// Bug #1460, part 1.
///
/// Pool is configured for token A, donations arrive in A, but the student
/// claims in token B. `claim_funds` never reads `POOL_TOKEN_PREFIX`, so the
/// transfer goes through against token B's balance of the contract.
#[test]
fn test_claim_funds_ignores_configured_pool_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    // Both tokens are minted to the contract so both transfer paths can succeed.
    let token_a = create_token(&env, 1_000_000, &contract_id);
    let token_b = create_token(&env, 1_000_000, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1460 Pool"),
        &String::from_str(&env, "Token mismatch on claim"),
        &1_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token_a);
    client.donate_with_token(&pool_id, &creator, &token_a, &500_000i128);
    client.set_application_status(&pool_id, &student, &approved(&env));

    // Claim using token B — different from the configured pool token (A).
    client.claim_funds(&student, &pool_id, &100_000i128, &token_b);

    // The student received B. Token A is untouched — the transfer succeeded
    // against a completely unrelated token.
    let student_bal_b =
        soroban_sdk::token::Client::new(&env, &token_b).balance(&student);
    assert_eq!(student_bal_b, 99_000); // 100_000 minus 1% fee

    let student_bal_a =
        soroban_sdk::token::Client::new(&env, &token_a).balance(&student);
    assert_eq!(student_bal_a, 0);
}

// =============================================================================
// Test B — refund_donation rejects a mismatched token (the inconsistency)
// =============================================================================

/// Bug #1460, part 2.
///
/// Same pool state as Test A, but calling `refund_donation` with token B
/// panics with `"TokenTransferFailed"` — proving `refund_donation` *does*
/// validate the stored pool token while `claim_funds` does not.
#[test]
#[should_panic(expected = "TokenTransferFailed")]
fn test_refund_donation_rejects_mismatched_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token_a = create_token(&env, 1_000_000, &donor);
    let token_b = create_token(&env, 1_000_000, &creator);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1460 Refund Pool"),
        &String::from_str(&env, "Refund token check"),
        &1_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token_a);
    client.donate_with_token(&pool_id, &donor, &token_a, &500_000i128);

    // Advance past the deadline + grace period so refund is allowed.
    let deadline = 10_000u32;
    client.set_pool_deadline(&pool_id, &deadline);
    env.ledger()
        .with_mut(|l| l.sequence_number = deadline + REFUND_GRACE_PERIOD_LEDGERS + 1);

    // Refund in token B — mismatches the stored token A, so refund panics.
    client.refund_donation(&pool_id, &donor, &token_b);
}

// =============================================================================
// Test C — side-by-side: claim_funds vs donate_with_token on the same pool
// =============================================================================

/// Bug #1460, part 3 (the "enumerate the inconsistency" part of the issue).
///
/// Donations in token B are rejected by `donate_with_token` because the
/// stored token is A. But a claim in token B by an approved student
/// *succeeds*. Same pool, same stored token, opposite outcomes.
#[test]
fn test_inconsistency_between_donation_and_claim_token_checks() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let student = Address::generate(&env);
    let token_a = create_token(&env, 1_000_000, &contract_id);
    let token_b = create_token(&env, 1_000_000, &contract_id);
    // Give the donor B so they could attempt a B donation.
    let _ = create_token(&env, 1_000_000, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "1460 Inconsistency"),
        &String::from_str(&env, "Same pool, two token policies"),
        &1_000_000u128,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token_a);
    client.donate_with_token(&pool_id, &creator, &token_a, &500_000i128);
    client.set_application_status(&pool_id, &student, &approved(&env));

    // Donate in token B: rejected because stored token is A.
    let donate_b = client.try_donate_with_token(&pool_id, &donor, &token_b, &1_000i128);
    assert!(donate_b.is_err(), "donate_with_token must reject token B");

    // Claim in token B: accepted by claim_funds.
    client.claim_funds(&student, &pool_id, &100_000i128, &token_b);

    let student_bal_b =
        soroban_sdk::token::Client::new(&env, &token_b).balance(&student);
    assert_eq!(student_bal_b, 99_000);
}