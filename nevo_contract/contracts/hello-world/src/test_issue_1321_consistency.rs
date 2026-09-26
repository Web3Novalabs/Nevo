#![cfg(test)]

// ============= ISSUE #1321: BUSINESS-LOGIC CONSISTENCY =============
//
// The five areas the issue lists, one test each:
//   1. campaign rules applied consistently (creation validation)
//   2. pool rules enforced uniformly (same rule on every path)
//   3. fee calculations accurate (typed errors, round-trip)
//   4. time-based rules correct (deadline + grace-period boundaries)
//   5. state logic consistent across every read path
//
// Two divergences this module deliberately does NOT paper over are reported in
// the PR description instead of asserted here: `create_pool` raises bare strings
// where the error enum defines `InvalidPoolName`/`InvalidPoolTarget`, and
// `MAX_TITLE_LENGTH` is declared but never used, so a title has no length cap
// while the description does. The two pre-existing failures in
// `test_pool_creation` are exactly those two gaps.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn make_client(env: &Env) -> ContractClient<'_> {
    let contract_id = env.register(Contract, ());
    ContractClient::new(env, &contract_id)
}

// A description of exactly the limit and one character over it. Written as
// literals rather than built in a loop: soroban_sdk::String has no concat, and
// the boundary itself is what is under test.
const DESC_AT_LIMIT: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const DESC_OVER_LIMIT: &str = "ddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

/// (1) Every invalid creation input is refused — the same rule is not enforced
/// on one field and skipped on the next.
#[test]
fn test_campaign_creation_rules_are_enforced_consistently() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Pool");
    let desc = String::from_str(&env, "description");

    // Empty title, empty description, over-long description, zero deadline:
    // all four must be rejected.
    assert!(client
        .try_create_pool(
            &creator,
            &String::from_str(&env, ""),
            &desc,
            &1_000u128,
            &100_000u64
        )
        .is_err());
    assert!(client
        .try_create_pool(
            &creator,
            &title,
            &String::from_str(&env, ""),
            &1_000u128,
            &100_000u64
        )
        .is_err());
    assert!(client
        .try_create_pool(
            &creator,
            &title,
            &String::from_str(&env, DESC_OVER_LIMIT),
            &1_000u128,
            &100_000u64
        )
        .is_err());
    assert!(client
        .try_create_pool(&creator, &title, &desc, &1_000u128, &0u64)
        .is_err());

    // ...and a well-formed pool is accepted, so the checks are not refusing
    // everything indiscriminately.
    let pool_id = client.create_pool(&creator, &title, &desc, &1_000u128, &100_000u64);
    assert_eq!(pool_id, 1u32, "the first accepted pool takes id 1");
}

/// (1b) The description limit is exact at its own boundary, not approximate.
#[test]
fn test_creation_rule_boundary_is_exact() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Boundary Pool");

    let ok = client.try_create_pool(
        &creator,
        &title,
        &String::from_str(&env, DESC_AT_LIMIT),
        &1_000u128,
        &100_000u64,
    );
    assert!(
        ok.is_ok(),
        "a description of exactly the limit must be accepted"
    );

    let too_long = client.try_create_pool(
        &creator,
        &title,
        &String::from_str(&env, DESC_OVER_LIMIT),
        &1_000u128,
        &100_000u64,
    );
    assert!(
        too_long.is_err(),
        "one character over the limit must be refused"
    );
}

/// (4) Time-based rules: the deadline helpers agree with each other, and the
/// grace period includes its last second and excludes the next one.
#[test]
fn test_time_based_rules_work_at_their_boundaries() {
    let now = current_timestamp();

    // A deadline that has not arrived is not in the grace period.
    let future = now + 1;
    assert!(!is_within_grace_period(future, GRACE_PERIOD_SECS));
    assert!(validate_deadline(future).is_ok());

    // "Now" itself is the first second of the grace period, not a failure.
    assert!(is_within_grace_period(now, GRACE_PERIOD_SECS));
    assert!(
        validate_deadline(now).is_err(),
        "a deadline of 'now' is not in the future"
    );

    // The last second inside the window is in, the next one is out.
    let last_inside = now - GRACE_PERIOD_SECS;
    let first_outside = now - GRACE_PERIOD_SECS - 1;
    assert!(is_within_grace_period(last_inside, GRACE_PERIOD_SECS));
    assert!(!is_within_grace_period(first_outside, GRACE_PERIOD_SECS));

    // Zero grace means only the deadline instant itself counts.
    assert!(is_within_grace_period(now, 0));
    assert!(!is_within_grace_period(now - 1, 0));
}

