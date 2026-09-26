#![cfg(test)]

// ============= ISSUE #332: ADMIN `register_school` IDENTITY MAPPING TESTS =============

use super::*;
use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, BytesN, Env, IntoVal, String, Symbol,
};

fn setup<'a>(env: &Env) -> (ContractClient<'a>, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    (client, admin)
}

/// The root admin can register a school and the metadata hash is persisted.
#[test]
fn test_admin_registers_school() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_admin(&admin);

    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[7u8; 32]);
    client.register_school(&school, &metadata_hash);

    assert!(client.is_school_registered(&school));
    assert_eq!(client.get_school_metadata(&school), metadata_hash);
}

/// Registration is authorized against the stored root admin address.
#[test]
fn test_register_school_requires_admin_auth() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_admin(&admin);

    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.register_school(&school, &metadata_hash);

    // The most recent authorization must be from the root admin invoking
    // `register_school`, proving the call is gated behind admin auth.
    let auths = env.auths();
    let (addr, invocation) = auths.last().unwrap().clone();
    assert_eq!(addr, admin);
    assert_eq!(
        invocation.function,
        AuthorizedFunction::Contract((
            client.address.clone(),
            Symbol::new(&env, "register_school"),
            (school, metadata_hash).into_val(&env),
        ))
    );
    let _: AuthorizedInvocation = invocation;
}

/// A caller who is not the stored admin cannot register a school.
#[test]
#[should_panic]
fn test_non_admin_cannot_register_school() {
    let env = Env::default();
    let (client, admin) = setup(&env);
    let attacker = Address::generate(&env);

    // Admin is set with its own auth mocked.
    env.mock_all_auths();
    client.set_admin(&admin);

    // Now only authorize the attacker; the stored-admin `require_auth()` must
    // fail because the attacker's signature does not satisfy it.
    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[2u8; 32]);
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &attacker,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &client.address,
            fn_name: "register_school",
            args: (school.clone(), metadata_hash.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.register_school(&school, &metadata_hash);
}

/// Re-registering an existing school overwrites its metadata hash in place.
#[test]
fn test_reregister_updates_metadata_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_admin(&admin);

    let school = Address::generate(&env);
    let first = BytesN::from_array(&env, &[3u8; 32]);
    let second = BytesN::from_array(&env, &[9u8; 32]);

    client.register_school(&school, &first);
    assert_eq!(client.get_school_metadata(&school), first);

    client.register_school(&school, &second);
    assert_eq!(client.get_school_metadata(&school), second);
    assert!(client.is_school_registered(&school));
}

/// Registration fails cleanly when no admin has been configured.
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_register_school_without_admin_panics() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin) = setup(&env);

    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[4u8; 32]);
    client.register_school(&school, &metadata_hash);
}

/// Unregistered schools report `false` and have no metadata.
#[test]
fn test_unregistered_school_is_not_registered() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_admin(&admin);

    let school = Address::generate(&env);
    assert!(!client.is_school_registered(&school));
}

/// A registered school can back a sponsorship pool via `create_pool_for_school`.
#[test]
fn test_registered_school_enables_pool_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    client.set_admin(&admin);

    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[5u8; 32]);
    client.register_school(&school, &metadata_hash);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Registered School Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &school,
        &100_000u64,
    );

    assert_eq!(client.get_pool_school(&pool_id), school);
}
