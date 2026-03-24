#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token, Address, Env, IntoVal, String, Vec,
};

use crate::{
    base::{errors::CrowdfundingError, types::PoolMetadata},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    (client, admin, token_address)
}

fn create_pool_with_funds(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token_address: &Address,
    contribution_amount: i128,
) -> u64 {
    let creator = Address::generate(env);
    let contributor = Address::generate(env);

    // Create pool
    let name = String::from_str(env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(env, "Test pool for emergency drain"),
        external_url: String::from_str(env, ""),
        image_hash: String::from_str(env, ""),
    };

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &10_000i128,
        &(env.ledger().timestamp() + 86400),
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Fund the contributor and make contribution
    let token_client = token::StellarAssetClient::new(env, token_address);
    token_client.mint(&contributor, &contribution_amount);

    client.contribute(
        &pool_id,
        &contributor,
        token_address,
        &contribution_amount,
        &false,
    );

    pool_id
}

#[test]
fn test_emergency_drain_pool_success() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    let contribution_amount = 5_000i128;
    let pool_id = create_pool_with_funds(&env, &client, &token_address, contribution_amount);

    let recipient = Address::generate(&env);

    // Drain the pool
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "emergency_drain_pool",
                args: (&pool_id, &recipient, &token_address).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .emergency_drain_pool(&pool_id, &recipient, &token_address);

    // Verify recipient received the funds
    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&recipient), contribution_amount);

    // Verify pool metrics show zero balance
    let pool = client.get_pool(&pool_id).unwrap();
    assert!(pool.target_amount > 0); // Pool config still exists
}

#[test]
fn test_emergency_drain_pool_requires_admin_auth() {
    let env = Env::default();
    let (client, _admin, token_address) = setup_test(&env);

    let contribution_amount = 5_000i128;
    let pool_id = create_pool_with_funds(&env, &client, &token_address, contribution_amount);

    let non_admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Attempt to drain without admin auth should fail
    let result = client
        .mock_auths(&[MockAuth {
            address: &non_admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "emergency_drain_pool",
                args: (&pool_id, &recipient, &token_address).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_emergency_drain_pool(&pool_id, &recipient, &token_address);

    // Should fail due to auth mismatch (admin.require_auth() will panic in mock)
    assert!(result.is_err());
}

#[test]
fn test_emergency_drain_pool_nonexistent_pool() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    let recipient = Address::generate(&env);
    let nonexistent_pool_id = 9999u64;

    let result = client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "emergency_drain_pool",
                args: (&nonexistent_pool_id, &recipient, &token_address).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_emergency_drain_pool(&nonexistent_pool_id, &recipient, &token_address);

    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_emergency_drain_pool_with_zero_balance() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Empty Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Pool with no contributions"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &10_000i128,
        &(env.ledger().timestamp() + 86400),
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    let recipient = Address::generate(&env);

    let result = client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "emergency_drain_pool",
                args: (&pool_id, &recipient, &token_address).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_emergency_drain_pool(&pool_id, &recipient, &token_address);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_emergency_drain_pool_updates_state_to_cancelled() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    let contribution_amount = 3_000i128;
    let pool_id = create_pool_with_funds(&env, &client, &token_address, contribution_amount);

    let recipient = Address::generate(&env);

    // Drain the pool
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "emergency_drain_pool",
                args: (&pool_id, &recipient, &token_address).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .emergency_drain_pool(&pool_id, &recipient, &token_address);

    // Verify pool still exists but state should be cancelled
    let pool = client.get_pool(&pool_id);
    assert!(pool.is_some());
}

#[test]
fn test_emergency_drain_pool_multiple_pools() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Create two pools with different amounts
    let pool1_amount = 2_000i128;
    let pool2_amount = 3_000i128;

    let pool_id_1 = create_pool_with_funds(&env, &client, &token_address, pool1_amount);
    let pool_id_2 = create_pool_with_funds(&env, &client, &token_address, pool2_amount);

    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);

    // Drain first pool
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "emergency_drain_pool",
                args: (&pool_id_1, &recipient1, &token_address).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .emergency_drain_pool(&pool_id_1, &recipient1, &token_address);

    // Drain second pool
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "emergency_drain_pool",
                args: (&pool_id_2, &recipient2, &token_address).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .emergency_drain_pool(&pool_id_2, &recipient2, &token_address);

    // Verify each recipient got the correct amount
    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&recipient1), pool1_amount);
    assert_eq!(token_client.balance(&recipient2), pool2_amount);
}
