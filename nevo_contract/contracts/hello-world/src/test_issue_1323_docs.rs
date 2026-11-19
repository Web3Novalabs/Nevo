#![cfg(test)]

// ============= ISSUE #1323: DOCUMENTATION ACCURACY =============
//
// Each test takes a specific written claim — a doc comment, the crate header,
// or the `# Panics` list — and holds the code to it. Where the two disagree the
// test says so rather than asserting the current behaviour as if it were the
// documented one; the disagreements found are listed in the PR description.

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, BytesN, Env, String, Vec,
};

fn make_client(env: &Env) -> ContractClient<'_> {
    let contract_id = env.register(Contract, ());
    ContractClient::new(env, &contract_id)
}

fn create_pool(client: &ContractClient, env: &Env, goal: u128) -> u32 {
    client.create_pool(
        &Address::generate(env),
        &String::from_str(env, "Documented Pool"),
        &String::from_str(env, "description"),
        &goal,
        &100_000u64,
    )
}

/// A pool linked to a registered school, plus a token the contract can pay from.
fn registered_school_pool(
    env: &Env,
    client: &ContractClient<'_>,
    goal: u128,
) -> (u32, Address, Address) {
    let admin = Address::generate(env);
    client.set_admin(&admin);

    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[9u8; 32]));

    let creator = Address::generate(env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(env, "School Pool"),
        &String::from_str(env, "description"),
        &goal,
        &school,
        &100_000u64,
    );

    let token = env.register_stellar_asset_contract_v2(Address::generate(env));
    StellarAssetClient::new(env, &token.address()).mint(&client.address, &(goal as i128));

    (pool_id, school, token.address())
}

/// `get_all_campaigns` is documented as "Return campaign ids in creation order."
#[test]
fn test_campaign_ids_come_back_in_creation_order_as_documented() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);

    let mut ids: Vec<u32> = Vec::new(&env);
    for _ in 0..3 {
        ids.push_back(create_pool(&client, &env, 1_000u128));
    }

    assert_eq!(ids, Vec::from_array(&env, [1u32, 2u32, 3u32]));
    assert_eq!(
        client.get_all_campaigns(),
        Vec::from_array(&env, [1u32, 2u32, 3u32]),
        "the documented creation order is the order the ids are stored and returned in"
    );
}

/// `get_creation_fee` is documented as returning `0` if no fee has been set.
#[test]
fn test_creation_fee_defaults_to_zero_as_documented() {
    let env = Env::default();
    let client = make_client(&env);

    assert_eq!(
        client.get_creation_fee(),
        0i128,
        "a fresh contract must report no fee rather than panicking"
    );
}

/// `donate` is documented as *not* requiring donor authentication and as not
/// performing a token transfer. No auths are mocked in this test on purpose: if
/// the documented promise were wrong, the call would trap here.
#[test]
fn test_donate_needs_no_donor_auth_as_documented() {
    let env = Env::default();
    let client = make_client(&env);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "No Auth Pool"),
        &String::from_str(&env, "description"),
        &1_000u128,
        &100_000u64,
    );

    let donor = Address::generate(&env);
    client.donate(&pool_id, &donor, &250u128);

    assert_eq!(client.get_total_raised(&pool_id), 250u128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 250u128);
}

/// The error enum documents that a pool "must be in Disbursed or Cancelled
/// state before it can be closed" — and that it cannot be closed twice.
#[test]
fn test_close_pool_obeys_the_documented_state_requirement() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);

    let pool_id = create_pool(&client, &env, 1_000u128);

    assert!(
        client.try_close_pool(&pool_id).is_err(),
        "an active pool is not in a closable state"
    );

    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);
    assert_eq!(client.get_pool(&pool_id).4, true, "the pool reports closed");

    assert!(
        client.try_close_pool(&pool_id).is_err(),
        "closing an already closed pool is refused, as documented"
    );
}

/// The `# Panics` list gives an exact message for each of four conditions. One
/// test per condition: `should_panic(expected = …)` is the only way to see the
/// message, because the generated `try_` variant hands back a bare status with
/// no text in it.
#[test]
#[should_panic(expected = "Claim amount must be positive")]
fn test_claim_zero_amount_panics_with_the_documented_message() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let (pool_id, _school, token) = registered_school_pool(&env, &client, 100_000u128);

    let student = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &0i128, &token);
}

