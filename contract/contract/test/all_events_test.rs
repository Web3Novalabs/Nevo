#![cfg(test)]

use crate::{
    base::types::PoolConfig,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ---------------------------------------------------------------------------
// Shared setup
// ---------------------------------------------------------------------------

/// Register the contract, initialise it, and return (client, admin, token).
fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token, &0);
    (client, admin, token)
}

/// Build a minimal valid PoolConfig using the contract's registered token.
fn pool_config(env: &Env, token: &Address) -> PoolConfig {
    PoolConfig {
        name: String::from_str(env, "Test Pool"),
        description: String::from_str(env, "A test pool"),
        target_amount: 10_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
    }
}

// ---------------------------------------------------------------------------
// Initial-state tests
// ---------------------------------------------------------------------------

#[test]
fn test_initial_event_count_is_zero() {
    // Before any action the counter must not exist / return 0.
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // initialize itself emits no event, but even if it did the count starts at 0
    // before the call.  We verify the raw persistent storage via the query method.
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    // Count before any initialisation
    assert_eq!(client.get_all_events_count(), 0);
    assert_eq!(client.get_all_events().len(), 0);

    client.initialize(&admin, &token, &0);
    // initialize does not emit a tracked event, so count stays 0
    assert_eq!(client.get_all_events_count(), 0);
}

#[test]
fn test_initial_events_list_is_empty() {
    let env = Env::default();
    let (client, _, _) = setup(&env);
    // After setup (initialize only) the list must be empty.
    assert_eq!(client.get_all_events().len(), 0);
}

// ---------------------------------------------------------------------------
// Increment tests
// ---------------------------------------------------------------------------

#[test]
fn test_counter_increments_by_one_per_event() {
    let env = Env::default();
    let (client, creator, token) = setup(&env);

    let before = client.get_all_events_count();
    client.create_pool(&creator, &pool_config(&env, &token));
    let after = client.get_all_events_count();

    // create_pool emits pool_created + event_created → +2
    // We assert the delta is exactly 2 (both are tracked).
    assert_eq!(
        after - before,
        2,
        "create_pool must emit exactly 2 tracked events (pool_created + event_created)"
    );
}

#[test]
fn test_list_size_matches_counter() {
    let env = Env::default();
    let (client, creator, token) = setup(&env);

    client.create_pool(&creator, &pool_config(&env, &token));

    let count = client.get_all_events_count();
    let list_len = client.get_all_events().len() as u64;

    assert_eq!(count, list_len, "AllEventsCount must equal AllEvents.len()");
}

#[test]
fn test_single_event_increments_counter_by_one() {
    // pause emits exactly one tracked event (contract_paused).
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let before = client.get_all_events_count();
    client.pause();
    let after = client.get_all_events_count();

    assert_eq!(
        after - before,
        1,
        "pause must increment counter by exactly 1"
    );
}

#[test]
fn test_event_record_fields_are_correct() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let before_count = client.get_all_events_count();
    let ts = env.ledger().timestamp();

    client.pause();

    let records = client.get_all_events();
    let record = records.get(before_count as u32).unwrap();

    // index is 1-based and equals the new counter value
    assert_eq!(record.index, before_count + 1);
    assert_eq!(record.name, String::from_str(&env, "contract_paused"));
    assert_eq!(record.timestamp, ts);
}

// ---------------------------------------------------------------------------
// Multiple consecutive events
// ---------------------------------------------------------------------------

#[test]
fn test_counter_does_not_reset_across_multiple_events() {
    let env = Env::default();
    let (client, creator, token) = setup(&env);

    // Event 1: pause  (+1)
    client.pause();
    assert_eq!(client.get_all_events_count(), 1);

    // Event 2: unpause (+1)
    client.unpause();
    assert_eq!(client.get_all_events_count(), 2);

    // Events 3+4: create_pool emits pool_created + event_created (+2)
    client.create_pool(&creator, &pool_config(&env, &token));
    assert_eq!(client.get_all_events_count(), 4);

    // Events 5+6: second pool
    client.create_pool(&creator, &pool_config(&env, &token));
    assert_eq!(client.get_all_events_count(), 6);
}

#[test]
fn test_list_grows_monotonically() {
    let env = Env::default();
    let (client, creator, token) = setup(&env);

    let mut prev_len = client.get_all_events().len();

    for _ in 0..3 {
        client.create_pool(&creator, &pool_config(&env, &token));
        let new_len = client.get_all_events().len();
        assert!(
            new_len > prev_len,
            "AllEvents list must grow with each create_pool call"
        );
        prev_len = new_len;
    }
}

#[test]
fn test_event_indices_are_sequential() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    // Emit 3 single-event operations
    client.pause();
    client.unpause();
    client.pause();

    let records = client.get_all_events();
    for (i, record) in records.iter().enumerate() {
        assert_eq!(
            record.index,
            (i as u64) + 1,
            "event index must be sequential and 1-based"
        );
    }
}

#[test]
fn test_counter_and_list_stay_in_sync_after_many_events() {
    let env = Env::default();
    let (client, creator, token) = setup(&env);

    // Interleave different event types
    client.pause();
    client.unpause();
    client.create_pool(&creator, &pool_config(&env, &token));
    client.pause();
    client.unpause();
    client.create_pool(&creator, &pool_config(&env, &token));

    let count = client.get_all_events_count();
    let list_len = client.get_all_events().len() as u64;

    assert_eq!(
        count, list_len,
        "counter and list length must stay in sync after many events"
    );
    // 2×pause + 2×unpause + 2×create_pool(2 events each) = 8
    assert_eq!(count, 8);
}
