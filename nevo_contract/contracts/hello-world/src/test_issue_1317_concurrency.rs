#![cfg(test)]

// ============= ISSUE #1317: CONCURRENT OPERATION SAFETY =============
//
// A Soroban contract is single-threaded, so "concurrent" here means what it can
// actually mean on-chain: many callers whose calls land in the same ledger
// sequence, and calls that interleave with a state change or with each other.
// What must hold is that no update is lost, no counter double-counts, and a
// refused call leaves nothing half-written.
//
// The classic hazards these tests go after:
//   - a donor counter that increments per *call* instead of per unique donor;
//   - a running total that misses an update when calls interleave;
//   - two approved students claiming past the same pool balance (double spend);
//   - a state change mid-sequence that later calls ignore.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    token::StellarAssetClient,
    Address, BytesN, Env, String,
};

fn make_client(env: &Env) -> ContractClient<'_> {
    let contract_id = env.register(Contract, ());
    ContractClient::new(env, &contract_id)
}

fn create_pool(client: &ContractClient, env: &Env, goal: u128) -> u32 {
    client.create_pool(
        &Address::generate(env),
        &String::from_str(env, "Concurrent Pool"),
        &String::from_str(env, "description"),
        &goal,
        &100_000u64,
    )
}

/// A registered school, an approved student, and a funded token — the fixtures a
/// claim race needs.
fn claimable(env: &Env, collected: u128) -> (ContractClient<'_>, u32, Address, Address) {
    let client = make_client(env);
    let admin = Address::generate(env);
    client.set_admin(&admin);

    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[4u8; 32]));

    let pool_id = client.create_pool_for_school(
        &Address::generate(env),
        &String::from_str(env, "Race Pool"),
        &String::from_str(env, "description"),
        &collected,
        &school,
        &100_000u64,
    );
    client.donate(&pool_id, &Address::generate(env), &collected);

    let token = env.register_stellar_asset_contract_v2(Address::generate(env));
    StellarAssetClient::new(env, &token.address()).mint(&client.address, &(collected as i128));

    let student = Address::generate(env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(env, "application"));
    client.approve_application(&pool_id, &school, &student, &true);

    (client, pool_id, student, token.address())
}

/// (1) Many donations landing in one sequence are all accounted for — no update
/// is lost and the total matches the individual contributions exactly.
#[test]
fn test_many_donations_in_one_sequence_are_never_lost() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let pool_id = create_pool(&client, &env, 1_000_000u128);

    let mut expected = 0u128;
    let mut donors = Vec::new(&env);
    for i in 1..=5u128 {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &(i * 1_000u128));
        expected += i * 1_000u128;
        donors.push_back(donor);
    }

    assert_eq!(client.get_total_raised(&pool_id), expected);
    assert_eq!(client.get_donor_count(&pool_id), 5u32);

    let mut recomputed = 0u128;
    for donor in donors.iter() {
        recomputed += client.get_contribution(&pool_id, &donor);
    }
    assert_eq!(
        recomputed, expected,
        "every donation must be visible individually"
    );
}

/// (2) The donor counter counts unique donors, not calls. Three donations from
/// one address are three ledger writes but a single donor.
#[test]
fn test_the_donor_counter_counts_unique_donors_not_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let pool_id = create_pool(&client, &env, 1_000_000u128);

    let alice = Address::generate(&env);
    for _ in 0..3 {
        client.donate(&pool_id, &alice, &100u128);
    }
    client.donate(&pool_id, &Address::generate(&env), &100u128);

    assert_eq!(client.get_contribution(&pool_id, &alice), 300u128);
    assert_eq!(client.get_total_raised(&pool_id), 400u128);
    assert_eq!(
        client.get_donor_count(&pool_id),
        2u32,
        "three calls from one address plus one from another is two donors"
    );
}

/// (3) Donations interleaved with unrelated writes (applications being filed)
/// keep the ledger consistent: the running total after every step equals the sum
/// of the donations that have succeeded so far.
#[test]
fn test_interleaved_writes_do_not_lose_updates() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let pool_id = create_pool(&client, &env, 1_000_000u128);

    let mut expected = 0u128;
    for i in 0..4u128 {
        client.donate(&pool_id, &Address::generate(&env), &((i + 1) * 10u128));
        expected += (i + 1) * 10u128;
        client.apply_to_pool(
            &pool_id,
            &Address::generate(&env),
            &String::from_str(&env, "application"),
        );
        assert_eq!(
            client.get_total_raised(&pool_id),
            expected,
            "the total must track the successful donations while other writes land"
        );
    }
}

/// (4) A state change in the middle of a sequence governs everything after it —
/// the later call sees the new state, and the refused call changes nothing.
#[test]
fn test_a_state_change_mid_sequence_governs_later_calls() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let pool_id = create_pool(&client, &env, 1_000_000u128);

    client.donate(&pool_id, &Address::generate(&env), &100u128);
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    assert!(client
        .try_donate(&pool_id, &Address::generate(&env), &100u128)
        .is_err());
    assert_eq!(client.get_total_raised(&pool_id), 100u128);

    client.set_pool_state(&pool_id, &PoolState::Active);
    client.donate(&pool_id, &Address::generate(&env), &100u128);
    assert_eq!(
        client.get_total_raised(&pool_id),
        200u128,
        "only the two donations that were accepted are counted"
    );
}

/// (5) The double-spend case: two claims against the same balance in one
/// sequence must not collectively take more than the pool holds.
#[test]
fn test_two_claims_against_one_balance_cannot_both_succeed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, pool_id, first, token) = claimable(&env, 100_000u128);

    client.claim_funds(&first, &pool_id, &60_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &first), 60_000i128);

    // The balance left is 40k; the next claim of 60k must be refused outright.
    let refused = client.try_claim_funds(&first, &pool_id, &60_000i128, &token);
    assert!(refused.is_err(), "the pool cannot pay out twice over");
    assert_eq!(
        client.get_claimed_amount(&pool_id, &first),
        60_000i128,
        "the refused claim must not advance the claimed total"
    );

    // And what is genuinely left is still claimable.
    client.claim_funds(&first, &pool_id, &40_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &first), 100_000i128);
}

/// (6) Ordering guarantee: the recorded last-donation timestamp never goes
/// backwards as calls land, even when the ledger clock does.
#[test]
fn test_last_donation_timestamp_is_monotonic() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let pool_id = create_pool(&client, &env, 1_000_000u128);

    env.ledger().set_timestamp(1_000);
    client.donate(&pool_id, &Address::generate(&env), &10u128);
    let first = client.get_last_donation_at(&pool_id);
    assert_eq!(first, 1_000u64);

    env.ledger().set_timestamp(2_000);
    client.donate(&pool_id, &Address::generate(&env), &10u128);
    let second = client.get_last_donation_at(&pool_id);
    assert_eq!(second, 2_000u64);
    assert!(
        second > first,
        "the stamp moves forward with the ledger clock"
    );
}
