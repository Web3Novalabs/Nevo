#![cfg(test)]

// ============= ISSUE #1370: FEE CALCULATION ACROSS MID-FLOW CHANGES =============
//
// `create_pool_with_fee` has no per-pool fee record — it reads
// `get_creation_fee()` at call time, transfers that amount from the
// creator, and accumulates it into the global unclaimed-fees total. These
// tests verify that each pool creation is charged exactly the fee that was
// active at the moment it was created, with no cross-contamination when
// `set_creation_fee` is called between creations.

use super::*;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String, Symbol};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn token_balance(env: &Env, token: &Address, account: &Address) -> i128 {
    token::Client::new(env, token).balance(account)
}

/// A pool created before a fee change was charged the fee that was active
/// at its own creation time; the later change does not retroactively alter
/// what was already charged (there is no per-pool fee record to update).
#[test]
fn test_pool_created_before_fee_change_charged_original_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let original_fee: i128 = 100_000;
    client.set_creation_fee(&admin, &original_fee);

    let creator_a = Address::generate(&env);
    let fee_token = create_token(&env, original_fee, &creator_a);
    client.create_pool_with_fee(
        &creator_a,
        &String::from_str(&env, "Pool A"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &200_000u64,
        &fee_token,
    );
    assert_eq!(token_balance(&env, &fee_token, &creator_a), 0i128);
    assert_eq!(
        token_balance(&env, &fee_token, &contract_id),
        original_fee,
        "Pool A must have been charged the original fee"
    );

    // Changing the fee afterwards does not touch creator_a's already-paid balance.
    client.set_creation_fee(&admin, &500_000i128);
    assert_eq!(token_balance(&env, &fee_token, &creator_a), 0i128);
    assert_eq!(token_balance(&env, &fee_token, &contract_id), original_fee);
}

/// A pool created after a fee change is charged the new fee, not the fee
/// that was active when an earlier pool was created.
#[test]
fn test_pool_created_after_fee_change_uses_new_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &100_000i128);

    let new_fee: i128 = 500_000;
    client.set_creation_fee(&admin, &new_fee);

    let creator_b = Address::generate(&env);
    let fee_token = create_token(&env, new_fee, &creator_b);
    client.create_pool_with_fee(
        &creator_b,
        &String::from_str(&env, "Pool B"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &200_000u64,
        &fee_token,
    );

    assert_eq!(token_balance(&env, &fee_token, &creator_b), 0i128);
    assert_eq!(token_balance(&env, &fee_token, &contract_id), new_fee);
}

/// Sequential creations around a fee change each pay exactly the fee that
/// was active for their own call — the two charges never mix into a
/// combined or averaged value.
#[test]
fn test_sequential_pool_creations_around_fee_change_use_consistent_values() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let fee_x: i128 = 200_000;
    client.set_creation_fee(&admin, &fee_x);

    let creator_a = Address::generate(&env);
    let fee_token = create_token(&env, fee_x, &creator_a);
    client.create_pool_with_fee(
        &creator_a,
        &String::from_str(&env, "Pool A"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &200_000u64,
        &fee_token,
    );

    let fee_y: i128 = 700_000;
    client.set_creation_fee(&admin, &fee_y);

    let creator_b = Address::generate(&env);
    // Fund creator_b through the same token so contract balance is directly comparable.
    let sac = StellarAssetClient::new(&env, &fee_token);
    sac.mint(&creator_b, &fee_y);
    client.create_pool_with_fee(
        &creator_b,
        &String::from_str(&env, "Pool B"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &200_000u64,
        &fee_token,
    );

    // The contract must hold exactly fee_x + fee_y — never a mixed/partial value.
    assert_eq!(
        token_balance(&env, &fee_token, &contract_id),
        fee_x + fee_y
    );
    assert_eq!(token_balance(&env, &fee_token, &creator_a), 0i128);
    assert_eq!(token_balance(&env, &fee_token, &creator_b), 0i128);
}

/// `get_creation_fee` reflects the latest committed value throughout a
/// sequence of pool creations interleaved with fee changes.
#[test]
fn test_get_creation_fee_reflects_latest_value_mid_flow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    client.set_creation_fee(&admin, &100_000i128);
    assert_eq!(client.get_creation_fee(), 100_000i128);

    let creator = Address::generate(&env);
    let fee_token = create_token(&env, 100_000i128, &creator);
    client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &200_000u64,
        &fee_token,
    );
    // Creating a pool does not itself change the configured fee.
    assert_eq!(client.get_creation_fee(), 100_000i128);

    client.set_creation_fee(&admin, &900_000i128);
    assert_eq!(client.get_creation_fee(), 900_000i128);
}

/// `set_creation_fee` emits the `creation_fee_updated` event carrying the
/// exact new fee value.
#[test]
fn test_fee_change_event_carries_new_fee_value() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let new_fee: i128 = 350_000;
    client.set_creation_fee(&admin, &new_fee);

    let events = env.events().all();
    let event = events.last().unwrap();
    let topic_sym: Symbol = event.1.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(topic_sym, Symbol::new(&env, "creation_fee_updated"));
    let emitted_fee: i128 = event.2.try_into_val(&env).unwrap();
    assert_eq!(emitted_fee, new_fee);
}
