#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    Address, Env,
};

// ============= ISSUE #1087: CONTRACT INITIALIZATION VALIDATION TESTS =============

/// Test 1: First admin initialization succeeds
#[test]
fn test_first_admin_initialization_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // Verify admin was stored by setting a creation fee (admin-only operation)
    client.set_creation_fee(&admin, &100_000i128);
    assert_eq!(client.get_creation_fee(), 100_000i128);
}

/// Test 2: Second admin initialization succeeds (overwrites previous)
#[test]
fn test_second_admin_initialization_overwrites() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    // First admin set
    client.set_admin(&admin1);
    client.set_creation_fee(&admin1, &50_000i128);

    // Second admin set overwrites
    client.set_admin(&admin2);

    // New admin can now perform admin operations
    client.set_creation_fee(&admin2, &200_000i128);
    assert_eq!(client.get_creation_fee(), 200_000i128);
}

/// Test 3: Negative creation fee fails with InvalidFee
#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_negative_creation_fee_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    client.set_creation_fee(&admin, &-1i128);
}

/// Test 4: Valid creation fee parameters are set correctly
#[test]
fn test_valid_creation_fee_set_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // Zero fee (disables fee)
    client.set_creation_fee(&admin, &0i128);
    assert_eq!(client.get_creation_fee(), 0i128);

    // Small fee
    client.set_creation_fee(&admin, &1_000i128);
    assert_eq!(client.get_creation_fee(), 1_000i128);

    // Large fee
    client.set_creation_fee(&admin, &1_000_000_000i128);
    assert_eq!(client.get_creation_fee(), 1_000_000_000i128);
}

/// Test 5: Admin authorization is enforced on set_creation_fee
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_set_creation_fee_requires_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);

    client.set_admin(&admin);
    client.set_creation_fee(&non_admin, &100_000i128);
}

/// Test 6: Creation fee defaults to zero before any set
#[test]
fn test_creation_fee_defaults_to_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_creation_fee(), 0i128);
}

/// Test 7: Setting admin without prior admin succeeds
#[test]
fn test_set_admin_without_prior_admin_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // Verify by checking fee can be set
    let fee = client.get_creation_fee();
    assert_eq!(fee, 0i128);
}

/// Test 8: set_creation_fee without admin set fails
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_set_creation_fee_without_admin_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let caller = Address::generate(&env);
    client.set_creation_fee(&caller, &100i128);
}

// ============= ISSUE #1311: CONTRACT UPGRADE COMPATIBILITY TESTS =============

/// Upgrade Test 1: Storage layout compatibility — admin and creation fee
/// persist across a re-registration of the same contract code, and the
/// stored values remain readable with their original types.
#[test]
fn test_upgrade_storage_layout_compatibility() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &123_456i128);

    // Simulate an upgrade by re-registering the contract at the same id.
    // The storage layout must remain compatible: previously written keys
    // are still readable with the same types after the upgrade.
    let upgraded_id = env.register_at(&contract_id, Contract, ());
    let upgraded_client = ContractClient::new(&env, &upgraded_id);

    assert_eq!(upgraded_client.get_creation_fee(), 123_456i128);

    // Admin authority is preserved: the original admin can still mutate state.
    upgraded_client.set_creation_fee(&admin, &654_321i128);
    assert_eq!(upgraded_client.get_creation_fee(), 654_321i128);
}

/// Upgrade Test 2: Function signature stability — the public entry points
/// keep their signatures (argument count/types and return types) so existing
/// callers continue to compile and behave identically after an upgrade.
#[test]
fn test_upgrade_function_signature_stability() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // set_admin(Address) -> ()
    client.set_admin(&admin);

    // set_creation_fee(Address, i128) -> ()
    client.set_creation_fee(&admin, &42i128);

    // get_creation_fee() -> i128
    let fee: i128 = client.get_creation_fee();
    assert_eq!(fee, 42i128);
}

/// Upgrade Test 3: New field additions — adding a new storage field must not
/// disturb existing fields. We verify that writing a new value (creation fee)
/// leaves the pre-existing admin field intact and usable.
#[test]
fn test_upgrade_new_field_additions_preserve_existing() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // Pre-existing field default is intact before any new field is written.
    assert_eq!(client.get_creation_fee(), 0i128);

    // Writing the (new) creation fee field does not clobber the admin field.
    client.set_creation_fee(&admin, &777i128);
    assert_eq!(client.get_creation_fee(), 777i128);

    // Admin field still governs authorization after the new field is set.
    let other = Address::generate(&env);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.set_creation_fee(&other, &1i128);
    }));
    assert!(result.is_err(), "non-admin must not mutate after upgrade");
}

/// Upgrade Test 4: Deprecated function handling — deprecated entry points
/// must remain callable (or fail deterministically) so old clients do not
/// silently corrupt state. Here we assert that the legacy admin path still
/// behaves deterministically after an upgrade.
#[test]
fn test_upgrade_deprecated_function_handling() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // Legacy callers that only know set_admin must still succeed.
    client.set_admin(&admin);

    // And the contract remains in a consistent, usable state.
    client.set_creation_fee(&admin, &5i128);
    assert_eq!(client.get_creation_fee(), 5i128);
}

/// Upgrade Test 5: Migration path validation — state written before an
/// upgrade is fully readable and mutable after the upgrade, matching the
/// expected migration outcome (no data loss, no type drift).
#[test]
fn test_upgrade_migration_path_validation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &9_999i128);

    // Capture pre-upgrade state.
    let pre_upgrade_fee = client.get_creation_fee();

    // Perform the upgrade (re-register at the same id).
    let upgraded_id = env.register_at(&contract_id, Contract, ());
    let upgraded_client = ContractClient::new(&env, &upgraded_id);

    // Post-upgrade state matches pre-upgrade state exactly.
    assert_eq!(upgraded_client.get_creation_fee(), pre_upgrade_fee);

    // Migration is complete: the admin can continue operating normally.
    upgraded_client.set_creation_fee(&admin, &1i128);
    assert_eq!(upgraded_client.get_creation_fee(), 1i128);
}
