#![cfg(test)]

// ============= ISSUE #1339: `get_pool_school` ASSOCIATION INTEGRITY TESTS =============

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup(env: &Env) -> ContractClient<'_> {
    let contract_id = env.register(Contract, ());
    ContractClient::new(env, &contract_id)
}

fn register_school(env: &Env, client: &ContractClient<'_>, admin: &Address) -> Address {
    client.set_admin(admin);
    let school = Address::generate(env);
    let metadata_hash = BytesN::from_array(env, &[8u8; 32]);
    client.register_school(&school, &metadata_hash);
    school
}

/// `get_pool_school` returns the correct school address for a pool created
/// via `create_pool_for_school`.
#[test]
fn test_get_pool_school_returns_correct_school_for_school_created_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let admin = Address::generate(&env);
    let school = register_school(&env, &client, &admin);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &school,
        &200_000u64,
    );

    assert_eq!(client.get_pool_school(&pool_id), school);
}

/// A pool created via the non-school `create_pool` path has no school
/// association recorded, so `get_pool_school` panics.
#[test]
#[should_panic(expected = "Pool school not set")]
fn test_get_pool_school_panics_for_pool_created_via_create_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Non-School Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &200_000u64,
    );

    client.get_pool_school(&pool_id);
}

/// `get_pool_school` panics for a pool id that was never created at all.
#[test]
#[should_panic(expected = "Pool school not set")]
fn test_get_pool_school_panics_for_nonexistent_pool_id() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

    client.get_pool_school(&999u32);
}

/// The pool/school association is set once at creation and is not affected
/// by later, unrelated pool creations.
#[test]
fn test_pool_school_association_immutable_after_creation() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let admin = Address::generate(&env);
    let school_a = register_school(&env, &client, &admin);
    let school_b = Address::generate(&env);
    let metadata_hash_b = BytesN::from_array(&env, &[9u8; 32]);
    client.register_school(&school_b, &metadata_hash_b);

    let creator = Address::generate(&env);
    let pool_id_a = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool A"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &school_a,
        &200_000u64,
    );
    assert_eq!(client.get_pool_school(&pool_id_a), school_a);

    // Creating a second, unrelated pool for a different school must not
    // change pool_id_a's stored association.
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool B"),
        &String::from_str(&env, "Description"),
        &2_000_000u128,
        &school_b,
        &200_000u64,
    );
    assert_eq!(client.get_pool_school(&pool_id_a), school_a);
}

/// `get_pool_school` resolves correctly across multiple pools linked to the
/// same school.
#[test]
fn test_get_pool_school_correct_across_multiple_pools_per_school() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let admin = Address::generate(&env);
    let school = register_school(&env, &client, &admin);

    let creator = Address::generate(&env);
    let pool_id_1 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "First Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000u128,
        &school,
        &200_000u64,
    );
    let pool_id_2 = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Second Pool"),
        &String::from_str(&env, "Description"),
        &2_000_000u128,
        &school,
        &200_000u64,
    );

    assert_eq!(client.get_pool_school(&pool_id_1), school);
    assert_eq!(client.get_pool_school(&pool_id_2), school);
    assert_ne!(pool_id_1, pool_id_2);
}
