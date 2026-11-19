#![cfg(test)]

// ============= ISSUE #1343: `get_claimed_amount` RUNNING TOTAL TESTS =============

use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, BytesN, Env, String};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// Sets up a pool linked to a registered school, funds it (accounting +
/// real token balance held by the contract), and approves a student's
/// application so `claim_funds` can be exercised. Returns the client, pool
/// id, student, and the funding token address the contract actually holds
/// a balance of (must be reused by callers when invoking `claim_funds`).
fn setup_claimable_pool(
    env: &Env,
    collected: u128,
) -> (ContractClient<'_>, u32, Address, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);

    let school = Address::generate(env);
    let metadata_hash = BytesN::from_array(env, &[6u8; 32]);
    client.register_school(&school, &metadata_hash);

    let creator = Address::generate(env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(env, "Claim Pool"),
        &String::from_str(env, "Description"),
        &collected,
        &school,
        &200_000u64,
    );

    let sponsor = Address::generate(env);
    client.donate(&pool_id, &sponsor, &collected);
    let token = create_token(env, collected as i128, &contract_id);

    let student = Address::generate(env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(env, "application"));
    client.approve_application(&pool_id, &school, &student, &true);

    (client, pool_id, student, token)
}

/// `get_claimed_amount` returns 0 before any `claim_funds` call.
#[test]
fn test_claimed_amount_zero_before_any_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student, _) = setup_claimable_pool(&env, 1_000_000u128);

    assert_eq!(client.get_claimed_amount(&pool_id, &student), 0i128);
}

/// `get_claimed_amount` accumulates the gross claim amount (not net of the
/// 1% protocol fee) across successive successful claims.
#[test]
fn test_claimed_amount_increments_after_each_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student, token) = setup_claimable_pool(&env, 1_000_000u128);

    client.claim_funds(&student, &pool_id, &100_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 100_000i128);

    client.claim_funds(&student, &pool_id, &50_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 150_000i128);
}

/// A claim that would push the running total past the pool's collected
/// amount panics, and the running total is left unchanged (the failed
/// call's storage writes are not persisted).
#[test]
fn test_claimed_amount_never_exceeds_collected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student, token) = setup_claimable_pool(&env, 1_000_000u128);

    client.claim_funds(&student, &pool_id, &900_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 900_000i128);

    let res = client.try_claim_funds(&student, &pool_id, &200_000i128, &token);
    assert!(res.is_err(), "Overdraw attempt must fail");
    assert_eq!(
        client.get_claimed_amount(&pool_id, &student),
        900_000i128,
        "Running total must be unchanged after a failed overdraw"
    );
}

/// The running total is tracked independently per (pool_id, student) pair.
#[test]
fn test_claimed_amount_isolated_per_pool_and_student() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id_a, student_a, token_a) = setup_claimable_pool(&env, 1_000_000u128);
    client.claim_funds(&student_a, &pool_id_a, &100_000i128, &token_a);

    let (client_b, pool_id_b, student_b, token_b) = setup_claimable_pool(&env, 2_000_000u128);
    client_b.claim_funds(&student_b, &pool_id_b, &300_000i128, &token_b);

    assert_eq!(client.get_claimed_amount(&pool_id_a, &student_a), 100_000i128);
    // Different student in the same pool has no claims recorded.
    assert_eq!(client.get_claimed_amount(&pool_id_a, &student_b), 0i128);
    assert_eq!(
        client_b.get_claimed_amount(&pool_id_b, &student_b),
        300_000i128
    );
}

/// Large i128 claim amounts are reflected exactly, with no truncation or
/// precision loss.
#[test]
fn test_claimed_amount_precision_for_large_amounts() {
    let env = Env::default();
    env.mock_all_auths();
    let large_collected: u128 = 90_000_000_000_000_000u128;
    let (client, pool_id, student, token) = setup_claimable_pool(&env, large_collected);

    let claim_amount: i128 = 70_000_000_000_000_000i128;
    client.claim_funds(&student, &pool_id, &claim_amount, &token);

    assert_eq!(client.get_claimed_amount(&pool_id, &student), claim_amount);
}
