#![cfg(test)]

//! Configured pool tokens apply to donations, claims, and refunds.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
    Address, BytesN, Env, String,
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
// Test A — claim_funds enforces the configured pool token
// =============================================================================

#[test]
fn test_claim_funds_enforces_configured_pool_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    // Fund both tokens so the error specifically verifies token validation.
    let token_a = create_token(&env, 1_000_000, &contract_id);
    let token_b = create_token(&env, 1_000_000, &contract_id);
    // The donor needs A to fund the pool through donate_with_token.
    StellarAssetClient::new(&env, &token_a).mint(&creator, &500_000i128);

    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(&env, &[0u8; 32]));
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "1460 Pool"),
        &String::from_str(&env, "Token mismatch on claim"),
        &1_000_000u128,
        &school,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token_a);
    client.donate_with_token(&pool_id, &creator, &token_a, &500_000i128);
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "application"),
    );
    client.set_application_status(&admin, &pool_id, &student, &approved(&env));

    // Claim using token B — different from the configured pool token (A).
    assert_eq!(
        client.try_claim_funds(&student, &pool_id, &100_000i128, &token_b),
        Err(Ok(ContractError::TokenTransferFailed.into()))
    );

    let student_bal_b =
        soroban_sdk::token::Client::new(&env, &token_b).balance(&student);
    assert_eq!(student_bal_b, 0);

    let student_bal_a =
        soroban_sdk::token::Client::new(&env, &token_a).balance(&student);
    assert_eq!(student_bal_a, 0);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 0);

    client.claim_funds(&student, &pool_id, &100_000i128, &token_a);
    assert_eq!(soroban_sdk::token::Client::new(&env, &token_a).balance(&student), 99_000);
}

// =============================================================================
// Test B — refund_donation rejects a mismatched token
// =============================================================================

#[test]
#[should_panic(expected = "TokenTransferFailed")]
fn test_refund_donation_rejects_mismatched_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let token_a = create_token(&env, 1_000_000, &donor);
    let token_b = create_token(&env, 1_000_000, &creator);

    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(&env, &[0u8; 32]));
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "1460 Refund Pool"),
        &String::from_str(&env, "Refund token check"),
        &1_000_000u128,
        &school,
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

#[test]
fn test_donation_and_claim_reject_the_same_mismatched_token() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let student = Address::generate(&env);
    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let token_a = create_token(&env, 1_000_000, &contract_id);
    let token_b = create_token(&env, 1_000_000, &contract_id);
    // Give the donor B so they could attempt a B donation.
    StellarAssetClient::new(&env, &token_b).mint(&donor, &1_000i128);
    // The donor of A also needs A to fund the pool.
    StellarAssetClient::new(&env, &token_a).mint(&creator, &500_000i128);

    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(&env, &[0u8; 32]));
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "1460 Inconsistency"),
        &String::from_str(&env, "Same pool, two token policies"),
        &1_000_000u128,
        &school,
        &100_000u64,
    );

    client.set_pool_token(&pool_id, &token_a);
    client.donate_with_token(&pool_id, &creator, &token_a, &500_000i128);
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "application"),
    );
    client.set_application_status(&admin, &pool_id, &student, &approved(&env));

    // Donate in token B: rejected because stored token is A.
    let donate_b = client.try_donate_with_token(&pool_id, &donor, &token_b, &1_000i128);
    assert!(donate_b.is_err(), "donate_with_token must reject token B");

    assert_eq!(
        client.try_claim_funds(&student, &pool_id, &100_000i128, &token_b),
        Err(Ok(ContractError::TokenTransferFailed.into()))
    );

    let student_bal_b =
        soroban_sdk::token::Client::new(&env, &token_b).balance(&student);
    assert_eq!(student_bal_b, 0);
}
