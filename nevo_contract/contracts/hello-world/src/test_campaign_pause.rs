#![cfg(test)]

use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, String};

// ============= GET_TOTAL_RAISED TESTS =============

#[test]
fn test_get_total_raised_new_campaign_returns_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "New Campaign"),
        &String::from_str(&env, "Just created"),
        &1_000_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_total_raised(&pool_id), 0u128);
}

#[test]
fn test_get_total_raised_single_donation_updates_total() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Single Donation Campaign"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor, &250_000_000u128);
    assert_eq!(client.get_total_raised(&pool_id), 250_000_000u128);
}

#[test]
fn test_get_total_raised_multiple_donations_sum_correctly() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Multi Donation Campaign"),
        &String::from_str(&env, "Test"),
        &5_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
    client.donate(&pool_id, &Address::generate(&env), &200_000_000u128);
    client.donate(&pool_id, &Address::generate(&env), &50_000_000u128);

    assert_eq!(client.get_total_raised(&pool_id), 350_000_000u128);
}

#[test]
fn test_get_total_raised_matches_campaign_balance() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Balance Match Campaign"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &Address::generate(&env), &123_456_789u128);

    let pool = client.get_pool(&pool_id);
    assert_eq!(client.get_total_raised(&pool_id), pool.3);
}

#[test]
#[should_panic(expected = "Pool not found")]
fn test_get_total_raised_nonexistent_campaign_returns_error() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let _ = client.get_total_raised(&999u32);
}

// ============= PAUSED CONTRACT RESTRICTIONS TESTS =============

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_create_campaign_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.pause(&admin);

    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Blocked Campaign"),
        &String::from_str(&env, "Should not be created"),
        &1_000_000_000u128,
        &100_000u64,
    );
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_save_pool_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    client.set_admin(&admin);
    client.register_school(&admin, &school);

    client.pause(&admin);

    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Blocked School Campaign"),
        &String::from_str(&env, "Should not be saved"),
        &1_000_000_000u128,
        &school,
        &100_000u64,
    );
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_contribute_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Campaign"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.pause(&admin);

    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
}

#[test]
#[should_panic(expected = "ContractPaused")]
fn test_update_pool_state_fails_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Campaign"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client.pause(&admin);

    client.set_pool_state(&pool_id, &PoolState::Paused);
}

#[test]
fn test_getters_work_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Readable Campaign"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );
    client.donate(&pool_id, &Address::generate(&env), &50_000_000u128);

    client.pause(&admin);

    assert!(client.is_paused());
    assert_eq!(client.get_total_raised(&pool_id), 50_000_000u128);
    assert_eq!(client.get_campaign_goal(&pool_id), 1_000_000_000u128);
    assert_eq!(client.get_pool_count(), 1);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, pool_id);
}

// ============= CONTRACT PAUSE FUNCTIONALITY TESTS =============

#[test]
fn test_admin_can_pause_unpaused_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    assert!(!client.is_paused());
    client.pause(&admin);
    assert!(client.is_paused());
}

#[test]
fn test_admin_can_unpause_paused_contract() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.pause(&admin);
    assert!(client.is_paused());

    client.unpause(&admin);
    assert!(!client.is_paused());
}

#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_non_admin_cannot_pause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.set_admin(&admin);

    client.pause(&non_admin);
}

#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_non_admin_cannot_unpause() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.set_admin(&admin);
    client.pause(&admin);

    client.unpause(&non_admin);
}

#[test]
#[should_panic(expected = "AlreadyPaused")]
fn test_double_pause_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.pause(&admin);
    client.pause(&admin);
}

#[test]
#[should_panic(expected = "AlreadyUnpaused")]
fn test_double_unpause_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.pause(&admin);
    client.unpause(&admin);
    client.unpause(&admin);
}

#[test]
fn test_is_paused_returns_correct_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    assert_eq!(client.is_paused(), false);
    client.pause(&admin);
    assert_eq!(client.is_paused(), true);
    client.unpause(&admin);
    assert_eq!(client.is_paused(), false);
}

// ============= GET_CAMPAIGN_GOAL TESTS =============

#[test]
fn test_get_campaign_goal_returns_correct_goal() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Goal Campaign"),
        &String::from_str(&env, "Test"),
        &7_500_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_campaign_goal(&pool_id), 7_500_000_000u128);
}

#[test]
#[should_panic(expected = "CampaignNotFound")]
fn test_get_campaign_goal_nonexistent_campaign_returns_error() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let _ = client.get_campaign_goal(&999u32);
}

#[test]
fn test_get_campaign_goal_matches_creation_value() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal = 42_000_000u128;
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Precise Goal Campaign"),
        &String::from_str(&env, "Test"),
        &goal,
        &100_000u64,
    );

    assert_eq!(client.get_campaign_goal(&pool_id), goal);
}

#[test]
fn test_get_campaign_goal_multiple_campaigns_independent_goals() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id_1 = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Campaign A"),
        &String::from_str(&env, "First"),
        &1_000_000_000u128,
        &100_000u64,
    );
    let pool_id_2 = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Campaign B"),
        &String::from_str(&env, "Second"),
        &2_500_000_000u128,
        &100_000u64,
    );

    assert_eq!(client.get_campaign_goal(&pool_id_1), 1_000_000_000u128);
    assert_eq!(client.get_campaign_goal(&pool_id_2), 2_500_000_000u128);
}
