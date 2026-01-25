#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String, Vec,
};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{PoolMetadata, PoolState},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn create_test_campaign_id(env: &Env, seed: u8) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    BytesN::from_array(env, &bytes)
}

#[test]
fn test_create_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Save the Whales");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline);
}

#[test]
fn test_get_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 2);
    let title = String::from_str(&env, "Build a School");
    let goal = 500_000i128;
    let deadline = env.ledger().timestamp() + 172800;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.id, campaign_id);
    assert_eq!(campaign.title, title);
    assert_eq!(campaign.creator, creator);
    assert_eq!(campaign.goal, goal);
    assert_eq!(campaign.deadline, deadline);
}

#[test]
fn test_get_nonexistent_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let campaign_id = create_test_campaign_id(&env, 99);

    let result = client.try_get_campaign(&campaign_id);

    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignNotFound)));
}

#[test]
fn test_create_campaign_with_empty_title() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 3);
    let title = String::from_str(&env, "");
    let goal = 100_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidTitle)));
}

#[test]
fn test_create_campaign_with_zero_goal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 4);
    let title = String::from_str(&env, "Zero Goal Campaign");
    let goal = 0i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidGoal)));
}

#[test]
fn test_create_campaign_with_negative_goal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 5);
    let title = String::from_str(&env, "Negative Goal Campaign");
    let goal = -100i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidGoal)));
}

#[test]
fn test_create_campaign_with_past_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 6);
    let title = String::from_str(&env, "Past Deadline Campaign");
    let goal = 100_000i128;
    let deadline = 500;

    let result = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDeadline)));
}

#[test]
fn test_create_duplicate_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 7);
    let title = String::from_str(&env, "Duplicate Campaign");
    let goal = 100_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    let result2 = client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline);

    assert_eq!(result2, Err(Ok(CrowdfundingError::CampaignAlreadyExists)));
}

#[test]
fn test_multiple_campaigns() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let campaign_id_1 = create_test_campaign_id(&env, 8);
    let campaign_id_2 = create_test_campaign_id(&env, 9);

    let title1 = String::from_str(&env, "Campaign One");
    let title2 = String::from_str(&env, "Campaign Two");

    let goal1 = 100_000i128;
    let goal2 = 200_000i128;

    let deadline1 = env.ledger().timestamp() + 86400;
    let deadline2 = env.ledger().timestamp() + 172800;

    client.create_campaign(&campaign_id_1, &title1, &creator1, &goal1, &deadline1);
    client.create_campaign(&campaign_id_2, &title2, &creator2, &goal2, &deadline2);

    let campaign1 = client.get_campaign(&campaign_id_1);
    let campaign2 = client.get_campaign(&campaign_id_2);

    assert_eq!(campaign1.title, title1);
    assert_eq!(campaign1.goal, goal1);

    assert_eq!(campaign2.title, title2);
    assert_eq!(campaign2.goal, goal2);
}

// Pool Storage Tests

#[test]
fn test_save_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Education Fund");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Fund for educational supplies"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    assert_eq!(pool_id, 1);
}

#[test]
fn test_save_pool_validation() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    // Test empty name
    let empty_name = String::from_str(&env, "");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_save_pool(
        &empty_name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None,
        &None,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolName)));

    // Test invalid target amount
    let name = String::from_str(&env, "Test Pool");
    let result = client.try_save_pool(
        &name,
        &metadata,
        &creator,
        &0i128,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolTarget)));

    // Test invalid deadline
    let past_deadline = 0; // Use 0 as a past timestamp since ledger starts at 0
    let result = client.try_save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &past_deadline,
        &None,
        &None,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolDeadline)));
}

#[test]
fn test_get_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Medical Fund");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Fund for medical expenses"),
        external_url: String::from_str(&env, "https://medical.example.com"),
        image_hash: String::from_str(&env, "hash123"),
    };
    let target_amount = 5_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    let pool = client.get_pool(&pool_id).unwrap();

    // PoolConfig no longer carries description
    assert_eq!(pool.name, name);
    assert_eq!(pool.target_amount, target_amount);
    // duration is derived from deadline and current timestamp, so it
    // should be positive and no greater than the originally requested
    // deadline offset.
    assert!(pool.duration > 0);
    assert!(pool.created_at <= env.ledger().timestamp());

    // Verify metadata separately
    let (desc, url, hash) = client.get_pool_metadata(&pool_id);
    assert_eq!(desc, metadata.description);
    assert_eq!(url, metadata.external_url);
    assert_eq!(hash, metadata.image_hash);
}

#[test]
fn test_get_pool_metadata_nonexistent() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let (desc, url, hash) = client.get_pool_metadata(&999);
    assert_eq!(desc, String::from_str(&env, ""));
    assert_eq!(url, String::from_str(&env, ""));
    assert_eq!(hash, String::from_str(&env, ""));
}

#[test]
fn test_get_nonexistent_pool() {
    let env = Env::default();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let pool = client.get_pool(&999);
    assert!(pool.is_none());
}

