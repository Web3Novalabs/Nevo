#![cfg(test)]

// ============= ISSUE #1322: ERROR-RECOVERY SCENARIOS =============
//
// The five properties the issue asks about, one test each:
//   1. failed operations do not corrupt state
//   2. partial failures are handled cleanly
//   3. the system is recoverable after an error
//   4. rollback mechanisms work
//   5. graceful degradation is possible

use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

fn create_pool(env: &Env, client: &ContractClient, goal: u128) -> (u32, Address) {
    let sponsor = Address::generate(env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(env, "Recovery Pool"),
        &String::from_str(env, "Pool description"),
        &goal,
        &200_000u64,
    );
    (pool_id, sponsor)
}

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// Pool with an approved student and a real token balance, so `claim_funds`
/// can be exercised (mirrors the #1343 fixture).
fn setup_claimable_pool(env: &Env, collected: u128) -> (ContractClient<'_>, u32, Address, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);

    let school = Address::generate(env);
    client.register_school(&school, &soroban_sdk::BytesN::from_array(env, &[7u8; 32]));

    let creator = Address::generate(env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(env, "Recovery Claim Pool"),
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

/// (1) A rejected donation must not move the running total, the donor count or
/// the donor's own contribution.
#[test]
fn test_failed_donation_does_not_corrupt_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _sponsor) = create_pool(&env, &client, 1_000_000u128);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.donate(&pool_id, &alice, &100_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 100_000u128);

    // A cancelled pool rejects donations.
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    let rejected = client.try_donate(&pool_id, &bob, &50_000u128);
    assert!(
        rejected.is_err(),
        "donation into a cancelled pool must fail"
    );

    assert_eq!(
        client.get_total_raised(&pool_id),
        100_000u128,
        "total must not absorb the rejected amount"
    );
    assert_eq!(
        client.get_donor_count(&pool_id),
        1u32,
        "the rejected donor must not be counted"
    );
    assert_eq!(
        client.get_contribution(&pool_id, &bob),
        0u128,
        "the rejected donor must have no contribution recorded"
    );
    assert_eq!(client.get_contribution(&pool_id, &alice), 100_000u128);
}

/// (2) When one donor's payment fails, the donations that already succeeded are
/// untouched and remain individually correct.
#[test]
fn test_partial_failure_leaves_completed_donations_intact() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _sponsor) = create_pool(&env, &client, 1_000_000u128);
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    client.donate(&pool_id, &alice, &100_000u128);
    client.donate(&pool_id, &bob, &200_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 300_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 2u32);

    // The third payment fails; the first two must stay exactly as they were.
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    assert!(client.try_donate(&pool_id, &bob, &25_000u128).is_err());

    assert_eq!(client.get_total_raised(&pool_id), 300_000u128);
    assert_eq!(client.get_contribution(&pool_id, &alice), 100_000u128);
    assert_eq!(
        client.get_contribution(&pool_id, &bob),
        200_000u128,
        "a failed top-up must not be added to an existing contribution"
    );
    assert_eq!(client.get_donor_count(&pool_id), 2u32);
}

/// (3) After an operation is rejected the contract keeps working: the same pool
/// accepts donations again once it is active.
#[test]
fn test_system_recovers_after_a_rejected_operation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, _sponsor) = create_pool(&env, &client, 1_000_000u128);
    let alice = Address::generate(&env);

    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    assert!(client.try_donate(&pool_id, &alice, &10_000u128).is_err());
    assert_eq!(client.get_total_raised(&pool_id), 0u128);

    // Back to active: the pool is usable, and the earlier failure left nothing behind.
    client.set_pool_state(&pool_id, &PoolState::Active);
    client.donate(&pool_id, &alice, &25_000u128);

    assert_eq!(client.get_total_raised(&pool_id), 25_000u128);
    assert_eq!(client.get_contribution(&pool_id, &alice), 25_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1u32);
}

/// (4) A claim that overdraws the pool must roll back completely: the claimed
/// amount stays where it was and the exact remaining amount can still be taken.
#[test]
fn test_failed_claim_rolls_back_the_claimed_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, student, token) = setup_claimable_pool(&env, 1_000_000u128);

    client.claim_funds(&student, &pool_id, &900_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 900_000i128);

    let overdraw = client.try_claim_funds(&student, &pool_id, &200_000i128, &token);
    assert!(overdraw.is_err(), "claiming past the pool must fail");
    assert_eq!(
        client.get_claimed_amount(&pool_id, &student),
        900_000i128,
        "the failed claim must not advance the claimed amount"
    );

    // The remaining 100k is still claimable — proof the rollback was complete.
    client.claim_funds(&student, &pool_id, &100_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 1_000_000i128);
}

/// (5) With a pool closed, writes are rejected cleanly while reads keep serving
/// the frozen state — and a repeated close is refused, not silently applied.
#[test]
fn test_reads_degrade_gracefully_when_writes_are_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let (pool_id, sponsor) = create_pool(&env, &client, 1_000_000u128);
    let alice = Address::generate(&env);
    client.donate(&pool_id, &alice, &150_000u128);

    // A pool can only be closed once disbursed or cancelled.
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    // Writes are refused...
    assert!(client.try_donate(&pool_id, &alice, &10_000u128).is_err());
    assert!(
        client.try_close_pool(&pool_id).is_err(),
        "closing an already closed pool must be refused"
    );

    // ...while every read still answers with the frozen values.
    let (id, returned_sponsor, _goal, collected, is_closed, _last) = client.get_pool(&pool_id);
    assert_eq!(id, pool_id);
    assert_eq!(returned_sponsor, sponsor);
    assert_eq!(collected, 150_000u128);
    assert!(is_closed, "the pool must report itself closed");
    assert_eq!(client.get_total_raised(&pool_id), 150_000u128);
    assert_eq!(client.get_contribution(&pool_id, &alice), 150_000u128);
    assert!(client.get_all_campaigns().contains(pool_id));
}
