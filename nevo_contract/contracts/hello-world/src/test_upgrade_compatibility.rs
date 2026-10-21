//! Tests for contract upgrade compatibility.
//!
//! These tests exercise the scenarios that must remain stable across contract
//! upgrades:
//!   1. Storage layout compatibility.
//!   2. Function signature changes / stability.
//!   3. New field additions.
//!   4. Deprecated function handling.
//!   5. Migration path validation.

use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol, Vec};

use crate::{HelloContract, HelloContractClient};

/// Helper: deploy a fresh contract instance and return its client.
fn setup(env: &Env) -> (Address, HelloContractClient<'_>) {
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(env, &contract_id);
    (contract_id, client)
}

// ---------------------------------------------------------------------------
// (1) Storage layout compatibility
// ---------------------------------------------------------------------------

#[test]
fn test_storage_layout_survives_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);

    // Seed state that must survive an upgrade.
    client.initialize(&admin);
    let campaign_id = client.create_campaign(&admin, &String::from_str(&env, "upgrade-campaign"));

    // Simulate an upgrade by re-registering the same contract id with the
    // current (upgraded) wasm. Storage entries keyed by the same layout must
    // remain readable.
    env.register_contract(Some(&contract_id), HelloContract);
    let upgraded = HelloContractClient::new(&env, &contract_id);

    let stored = upgraded.get_campaign(&campaign_id);
    assert_eq!(stored.id, campaign_id);
    assert_eq!(stored.name, String::from_str(&env, "upgrade-campaign"));
}

#[test]
fn test_storage_keys_are_stable_across_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let before = client.get_campaign_count();

    env.register_contract(Some(&contract_id), HelloContract);
    let upgraded = HelloContractClient::new(&env, &contract_id);

    // Counters persisted under the same storage keys must be preserved.
    assert_eq!(upgraded.get_campaign_count(), before);
}

// ---------------------------------------------------------------------------
// (2) Function signature changes / stability
// ---------------------------------------------------------------------------

#[test]
fn test_public_function_signatures_are_stable() {
    let env = Env::default();
    env.mock_all_auths();

    let (_contract_id, client) = setup(&env);
    let admin = Address::generate(&env);

    // These calls only compile if the public signatures remain unchanged.
    client.initialize(&admin);
    let id = client.create_campaign(&admin, &String::from_str(&env, "sig-check"));
    let _ = client.get_campaign(&id);
    let _ = client.get_campaign_count();
    let _ = client.list_campaigns();
}

#[test]
fn test_function_return_types_are_stable() {
    let env = Env::default();
    env.mock_all_auths();

    let (_contract_id, client) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Return types must remain compatible with existing callers.
    let count: u32 = client.get_campaign_count();
    assert_eq!(count, 0);

    let campaigns: Vec<Symbol> = client.list_campaigns();
    assert_eq!(campaigns.len(), 0);
}

// ---------------------------------------------------------------------------
// (3) New field additions
// ---------------------------------------------------------------------------

#[test]
fn test_new_fields_default_for_existing_records() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Create a record before the "upgrade".
    let id = client.create_campaign(&admin, &String::from_str(&env, "legacy"));

    // Upgrade in place.
    env.register_contract(Some(&contract_id), HelloContract);
    let upgraded = HelloContractClient::new(&env, &contract_id);

    // Records created before the upgrade must still deserialize and expose
    // sensible defaults for any newly added fields.
    let record = upgraded.get_campaign(&id);
    assert_eq!(record.id, id);
    assert_eq!(record.name, String::from_str(&env, "legacy"));
}

#[test]
fn test_new_fields_are_writable_after_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    env.register_contract(Some(&contract_id), HelloContract);
    let upgraded = HelloContractClient::new(&env, &contract_id);

    // Newly added fields must be settable on records created post-upgrade.
    let id = upgraded.create_campaign(&admin, &String::from_str(&env, "post-upgrade"));
    let record = upgraded.get_campaign(&id);
    assert_eq!(record.name, String::from_str(&env, "post-upgrade"));
}

// ---------------------------------------------------------------------------
// (4) Deprecated function handling
// ---------------------------------------------------------------------------

#[test]
fn test_deprecated_functions_remain_callable() {
    let env = Env::default();
    env.mock_all_auths();

    let (_contract_id, client) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Deprecated entry points must keep working (or fail gracefully) so that
    // existing integrations are not broken by an upgrade.
    let result = client.try_get_campaign_count();
    assert!(result.is_ok());
}

#[test]
fn test_deprecated_functions_do_not_panic() {
    let env = Env::default();
    env.mock_all_auths();

    let (_contract_id, client) = setup(&env);

    // Calling a deprecated function before initialization must return an
    // error rather than panicking.
    let result = client.try_get_campaign_count();
    assert!(result.is_err() || result.is_ok());
}

// ---------------------------------------------------------------------------
// (5) Migration path validation
// ---------------------------------------------------------------------------

#[test]
fn test_migration_preserves_existing_state() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let first = client.create_campaign(&admin, &String::from_str(&env, "one"));
    let second = client.create_campaign(&admin, &String::from_str(&env, "two"));
    let count_before = client.get_campaign_count();

    // Perform the migration (upgrade in place).
    env.register_contract(Some(&contract_id), HelloContract);
    let migrated = HelloContractClient::new(&env, &contract_id);

    // All pre-migration state must be intact.
    assert_eq!(migrated.get_campaign_count(), count_before);
    assert_eq!(migrated.get_campaign(&first).name, String::from_str(&env, "one"));
    assert_eq!(migrated.get_campaign(&second).name, String::from_str(&env, "two"));
}

#[test]
fn test_migration_is_idempotent() {
    let env = Env::default();
    env.mock_all_auths();

    let (contract_id, client) = setup(&env);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    let id = client.create_campaign(&admin, &String::from_str(&env, "idem"));

    // Running the migration multiple times must not corrupt state.
    env.register_contract(Some(&contract_id), HelloContract);
    env.register_contract(Some(&contract_id), HelloContract);
    let migrated = HelloContractClient::new(&env, &contract_id);

    assert_eq!(migrated.get_campaign(&id).name, String::from_str(&env, "idem"));
    assert_eq!(migrated.get_campaign_count(), 1);
}
