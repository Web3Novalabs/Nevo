use soroban_sdk::{
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal,
};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolConfig, PoolState, StorageKey},
    },
    crowdfunding::CrowdfundingContract,
    test::create_test_token,
};

#[test]
fn test_remove_school_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let school_validator = Address::generate(&env);
    let token = create_test_token(&env, &admin);

    let contract = CrowdfundingContract;
    let client = contract.client(&env);

    // Initialize contract
    client.initialize(&admin, &token, &100i128, &1000u32);

    // Create a pool with the school as validator
    let pool_id = client.create_pool(
        &admin,
        &"Test Pool".into_val(&env),
        &"Test Description".into_val(&env),
        &1000i128,
        &100i128,
        &(env.ledger().timestamp() + 86400),
        &school_validator,
        &false,
    );

    // Verify pool exists
    let pool = client.get_pool(&pool_id);
    assert!(pool.is_some());
    assert_eq!(pool.unwrap().validator, school_validator);

    // Remove the school
    client.remove_school(&school_validator);

    // Verify pool and associated data are removed
    let pool_after = client.get_pool(&pool_id);
    assert!(pool_after.is_none());

    // Verify the correct authorization was required
    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract.address(&env),
                    "remove_school".into_val(&env),
                    (school_validator.clone(),).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

#[test]
fn test_remove_school_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);
    let school_validator = Address::generate(&env);
    let token = create_test_token(&env, &admin);

    let contract = CrowdfundingContract;
    let client = contract.client(&env);

    // Initialize contract
    client.initialize(&admin, &token, &100i128, &1000u32);

    // Create a pool with the school as validator
    client.create_pool(
        &admin,
        &"Test Pool".into_val(&env),
        &"Test Description".into_val(&env),
        &1000i128,
        &100i128,
        &(env.ledger().timestamp() + 86400),
        &school_validator,
        &false,
    );

    // Try to remove school as unauthorized user - should fail
    env.set_auths(&[]);
    let result = client.try_remove_school(&school_validator);
    assert!(result.is_err());
}

#[test]
fn test_remove_school_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let nonexistent_school = Address::generate(&env);
    let token = create_test_token(&env, &admin);

    let contract = CrowdfundingContract;
    let client = contract.client(&env);

    // Initialize contract
    client.initialize(&admin, &token, &100i128, &1000u32);

    // Try to remove a school that doesn't exist
    let result = client.try_remove_school(&nonexistent_school);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_remove_school_with_active_contributions() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let school_validator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let token = create_test_token(&env, &admin);

    let contract = CrowdfundingContract;
    let client = contract.client(&env);

    // Initialize contract
    client.initialize(&admin, &token, &100i128, &1000u32);

    // Create a pool with the school as validator
    let pool_id = client.create_pool(
        &admin,
        &"Test Pool".into_val(&env),
        &"Test Description".into_val(&env),
        &1000i128,
        &100i128,
        &(env.ledger().timestamp() + 86400),
        &school_validator,
        &false,
    );

    // Make a contribution to the pool
    client.contribute(&pool_id, &contributor, &token, &500i128, &false);

    // Try to remove school with active contributions - should fail
    let result = client.try_remove_school(&school_validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));
}

#[test]
fn test_remove_school_closed_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let school_validator = Address::generate(&env);
    let token = create_test_token(&env, &admin);

    let contract = CrowdfundingContract;
    let client = contract.client(&env);

    // Initialize contract
    client.initialize(&admin, &token, &100i128, &1000u32);

    // Create a pool with the school as validator
    let pool_id = client.create_pool(
        &admin,
        &"Test Pool".into_val(&env),
        &"Test Description".into_val(&env),
        &1000i128,
        &100i128,
        &(env.ledger().timestamp() + 86400),
        &school_validator,
        &false,
    );

    // Close the pool
    client.close_pool(&pool_id, &admin);

    // Now removing the school should succeed
    client.remove_school(&school_validator);

    // Verify pool is removed
    let pool_after = client.get_pool(&pool_id);
    assert!(pool_after.is_none());
}

#[test]
fn test_remove_school_completed_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let school_validator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let token = create_test_token(&env, &admin);

    let contract = CrowdfundingContract;
    let client = contract.client(&env);

    // Initialize contract
    client.initialize(&admin, &token, &100i128, &1000u32);

    // Create a pool with the school as validator
    let pool_id = client.create_pool(
        &admin,
        &"Test Pool".into_val(&env),
        &"Test Description".into_val(&env),
        &1000i128,
        &100i128,
        &(env.ledger().timestamp() + 86400),
        &school_validator,
        &false,
    );

    // Contribute to reach the target
    client.contribute(&pool_id, &contributor, &token, &1000i128, &false);

    // Update pool state to completed
    env.storage().instance().set(
        &StorageKey::PoolState(pool_id),
        &PoolState::Completed,
    );

    // Try to remove school with completed pool - should fail
    let result = client.try_remove_school(&school_validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));
}

#[test]
fn test_remove_school_multiple_pools_same_validator() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let school_validator = Address::generate(&env);
    let token = create_test_token(&env, &admin);

    let contract = CrowdfundingContract;
    let client = contract.client(&env);

    // Initialize contract
    client.initialize(&admin, &token, &100i128, &1000u32);

    // Create multiple pools with the same validator
    let pool_id1 = client.create_pool(
        &admin,
        &"Test Pool 1".into_val(&env),
        &"Test Description 1".into_val(&env),
        &1000i128,
        &100i128,
        &(env.ledger().timestamp() + 86400),
        &school_validator,
        &false,
    );

    let pool_id2 = client.create_pool(
        &admin,
        &"Test Pool 2".into_val(&env),
        &"Test Description 2".into_val(&env),
        &1000i128,
        &100i128,
        &(env.ledger().timestamp() + 86400),
        &school_validator,
        &false,
    );

    // Remove the school - should only remove the first pool found
    client.remove_school(&school_validator);

    // One pool should be removed, the other should still exist
    let pool1_after = client.get_pool(&pool_id1);
    let pool2_after = client.get_pool(&pool_id2);
    
    // The implementation removes the first pool found
    assert!(pool1_after.is_none());
    assert!(pool2_after.is_some());
}

#[test]
fn test_remove_school_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let school_validator = Address::generate(&env);

    let contract = CrowdfundingContract;
    let client = contract.client(&env);

    // Try to remove school without initializing contract
    let result = client.try_remove_school(&school_validator);
    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));
}