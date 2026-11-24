#![cfg(test)]
//! Issue #1333: `set_creation_fee` / `get_creation_fee` configuration.
//!
//! Scenarios:
//!   1. The admin can set a new fee value.
//!   2. A non-admin cannot set the fee (UnauthorizedAdmin).
//!   3. `get_creation_fee` reflects the latest set value.
//!   4. A zero fee is accepted and applied.
//!   5. A fee change does not retroactively affect already-created pools.

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// (1): the admin can set a fee and `get_creation_fee` reads it back.
#[test]
fn test_admin_can_set_new_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // No fee has been configured yet -> defaults to 0.
    assert_eq!(client.get_creation_fee(), 0);

    client.set_creation_fee(&admin, &500_000_000i128);
    assert_eq!(client.get_creation_fee(), 500_000_000);
}

/// (3): successive updates are reflected by `get_creation_fee`.
#[test]
fn test_get_creation_fee_reflects_latest_value() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    client.set_creation_fee(&admin, &100i128);
    assert_eq!(client.get_creation_fee(), 100);

    client.set_creation_fee(&admin, &250i128);
    assert_eq!(client.get_creation_fee(), 250);

    client.set_creation_fee(&admin, &7i128);
    assert_eq!(client.get_creation_fee(), 7);
}

/// (4): zero is a valid fee and is applied (overwriting a prior nonzero fee).
#[test]
fn test_zero_fee_is_accepted_and_applied() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    client.set_creation_fee(&admin, &1_000i128);
    assert_eq!(client.get_creation_fee(), 1_000);

    client.set_creation_fee(&admin, &0i128);
    assert_eq!(client.get_creation_fee(), 0);
}

/// (2): a non-admin caller cannot set the fee (UnauthorizedAdmin, #3).
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_non_admin_cannot_set_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let non_admin = Address::generate(&env);
    client.set_creation_fee(&non_admin, &100i128);
}

/// (5): changing the fee only affects pools created afterwards; a pool
/// created under the old fee is not re-charged when the fee changes.
#[test]
fn test_fee_change_does_not_retroactively_affect_existing_pools() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    let creator = Address::generate(&env);
    let initial_balance: i128 = 1_000_000_000;
    let fee_token = create_token(&env, initial_balance, &creator);
    let token_client = soroban_sdk::token::Client::new(&env, &fee_token);

    // Pool A is created while the fee is 100.
    let fee_a: i128 = 100;
    client.set_creation_fee(&admin, &fee_a);
    let pool_a = client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Pool A"),
        &String::from_str(&env, "Created under the original fee"),
        &5_000_000_000u128,
        &200_000u64,
        &fee_token,
    );
    assert_eq!(token_client.balance(&creator), initial_balance - fee_a);

    // Raise the fee to 250; only NEW pools pay this.
    let fee_b: i128 = 250;
    client.set_creation_fee(&admin, &fee_b);
    assert_eq!(client.get_creation_fee(), fee_b);

    let pool_b = client.create_pool_with_fee(
        &creator,
        &String::from_str(&env, "Pool B"),
        &String::from_str(&env, "Created under the updated fee"),
        &6_000_000_000u128,
        &210_000u64,
        &fee_token,
    );
    // Pool B paid the new fee; pool A was not re-charged.
    assert_eq!(
        token_client.balance(&creator),
        initial_balance - fee_a - fee_b
    );

    // Pool A's record is unchanged by the fee update.
    let pa = client.get_pool(&pool_a);
    assert_eq!(pa.0, pool_a);
    assert_eq!(pa.1, creator);
    assert_eq!(pa.2, 5_000_000_000i128);
    assert_eq!(pa.3, 0i128);

    // Pool B's record reflects its own creation parameters.
    let pb = client.get_pool(&pool_b);
    assert_eq!(pb.0, pool_b);
    assert_eq!(pb.1, creator);
    assert_eq!(pb.2, 6_000_000_000i128);
    assert_eq!(pb.3, 0i128);
}
