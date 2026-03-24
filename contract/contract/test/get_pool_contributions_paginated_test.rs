#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, Env, String, Vec};

use crate::{
    base::types::PoolConfig,
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::StellarAssetClient<'a> {
    let token_address = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    token::StellarAssetClient::new(env, &token_address)
}

fn setup_contract(
    env: &Env,
) -> (
    CrowdfundingContractClient,
    Address,
    token::StellarAssetClient,
) {
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_client = create_token_contract(env, &token_admin);

    client.initialize(&admin, &token_client.address, &0);

    (client, admin, token_client)
}

#[test]
fn test_get_pool_contributions_paginated_with_10_contributors() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, token_client) = setup_contract(&env);

    // Create a pool
    let creator = Address::generate(&env);
    let pool_config = PoolConfig {
        name: String::from_str(&env, "Test Pool"),
        description: String::from_str(&env, "A test pool for pagination"),
        target_amount: 10_000_000,
        min_contribution: 1000,
        is_private: false,
        duration: 30 * 24 * 60 * 60, // 30 days
        created_at: env.ledger().timestamp(),
    };

    let pool_id = client.create_pool(&creator, &pool_config);

    // Create 10 contributors and have them contribute
    let mut contributors = Vec::new(&env);
    for i in 0..10 {
        let contributor = Address::generate(&env);
        let amount = (i + 1) as i128 * 100_000; // Different amounts for each contributor

        // Mint tokens to contributor
        token_client.mint(&contributor, &amount);

        // Contribute to the pool
        client.contribute(
            &pool_id,
            &contributor,
            &token_client.address,
            &amount,
            &false,
        );

        contributors.push_back(contributor);
    }

    // Test: Fetch first batch of 5 contributions (offset=0, limit=5)
    let first_batch = client.get_pool_contributions_paginated(&pool_id, &0, &5);
    assert_eq!(first_batch.len(), 5);

    // Verify the first batch contains the correct contributors
    for i in 0..5 {
        let contribution = first_batch.get(i as u32).unwrap();
        let expected_contributor = contributors.get(i as u32).unwrap();
        assert_eq!(contribution.contributor, expected_contributor);
        assert_eq!(contribution.pool_id, pool_id);
        assert_eq!(contribution.amount, (i + 1) as i128 * 100_000);
    }

    // Test: Fetch second batch of 5 contributions (offset=5, limit=5)
    let second_batch = client.get_pool_contributions_paginated(&pool_id, &5, &5);
    assert_eq!(second_batch.len(), 5);

    // Verify the second batch contains the correct contributors
    for i in 0..5 {
        let contribution = second_batch.get(i as u32).unwrap();
        let expected_contributor = contributors.get((i + 5) as u32).unwrap();
        assert_eq!(contribution.contributor, expected_contributor);
        assert_eq!(contribution.pool_id, pool_id);
        assert_eq!(contribution.amount, (i + 6) as i128 * 100_000);
    }

    // Test: Fetch with offset beyond total (should return empty)
    let empty_batch = client.get_pool_contributions_paginated(&pool_id, &10, &5);
    assert_eq!(empty_batch.len(), 0);

    // Test: Fetch with limit larger than remaining (should return only remaining)
    let partial_batch = client.get_pool_contributions_paginated(&pool_id, &8, &5);
    assert_eq!(partial_batch.len(), 2);
}

#[test]
fn test_get_pool_contributions_paginated_empty_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _token_client) = setup_contract(&env);

    // Create a pool with no contributions
    let creator = Address::generate(&env);
    let pool_config = PoolConfig {
        name: String::from_str(&env, "Empty Pool"),
        description: String::from_str(&env, "A pool with no contributions"),
        target_amount: 5_000_000,
        min_contribution: 1000,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: env.ledger().timestamp(),
    };

    let pool_id = client.create_pool(&creator, &pool_config);

    // Test: Fetch from empty pool
    let result = client.get_pool_contributions_paginated(&pool_id, &0, &5);
    assert_eq!(result.len(), 0);
}

#[test]
#[should_panic]
fn test_get_pool_contributions_paginated_nonexistent_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _token_client) = setup_contract(&env);

    // Try to fetch contributions from a non-existent pool
    client.get_pool_contributions_paginated(&999, &0, &5);
}

#[test]
fn test_get_pool_contributions_paginated_single_contributor_multiple_contributions() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, token_client) = setup_contract(&env);

    // Create a pool
    let creator = Address::generate(&env);
    let pool_config = PoolConfig {
        name: String::from_str(&env, "Test Pool"),
        description: String::from_str(&env, "Test multiple contributions"),
        target_amount: 10_000_000,
        min_contribution: 1000,
        is_private: false,
        duration: 30 * 24 * 60 * 60,
        created_at: env.ledger().timestamp(),
    };

    let pool_id = client.create_pool(&creator, &pool_config);

    // Single contributor makes multiple contributions
    let contributor = Address::generate(&env);
    token_client.mint(&contributor, &1_000_000);

    client.contribute(
        &pool_id,
        &contributor,
        &token_client.address,
        &300_000,
        &false,
    );
    client.contribute(
        &pool_id,
        &contributor,
        &token_client.address,
        &400_000,
        &false,
    );
    client.contribute(
        &pool_id,
        &contributor,
        &token_client.address,
        &300_000,
        &false,
    );

    // Should only have 1 entry (contributor appears once with total amount)
    let result = client.get_pool_contributions_paginated(&pool_id, &0, &10);
    assert_eq!(result.len(), 1);

    let contribution = result.get(0).unwrap();
    assert_eq!(contribution.contributor, contributor);
    assert_eq!(contribution.amount, 1_000_000); // Sum of all contributions
}
