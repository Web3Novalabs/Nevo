#![cfg(test)]
//! Every pool-scoped getter fails with `PoolNotFound` for a missing pool, while
//! an existing pool with no data still returns its empty value — issue #1750.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

const MISSING: u32 = 999;

fn not_found() -> soroban_sdk::Error {
    ContractError::PoolNotFound.into()
}

macro_rules! assert_missing {
    ($($call:expr),+ $(,)?) => {
        $(assert_eq!($call.err(), Some(Ok(not_found())));)+
    };
}

#[test]
fn test_every_pool_getter_rejects_missing_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    assert_missing!(
        client.try_get_pool(&MISSING),
        client.try_get_pool_metadata(&MISSING),
        client.try_get_saved_pool_metadata(&MISSING),
        client.try_get_pool_signers(&MISSING),
        client.try_get_pool_token(&MISSING),
        client.try_get_pool_state(&MISSING),
        client.try_get_pool_school(&MISSING),
        client.try_get_pool_deadline(&MISSING),
        client.try_get_emergency_withdrawal(&MISSING),
        client.try_get_total_raised(&MISSING),
        client.try_get_campaign_balance(&MISSING),
        client.try_get_last_donation_at(&MISSING),
        client.try_get_donor_count(&MISSING),
        client.try_get_contribution(&MISSING, &user),
        client.try_get_application_count(&MISSING),
        client.try_get_application(&MISSING, &user),
        client.try_get_application_by_index(&MISSING, &1u32),
        client.try_get_application_status(&MISSING, &user),
        client.try_has_applied(&MISSING, &user),
        client.try_get_claimed_amount(&MISSING, &user),
        client.try_get_milestones(&MISSING, &user),
        client.try_get_released_total(&MISSING, &user),
        client.try_get_released_flags(&MISSING, &user),
    );
}

#[test]
fn test_existing_empty_pool_returns_empty_values() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let creator = Address::generate(&env);
    let user = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Empty Pool"),
        &String::from_str(&env, "No activity"),
        &1_000i128,
        &100_000u64,
    );

    let empty = String::from_str(&env, "");
    assert_eq!(
        client.get_saved_pool_metadata(&pool_id),
        (empty.clone(), empty.clone(), empty)
    );
    assert_eq!(client.get_application_count(&pool_id), 0);
    assert_eq!(client.get_donor_count(&pool_id), 0);
    assert_eq!(client.get_contribution(&pool_id, &user), 0);
    assert_eq!(client.get_application(&pool_id, &user), None);
    assert_eq!(client.get_application_by_index(&pool_id, &1u32), None);
    assert!(!client.has_applied(&pool_id, &user));
    assert_eq!(client.get_claimed_amount(&pool_id, &user), 0);
    assert_eq!(client.get_milestones(&pool_id, &user).len(), 0);
    assert_eq!(client.get_released_total(&pool_id, &user), 0);
    assert_eq!(client.get_released_flags(&pool_id, &user).len(), 0);
}