/// (3) Fee rules: a negative fee is refused with the typed error, and the
/// stored value round-trips unchanged.
#[test]
fn test_fee_rules_are_accurate_and_typed() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    client.set_creation_fee(&admin, &500i128);
    assert_eq!(client.get_creation_fee(), 500i128);

    // Setting zero is legal and means "no fee".
    client.set_creation_fee(&admin, &0i128);
    assert_eq!(client.get_creation_fee(), 0i128);

    // A negative fee is rejected.
    assert!(client.try_set_creation_fee(&admin, &-1i128).is_err());
    assert_eq!(
        client.get_creation_fee(),
        0i128,
        "a refused fee must not overwrite the stored one"
    );
}

/// (3b) Claiming when nothing has accumulated is refused rather than moving
/// zero tokens.
#[test]
fn test_claiming_fees_with_an_empty_accumulator_is_refused() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let token = Address::generate(&env);
    assert!(
        client.try_claim_protocol_fees(&admin, &token).is_err(),
        "there is nothing to claim on a fresh contract"
    );
}

/// (2 + 5) The contribution ledger must agree with itself: the running total is
/// exactly the sum of the per-donor contributions, and that identity survives
/// the pool being closed — the same fact read back through every path.
#[test]
fn test_pool_rules_and_state_are_consistent_across_read_paths() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Consistency Pool"),
        &String::from_str(&env, "description"),
        &1_000_000u128,
        &100_000u64,
    );

    let donors = [
        (Address::generate(&env), 10_000u128),
        (Address::generate(&env), 20_000u128),
        (Address::generate(&env), 30_000u128),
    ];
    let mut expected = 0u128;
    for (donor, amount) in donors.iter() {
        client.donate(&pool_id, donor, amount);
        expected += amount;
    }

    let sum_of_parts: u128 = donors
        .iter()
        .map(|(donor, _)| client.get_contribution(&pool_id, donor))
        .sum();
    assert_eq!(client.get_total_raised(&pool_id), expected);
    assert_eq!(sum_of_parts, expected, "the parts must sum to the whole");
    assert_eq!(client.get_donor_count(&pool_id), 3u32);

    // Closing the pool must not change any of those numbers.
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);

    let (_id, _sponsor, _goal, collected, is_closed, _last) = client.get_pool(&pool_id);
    assert!(is_closed);
    assert_eq!(
        collected, expected,
        "closing must not alter what was collected"
    );
    assert_eq!(client.get_total_raised(&pool_id), expected);
    assert_eq!(
        donors
            .iter()
            .map(|(donor, _)| client.get_contribution(&pool_id, donor))
            .sum::<u128>(),
        expected
    );
    assert!(
        client.try_donate(&pool_id, &donors[0].0, &1u128).is_err(),
        "a closed pool stops accepting donations"
    );
}

/// (5b) Arithmetic stays exact at magnitudes where a lossy implementation would
/// already have started rounding.
#[test]
fn test_arithmetic_stays_exact_at_large_values() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Large Value Pool"),
        &String::from_str(&env, "description"),
        &u128::MAX,
        &100_000u64,
    );

    let big = 1u128 << 100;
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    client.donate(&pool_id, &alice, &big);
    client.donate(&pool_id, &bob, &(big + 1));

    assert_eq!(client.get_total_raised(&pool_id), (1u128 << 101) + 1);
    assert_eq!(client.get_contribution(&pool_id, &alice), big);
    assert_eq!(client.get_contribution(&pool_id, &bob), big + 1);
    assert_eq!(client.get_donor_count(&pool_id), 2u32);
}
