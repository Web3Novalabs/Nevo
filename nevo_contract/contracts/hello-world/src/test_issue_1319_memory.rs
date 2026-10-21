#![cfg(test)]

// ============= ISSUE #1319: MEMORY / STORAGE FOOTPRINT =============
//
// On-chain "memory" is persistent storage, and it is measurable: the test
// environment exposes the contract's whole persistent map, so these tests count
// entries before and after an operation instead of describing them.
//
// The five properties the issue names, mapped onto what can actually be
// observed here:
//   1. large data structures handled      -> big milestone lists and long strings
//                                             round-trip intact;
//   2. memory cleanup after operations    -> refused operations add nothing;
//   3. storage vs memory usage balanced   -> growth follows entities, not calls;
//   4. no leaks in loops                  -> repeating an operation stops
//                                             consuming storage after its first run;
//   5. efficient data structures used     -> per-donor records stay individual and
//                                             exact at scale.

use super::*;
use soroban_sdk::{
    testutils::{storage::Persistent as _, Address as _},
    token::StellarAssetClient,
    Address, BytesN, Env, String, Vec,
};

fn setup(env: &Env) -> (ContractClient<'_>, Address) {
    let contract_id = env.register(Contract, ());
    (ContractClient::new(env, &contract_id), contract_id)
}

/// Number of persistent entries the contract currently holds. Storage is read
/// through `as_contract`, because outside a contract invocation the host has no
/// "current contract" to look at.
fn entries(env: &Env, contract_id: &Address) -> u32 {
    env.as_contract(contract_id, || env.storage().persistent().all().len())
}

fn create_pool(client: &ContractClient, env: &Env, goal: u128) -> u32 {
    client.create_pool(
        &Address::generate(env),
        &String::from_str(env, "Footprint Pool"),
        &String::from_str(env, "description"),
        &goal,
        &100_000u64,
    )
}

/// (4) Repeated identical work must stop consuming storage after the first run.
/// A loop that leaked one entry per iteration would show up here immediately.
#[test]
fn test_repeating_an_operation_stops_growing_storage() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id) = setup(&env);
    let pool_id = create_pool(&client, &env, 1_000_000u128);

    let alice = Address::generate(&env);
    client.donate(&pool_id, &alice, &1u128);
    let after_first = entries(&env, &contract_id);

    for _ in 0..50 {
        client.donate(&pool_id, &alice, &1u128);
    }
    let after_fifty = entries(&env, &contract_id);

    assert_eq!(
        after_fifty, after_first,
        "fifty further donations from the same donor must not add a single entry"
    );
    assert_eq!(client.get_contribution(&pool_id, &alice), 51u128);
}

/// (2) A refused operation must leave no orphan record behind — the classic way
/// a contract leaks storage without anyone noticing.
#[test]
fn test_refused_operations_leave_no_storage_behind() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id) = setup(&env);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    let school = Address::generate(&env);
    client.register_school(&school, &BytesN::from_array(&env, &[3u8; 32]));
    let pool_id = client.create_pool_for_school(
        &Address::generate(&env),
        &String::from_str(&env, "Refusal Pool"),
        &String::from_str(&env, "description"),
        &100_000u128,
        &school,
        &100_000u64,
    );
    client.donate(&pool_id, &Address::generate(&env), &100_000u128);

    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));
    client.approve_application(&pool_id, &school, &student, &true);

    let token = env.register_stellar_asset_contract_v2(Address::generate(&env));
    StellarAssetClient::new(&env, &token.address()).mint(&client.address, &100_000i128);

    let before = entries(&env, &contract_id);

    // Four refusals in a row: closing an active pool, closing it twice over once
    // it is closed, donating into a cancelled pool, and overdrawing a claim.
    assert!(client.try_close_pool(&pool_id).is_err());
    client.set_pool_state(&pool_id, &PoolState::Disbursed);
    client.close_pool(&pool_id);
    let after_close = entries(&env, &contract_id);
    assert!(client.try_close_pool(&pool_id).is_err());
    assert!(client
        .try_donate(&pool_id, &Address::generate(&env), &1u128)
        .is_err());
    assert!(client
        .try_claim_funds(&student, &pool_id, &100_001i128, &token.address())
        .is_err());

    assert_eq!(
        entries(&env, &contract_id),
        after_close,
        "refusals after the close must not have written anything"
    );
    assert!(after_close >= before);
}

