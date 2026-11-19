#![cfg(test)]
//! Regression test for the unset-crowdfunding-token panic — issue #1483.
//!
//! `get_crowdfunding_token` panics with a plain `.expect()` string, while
//! `set_crowdfunding_token` resolves its admin through the typed
//! `ContractError::AdminNotSet`. This test pins the exact current panic text so
//! that any later change to a typed error is a deliberate, reviewed one.
//!
//! Follow-up (not implemented here): replace the `.expect()` with a typed
//! `ContractError` variant, consistent with `ContractError::AdminNotSet` and
//! `ContractError::SchoolNotRegistered`, and document the new code in
//! `nevo_contract/ERRORS.md`.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Calling the getter before `set_crowdfunding_token` has ever succeeded must
/// panic with the exact string currently produced by the `.expect()` call.
#[test]
#[should_panic(expected = "Crowdfunding token not set")]
fn test_get_crowdfunding_token_before_configuration_panics_with_current_message() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let _ = client.get_crowdfunding_token();
}

/// The same condition reached through `try_get_crowdfunding_token`, which
/// confirms the failure is raised by the getter itself and is not an artifact
/// of an unrelated panic in the test setup.
#[test]
fn test_try_get_crowdfunding_token_before_configuration_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert!(
        client.try_get_crowdfunding_token().is_err(),
        "get_crowdfunding_token must fail while no token has been configured"
    );
}

/// Setting an admin alone does not configure a token: the getter must still
/// fail with the same panic until `set_crowdfunding_token` succeeds.
#[test]
#[should_panic(expected = "Crowdfunding token not set")]
fn test_get_crowdfunding_token_panics_until_token_is_configured() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.set_admin(&Address::generate(&env));

    let _ = client.get_crowdfunding_token();
}

/// After a successful `set_crowdfunding_token` the getter returns the token,
/// so the panic tests above cannot pass because the getter is simply broken.
#[test]
fn test_get_crowdfunding_token_succeeds_once_configured() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);

    client.set_admin(&admin);
    client.set_crowdfunding_token(&admin, &token);

    assert_eq!(client.get_crowdfunding_token(), token);
}