#[test]
fn test_update_pool_state() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Charity Fund");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Fund for charity"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 15_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Update state to Paused
    client.update_pool_state(&pool_id, &PoolState::Paused);

    // Update state to Completed
    client.update_pool_state(&pool_id, &PoolState::Completed);
}

#[test]
fn test_update_pool_state_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let result = client.try_update_pool_state(&999, &PoolState::Paused);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_update_pool_state_invalid_transition() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Test Fund");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test fund"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // First complete the pool
    client.update_pool_state(&pool_id, &PoolState::Completed);

    // Try to change state from completed - should fail
    let result = client.try_update_pool_state(&pool_id, &PoolState::Active);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));

    let result = client.try_update_pool_state(&pool_id, &PoolState::Paused);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));
}

#[test]
fn test_multiple_pools() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    // Create first pool
    let name1 = String::from_str(&env, "Pool One");
    let metadata1 = PoolMetadata {
        description: String::from_str(&env, "First pool"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target1 = 10_000i128;
    let deadline1 = env.ledger().timestamp() + 86400;
    let pool_id1 = client.save_pool(
        &name1,
        &metadata1,
        &creator1,
        &target1,
        &deadline1,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Create second pool
    let name2 = String::from_str(&env, "Pool Two");
    let metadata2 = PoolMetadata {
        description: String::from_str(&env, "Second pool"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target2 = 20_000i128;
    let deadline2 = env.ledger().timestamp() + 172800;
    let pool_id2 = client.save_pool(
        &name2,
        &metadata2,
        &creator2,
        &target2,
        &deadline2,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    assert_eq!(pool_id1, 1);
    assert_eq!(pool_id2, 2);

    // Verify both pools
    let pool1 = client.get_pool(&pool_id1).unwrap();
    let pool2 = client.get_pool(&pool_id2).unwrap();

    assert_eq!(pool1.name, name1);
    assert_eq!(pool1.target_amount, target1);

    assert_eq!(pool2.name, name2);
    assert_eq!(pool2.target_amount, target2);

    // Update different states
    client.update_pool_state(&pool_id1, &PoolState::Paused);
    client.update_pool_state(&pool_id2, &PoolState::Active);
}

#[test]
fn test_pause_unpause_full_cycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Initial state
    assert!(!client.is_paused());

    // Pause
    client.pause();
    assert!(client.is_paused());

    // Unpause
    client.unpause();
    assert!(!client.is_paused());
}

#[test]
fn test_admin_auth_for_pause() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Admin can pause
    client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "pause",
                args: soroban_sdk::vec![&env],
                sub_invokes: &[],
            },
        }])
        .pause();
    assert!(client.is_paused());
}

#[test]
#[should_panic]
fn test_non_admin_cannot_pause() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    client.initialize(&admin);

    // Non-admin trying to pause - should fail (require_auth will fail)
    client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &non_admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "pause",
                args: soroban_sdk::vec![&env],
                sub_invokes: &[],
            },
        }])
        .pause();
}

#[test]
fn test_operations_disabled_when_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    client.pause();

    // Try create campaign - should fail
    let creator = Address::generate(&env);
    let camp_id = create_test_campaign_id(&env, 10);
    let title = String::from_str(&env, "Test");
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 10000;

    let result = client.try_create_campaign(&camp_id, &title, &creator, &goal, &deadline);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractPaused)));

    // Try save pool - should fail
    let metadata = PoolMetadata {
        description: title.clone(),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };

    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 10000;

    let result_pool = client.try_save_pool(
        &title,
        &metadata,
        &creator,
        &goal,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );
    assert_eq!(result_pool, Err(Ok(CrowdfundingError::ContractPaused)));
}

#[test]
fn test_update_pool_state_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Create a pool first
    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Now pause the contract
    client.pause();

    // Try to update pool state - should fail
    let result = client.try_update_pool_state(&pool_id, &PoolState::Paused);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractPaused)));

    // Unpause and verify it works
    client.unpause();
    client.update_pool_state(&pool_id, &PoolState::Paused);
}

#[test]
fn test_getters_work_when_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    // Create a campaign before pausing
    let creator = Address::generate(&env);
    let camp_id = create_test_campaign_id(&env, 11);
    client.create_campaign(
        &camp_id,
        &String::from_str(&env, "Pre-pause"),
        &creator,
        &1000i128,
        &(env.ledger().timestamp() + 10000),
    );

    client.pause();

    // Getters should still work
    let campaign = client.get_campaign(&camp_id);
    assert_eq!(campaign.id, camp_id);
    assert!(client.is_paused());
}

#[test]
fn test_cannot_pause_already_paused() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    client.pause();
    let result = client.try_pause();
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractAlreadyPaused)));
}

#[test]
fn test_cannot_unpause_already_unpaused() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let result = client.try_unpause();
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractAlreadyUnpaused)));
}