#[test]
#[should_panic(expected = "Application status not found")]
fn test_claim_without_an_application_panics_with_the_documented_message() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let (pool_id, _school, token) = registered_school_pool(&env, &client, 100_000u128);

    // This student never applied to the pool.
    let student = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &1i128, &token);
}

#[test]
#[should_panic(expected = "Application is not approved")]
fn test_claim_not_approved_panics_with_the_documented_message() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let (pool_id, _school, token) = registered_school_pool(&env, &client, 100_000u128);

    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));
    client.claim_funds(&student, &pool_id, &1i128, &token);
}

#[test]
#[should_panic(expected = "Overdraw attempt")]
fn test_overdraw_panics_with_the_documented_message() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let (pool_id, school, token) = registered_school_pool(&env, &client, 100_000u128);
    client.donate(&pool_id, &Address::generate(&env), &100_000u128);

    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));
    client.approve_application(&pool_id, &school, &student, &true);

    client.claim_funds(&student, &pool_id, &100_001i128, &token);
}

/// Positive control for the four tests above: the very same setup succeeds for a
/// claim inside the limit, so they cannot be passing because the fixture traps
/// on its own before reaching the documented condition.
#[test]
fn test_an_approved_claim_inside_the_limit_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let (pool_id, school, token) = registered_school_pool(&env, &client, 100_000u128);
    client.donate(&pool_id, &Address::generate(&env), &100_000u128);

    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));
    client.approve_application(&pool_id, &school, &student, &true);

    client.claim_funds(&student, &pool_id, &100_000i128, &token);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 100_000i128);
}

/// The crate header states that milestones "must sum to the pool goal", and
/// `setup_application_milestones` repeats the rule on itself. Both under- and
/// over-shooting must be refused, and an empty list too.
#[test]
fn test_milestone_sum_must_equal_the_pool_goal_as_documented() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let (pool_id, _school, _token) = registered_school_pool(&env, &client, 100_000u128);

    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));

    let empty: Vec<Milestone> = Vec::new(&env);
    assert!(
        client
            .try_setup_application_milestones(&pool_id, &student, &empty)
            .is_err(),
        "an empty milestone list cannot sum to the goal"
    );

    let under = Vec::from_array(&env, [Milestone { amount: 99_999u128 }]);
    assert!(
        client
            .try_setup_application_milestones(&pool_id, &student, &under)
            .is_err(),
        "a sum below the goal is refused"
    );

    let over = Vec::from_array(
        &env,
        [Milestone {
            amount: 100_001u128,
        }],
    );
    assert!(
        client
            .try_setup_application_milestones(&pool_id, &student, &over)
            .is_err(),
        "a sum above the goal is refused"
    );

    // Two milestones that do add up are accepted and read back unchanged.
    let exact = Vec::from_array(
        &env,
        [
            Milestone { amount: 40_000u128 },
            Milestone { amount: 60_000u128 },
        ],
    );
    client.setup_application_milestones(&pool_id, &student, &exact);
    let stored = client.get_milestones(&pool_id, &student);
    assert_eq!(stored.len(), 2u32);
    assert_eq!(stored.get(0).unwrap().amount, 40_000u128);
    assert_eq!(stored.get(1).unwrap().amount, 60_000u128);
}

/// The `# Panics` list names four conditions and stops there, but `claim_funds`
/// also refuses a pool that has been cancelled — a condition that *is*
/// reachable, because a student can be approved and the pool cancelled
/// afterwards. That condition is missing from the documentation.
#[test]
fn test_claim_from_a_cancelled_pool_is_refused_though_the_docs_omit_it() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);
    let (pool_id, school, token) = registered_school_pool(&env, &client, 100_000u128);
    client.donate(&pool_id, &Address::generate(&env), &100_000u128);

    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));
    client.approve_application(&pool_id, &school, &student, &true);

    // Reached the way a real claim would be: approved first, cancelled after.
    client.set_pool_state(&pool_id, &PoolState::Cancelled);
    let refused = client.try_claim_funds(&student, &pool_id, &10_000i128, &token);

    assert!(refused.is_err(), "a cancelled pool must not pay out");
    assert_eq!(
        client.get_claimed_amount(&pool_id, &student),
        0i128,
        "a refused claim must not be recorded as taken"
    );
}