/// (3) Storage should track entities, not traffic: twenty donations from five
/// people must cost about a fifth of twenty donations from twenty people.
#[test]
fn test_storage_grows_with_entities_not_with_call_volume() {
    let env = Env::default();
    env.mock_all_auths();

    // Twenty donations from one donor.
    let (client, contract_id) = setup(&env);
    let pool_id = create_pool(&client, &env, 1_000_000u128);
    let start = entries(&env, &contract_id);
    let alice = Address::generate(&env);
    for _ in 0..20 {
        client.donate(&pool_id, &alice, &1u128);
    }
    let one_donor_cost = entries(&env, &contract_id) - start;

    // Twenty donations from twenty donors, same total volume.
    let env2 = Env::default();
    env2.mock_all_auths();
    let (client2, contract_id2) = setup(&env2);
    let pool_id2 = create_pool(&client2, &env2, 1_000_000u128);
    let start2 = entries(&env2, &contract_id2);
    for _ in 0..20 {
        client2.donate(&pool_id2, &Address::generate(&env2), &1u128);
    }
    let many_donor_cost = entries(&env2, &contract_id2) - start2;

    assert_eq!(
        one_donor_cost, 3u32,
        "one donor costs a donor marker, a contribution and a count update"
    );
    assert!(
        many_donor_cost >= 20u32,
        "twenty distinct donors must each own their record, cost was {many_donor_cost}"
    );
    assert_eq!(client.get_total_raised(&pool_id), 20u128);
    assert_eq!(client2.get_total_raised(&pool_id2), 20u128);
}

/// (1) A structure at the size limit of the flow is accepted and read back
/// whole — nothing truncated, nothing dropped.
#[test]
fn test_a_large_milestone_list_round_trips_whole() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id) = setup(&env);
    let admin = Address::generate(&env);
    client.set_admin(&admin);
    let school = Address::generate(&env);
    client.register_school(&school, &BytesN::from_array(&env, &[6u8; 32]));

    // 200 milestones of 500 each add up to a 100_000 goal exactly.
    let goal = 100_000u128;
    let pool_id = client.create_pool_for_school(
        &Address::generate(&env),
        &String::from_str(&env, "Large Structure Pool"),
        &String::from_str(&env, "description"),
        &goal,
        &school,
        &100_000u64,
    );
    let student = Address::generate(&env);
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));

    let mut milestones = Vec::new(&env);
    for _ in 0..200 {
        milestones.push_back(Milestone { amount: 500u128 });
    }
    let before = entries(&env, &contract_id);
    client.setup_application_milestones(&pool_id, &student, &milestones);

    let stored = client.get_milestones(&pool_id, &student);
    assert_eq!(
        stored.len(),
        200u32,
        "every milestone must survive the round trip"
    );
    let mut sum = 0u128;
    for m in stored.iter() {
        assert_eq!(m.amount, 500u128);
        sum += m.amount;
    }
    assert_eq!(sum, goal);
    assert_eq!(
        entries(&env, &contract_id) - before,
        1u32,
        "the whole list is one entry, not one entry per milestone"
    );
}

/// (5) Keyed lookups stay exact at scale: fifty donors with fifty different
/// amounts, each read back individually and correctly.
#[test]
fn test_per_donor_records_stay_exact_at_scale() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id) = setup(&env);
    let pool_id = create_pool(&client, &env, 1_000_000u128);

    let mut donors = Vec::new(&env);
    let mut amounts = Vec::new(&env);
    for i in 1..=50u128 {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &(i * 7u128));
        donors.push_back(donor);
        amounts.push_back(i * 7u128);
    }

    let mut expected = 0u128;
    for i in 0..donors.len() {
        let donor = donors.get(i).unwrap();
        let amount = amounts.get(i).unwrap();
        assert_eq!(
            client.get_contribution(&pool_id, &donor),
            amount,
            "each donor's own record must come back, not a neighbour's"
        );
        expected += amount;
    }
    assert_eq!(client.get_total_raised(&pool_id), expected);
    assert_eq!(client.get_donor_count(&pool_id), 50u32);
}

/// (3b) Creating entities costs storage in proportion to the entities — and the
/// campaign list stays exactly as long as the number of pools created.
#[test]
fn test_pool_storage_is_proportional_to_pools_created() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id) = setup(&env);

    let start = entries(&env, &contract_id);
    let mut ids = Vec::new(&env);
    for _ in 0..30 {
        ids.push_back(create_pool(&client, &env, 1_000u128));
    }
    let cost = entries(&env, &contract_id) - start;

    assert_eq!(client.get_all_campaigns().len(), 30u32);
    assert_eq!(ids.len(), 30u32);
    assert!(
        cost <= 90u32,
        "thirty pools must not cost more than a few entries each, cost was {cost}"
    );
    assert!(cost >= 30u32, "each pool needs at least its own record");
}