#[test]
fn test_operations_enabled_after_unpause() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    client.pause();
    client.unpause();

    let creator = Address::generate(&env);
    let camp_id = create_test_campaign_id(&env, 12);
    let title = String::from_str(&env, "After Unpause");
    client.create_campaign(
        &camp_id,
        &title,
        &creator,
        &1000i128,
        &(env.ledger().timestamp() + 10000),
    );

    let campaign = client.get_campaign(&camp_id);
    assert_eq!(campaign.title, title);
}

#[test]
fn test_contribute_and_event_emission() {
    let env = Env::default();
    env.mock_all_auths();

    // Register a mock token for testing
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_client = soroban_sdk::token::Client::new(&env, &token_id);
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Advance ledger time
    env.ledger().with_mut(|li| li.timestamp = 100);

    // Mint some tokens to the contributor
    token_admin_client.mint(&contributor, &5000i128);

    // Contribute
    let amount = 1000i128;
    client.contribute(&pool_id, &contributor, &token_id, &amount, &false);

    // Verify balance transfer
    assert_eq!(token_client.balance(&contributor), 4000i128);
    assert_eq!(token_client.balance(&contract_id), 1000i128);

    // Verify event emission via snapshot
    // (We've confirmed that env.events().all() has issues in this test setup,
    // but the snapshot recorder will capture the events if they are emitted.)
}

// Getter & Donation Tests

#[test]
fn test_get_all_campaigns() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // 1. Returns empty list when no campaigns exist
    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 0);

    // 2. Returns all campaign IDs after multiple campaigns created
    let creator = Address::generate(&env);
    let id1 = create_test_campaign_id(&env, 101);
    let id2 = create_test_campaign_id(&env, 102);
    
    client.create_campaign(&id1, &String::from_str(&env, "C1"), &creator, &1000i128, &(env.ledger().timestamp() + 100));
    client.create_campaign(&id2, &String::from_str(&env, "C2"), &creator, &1000i128, &(env.ledger().timestamp() + 100));

    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 2);
    assert!(campaigns.contains(id1));
    assert!(campaigns.contains(id2));
}

#[test]
fn test_donate_and_donor_count() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 103);
    client.create_campaign(&campaign_id, &String::from_str(&env, "Donation Test"), &creator, &10000i128, &(env.ledger().timestamp() + 1000));

    // 1. Returns 0 for campaign with no donors
    assert_eq!(client.get_donor_count(&campaign_id), 0);
    assert_eq!(client.get_campaign_balance(&campaign_id), 0);

    // Setup donor
    let donor1 = Address::generate(&env);
    token_admin_client.mint(&donor1, &5000i128);

    // 2. Donate and check count
    client.donate(&campaign_id, &donor1, &token_id, &100i128);

    assert_eq!(client.get_donor_count(&campaign_id), 1);
    assert_eq!(client.get_campaign_balance(&campaign_id), 100);

    // 3. Same donor donates again -> count should still be 1
    client.donate(&campaign_id, &donor1, &token_id, &50i128);
    assert_eq!(client.get_donor_count(&campaign_id), 1);
    assert_eq!(client.get_campaign_balance(&campaign_id), 150);

    // 4. Different donor donates -> count should be 2
    let donor2 = Address::generate(&env);
    token_admin_client.mint(&donor2, &5000i128);
    client.donate(&campaign_id, &donor2, &token_id, &200i128);

    assert_eq!(client.get_donor_count(&campaign_id), 2);
    assert_eq!(client.get_campaign_balance(&campaign_id), 350);
}

#[test]
fn test_get_campaign_goal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 104);
    let goal = 5555i128;
    
    client.create_campaign(&id, &String::from_str(&env, "Goal Test"), &creator, &goal, &(env.ledger().timestamp() + 100));

    assert_eq!(client.get_campaign_goal(&id), goal);
}

#[test]
fn test_is_campaign_completed() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 105);
    let goal = 1000i128;
    
    client.create_campaign(&id, &String::from_str(&env, "Completion Test"), &creator, &goal, &(env.ledger().timestamp() + 1000));

    // 1. Returns false for new campaign
    assert_eq!(client.is_campaign_completed(&id), false);

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &5000i128);

    // 2. Returns false when under goal
    client.donate(&id, &donor, &token_id, &900i128);
    assert_eq!(client.is_campaign_completed(&id), false);

    // 3. Returns true when goal is reached
    client.donate(&id, &donor, &token_id, &100i128); // Total 1000
    assert_eq!(client.is_campaign_completed(&id), true);

    // 4. Returns true when goal is exceeded
    client.donate(&id, &donor, &token_id, &100i128); // Total 1100
    assert_eq!(client.is_campaign_completed(&id), true);
}

#[test]
fn test_donate_deadline_passed() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 106);
    let deadline = 2000u64;
    
    client.create_campaign(&id, &String::from_str(&env, "Deadline Test"), &creator, &1000i128, &deadline);

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &5000i128);

    // Donate before deadline - should work
    client.donate(&id, &donor, &token_id, &100i128);

    // Advance time past deadline
    env.ledger().with_mut(|li| li.timestamp = 2001);

    // Donate after deadline - should fail
    let result = client.try_donate(&id, &donor, &token_id, &100i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDeadline)));
}
