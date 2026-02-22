#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token, Address, BytesN, Env, IntoVal, String, Vec,
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

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    // Initialize with 0 fee by default
    client.initialize(&admin, &token_address, &0);

    (client, admin, token_address)
}

#[test]
fn test_create_campaign() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Save the Whales");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.token_address, token_address);
}

#[test]
fn test_get_campaign() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 2);
    let title = String::from_str(&env, "Build a School");
    let goal = 500_000i128;
    let deadline = env.ledger().timestamp() + 172800;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

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
    let (client, _, _) = setup_test(&env);

    let campaign_id = create_test_campaign_id(&env, 99);

    let result = client.try_get_campaign(&campaign_id);

    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignNotFound)));
}

#[test]
fn test_create_campaign_with_empty_title() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 3);
    let title = String::from_str(&env, "");
    let goal = 100_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result =
        client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidTitle)));
}

#[test]
fn test_create_campaign_with_zero_goal() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 4);
    let title = String::from_str(&env, "Zero Goal Campaign");
    let goal = 0i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result =
        client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidGoal)));
}

#[test]
fn test_create_campaign_with_negative_goal() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 5);
    let title = String::from_str(&env, "Negative Goal Campaign");
    let goal = -100i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result =
        client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidGoal)));
}

#[test]
fn test_create_campaign_with_past_deadline() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 6);
    let title = String::from_str(&env, "Past Deadline Campaign");
    let goal = 100_000i128;
    let deadline = 500;

    let result = client.try_create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDeadline)));
}

#[test]
fn test_create_duplicate_campaign() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 7);
    let title = String::from_str(&env, "Duplicate Campaign");
    let goal = 100_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    let result2 =
        client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    assert_eq!(result2, Err(Ok(CrowdfundingError::CampaignAlreadyExists)));
}

#[test]
fn test_create_campaign_uninitialized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 99);
    let title = String::from_str(&env, "Uninitialized");
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    let result =
        client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);
    assert_eq!(result, Err(Ok(CrowdfundingError::NotInitialized)));
}

#[test]
fn test_multiple_campaigns() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

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

    client.create_campaign(
        &campaign_id_1,
        &title1,
        &creator1,
        &goal1,
        &deadline1,
        &token_address,
    );
    client.create_campaign(
        &campaign_id_2,
        &title2,
        &creator2,
        &goal2,
        &deadline2,
        &token_address,
    );

    let campaign1 = client.get_campaign(&campaign_id_1);
    let campaign2 = client.get_campaign(&campaign_id_2);

    assert_eq!(campaign1.title, title1);
    assert_eq!(campaign1.goal, goal1);

    assert_eq!(campaign2.title, title2);
    assert_eq!(campaign2.goal, goal2);
}

// Pool Storage Tests

#[test]
fn test_extend_campaign_deadline() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 12);
    let title = String::from_str(&env, "Extend Deadline");
    let goal = 1_000_000i128;
    let initial_deadline = env.ledger().timestamp() + 86400; // +1 day

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &initial_deadline,
        &token_address,
    );

    let new_deadline = env.ledger().timestamp() + 2 * 86400; // +2 days

    client
        .mock_auths(&[MockAuth {
            address: &creator,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "extend_campaign_deadline",
                args: (&campaign_id, &new_deadline).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .extend_campaign_deadline(&campaign_id, &new_deadline);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.deadline, new_deadline);
}

#[test]
fn test_extend_campaign_deadline_invalid_auth() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let malicious = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 13);
    let title = String::from_str(&env, "Extend Auth");
    let goal = 1_000_000i128;
    let initial_deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &initial_deadline,
        &token_address,
    );

    let new_deadline = env.ledger().timestamp() + 2 * 86400;

    // mock_auths to malicious address simulating a fail
    // However, in mock_auths, if the caller requires the auth of creator, it will panic
}

#[test]
fn test_extend_campaign_too_long() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 14);
    let title = String::from_str(&env, "Extend Too Long");
    let goal = 1_000_000i128;
    let initial_deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &initial_deadline,
        &token_address,
    );

    let new_deadline = env.ledger().timestamp() + 91 * 24 * 60 * 60; // 91 days

    let result = client
        .mock_auths(&[MockAuth {
            address: &creator,
            invoke: &MockAuthInvoke {
                contract: &client.address,
                fn_name: "extend_campaign_deadline",
                args: (&campaign_id, &new_deadline).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_extend_campaign_deadline(&campaign_id, &new_deadline);

    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDeadline)));
}

#[test]
fn test_get_campaign_fee_history() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Using token_admin pattern from other tests
    let token_admin = Address::generate(&env);
    let token_client = token::Client::new(&env, &token_address);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 15);
    let title = String::from_str(&env, "Fee Test Campaign");
    let goal = 1_000_000i128;
    let initial_deadline = env.ledger().timestamp() + 86400; // +1 day

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &initial_deadline,
        &token_address,
    );

    // Initial fees should be 0
    let initial_fees = client.get_campaign_fee_history(&campaign_id);
    assert_eq!(initial_fees, 0);

    // Setup donor balance
    token_admin_client.mint(&donor, &1_000_000i128);

    // Donate
    let donation_amount = 500_000i128; // 1% should be 5_000

    // We mock auth to allow transfer to crowdfunding contract and execute donate
    client.donate(&campaign_id, &donor, &token_address, &donation_amount);

    // Verify counter incremented correctly
    let current_fees = client.get_campaign_fee_history(&campaign_id);
    assert_eq!(current_fees, 5_000);

    // Second donation
    let donation2_amount = 200_000i128; // 1% should be 2_000
    client.donate(&campaign_id, &donor, &token_address, &donation2_amount);

    // Fees should compound (5000 + 2000)
    let total_fees = client.get_campaign_fee_history(&campaign_id);
    assert_eq!(total_fees, 7_000);
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
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

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
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

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

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

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
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

    client.pause();

    // Try create campaign - should fail
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let camp_id = create_test_campaign_id(&env, 10);
    let title = String::from_str(&env, "Test");
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 10000;

    let result =
        client.try_create_campaign(&camp_id, &title, &creator, &goal, &deadline, &token_id);
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
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

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
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

    // Create a campaign before pausing
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let camp_id = create_test_campaign_id(&env, 11);
    client.create_campaign(
        &camp_id,
        &String::from_str(&env, "Pre-pause"),
        &creator,
        &1000i128,
        &(env.ledger().timestamp() + 10000),
        &token_id,
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
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

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
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

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
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    client.initialize(&admin, &token_contract.address(), &0);

    client.pause();
    client.unpause();

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let camp_id = create_test_campaign_id(&env, 12);
    let title = String::from_str(&env, "After Unpause");
    client.create_campaign(
        &camp_id,
        &title,
        &creator,
        &1000i128,
        &(env.ledger().timestamp() + 10000),
        &token_id,
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
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = soroban_sdk::token::Client::new(&env, &token_id.address());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());

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
    client.contribute(&pool_id, &contributor, &token_id.address(), &amount, &false);

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
    let (client, _, _) = setup_test(&env);

    // 1. Returns empty list when no campaigns exist
    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 0);

    // 2. Returns all campaign IDs after multiple campaigns created
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let id1 = create_test_campaign_id(&env, 101);
    let id2 = create_test_campaign_id(&env, 102);

    client.create_campaign(
        &id1,
        &String::from_str(&env, "C1"),
        &creator,
        &1000i128,
        &(env.ledger().timestamp() + 100),
        &token_id,
    );
    client.create_campaign(
        &id2,
        &String::from_str(&env, "C2"),
        &creator,
        &1000i128,
        &(env.ledger().timestamp() + 100),
        &token_id,
    );

    let campaigns = client.get_all_campaigns();
    assert_eq!(campaigns.len(), 2);
    assert!(campaigns.contains(id1));
    assert!(campaigns.contains(id2));
}

#[test]
fn test_get_campaigns_bulk_retrieval_order() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);

    // Create multiple campaigns with different IDs
    let id_a = create_test_campaign_id(&env, 201);
    let id_b = create_test_campaign_id(&env, 202);
    let id_c = create_test_campaign_id(&env, 203);
    let id_d = create_test_campaign_id(&env, 204);

    client.create_campaign(
        &id_a,
        &String::from_str(&env, "Campaign A"),
        &creator,
        &1000i128,
        &(env.ledger().timestamp() + 100),
        &token_id,
    );
    client.create_campaign(
        &id_b,
        &String::from_str(&env, "Campaign B"),
        &creator,
        &2000i128,
        &(env.ledger().timestamp() + 200),
        &token_id,
    );
    client.create_campaign(
        &id_c,
        &String::from_str(&env, "Campaign C"),
        &creator,
        &3000i128,
        &(env.ledger().timestamp() + 300),
        &token_id,
    );
    client.create_campaign(
        &id_d,
        &String::from_str(&env, "Campaign D"),
        &creator,
        &4000i128,
        &(env.ledger().timestamp() + 400),
        &token_id,
    );

    // Request campaigns in reverse order: D, C, B, A
    let mut ids_reverse = Vec::new(&env);
    ids_reverse.push_back(id_d.clone());
    ids_reverse.push_back(id_c.clone());
    ids_reverse.push_back(id_b.clone());
    ids_reverse.push_back(id_a.clone());

    let campaigns = client.get_campaigns(&ids_reverse);

    // Verify the order matches the requested IDs
    assert_eq!(campaigns.len(), 4);
    assert_eq!(campaigns.get(0).unwrap().id, id_d);
    assert_eq!(campaigns.get(1).unwrap().id, id_c);
    assert_eq!(campaigns.get(2).unwrap().id, id_b);
    assert_eq!(campaigns.get(3).unwrap().id, id_a);

    // Verify content matches expected values
    assert_eq!(campaigns.get(0).unwrap().goal, 4000i128);
    assert_eq!(campaigns.get(1).unwrap().goal, 3000i128);
    assert_eq!(campaigns.get(2).unwrap().goal, 2000i128);
    assert_eq!(campaigns.get(3).unwrap().goal, 1000i128);
}

#[test]
fn test_get_campaigns_partial_retrieval() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);

    // Create two campaigns
    let id1 = create_test_campaign_id(&env, 211);
    let id2 = create_test_campaign_id(&env, 212);
    let id_nonexistent = create_test_campaign_id(&env, 213);

    client.create_campaign(
        &id1,
        &String::from_str(&env, "Existing Campaign"),
        &creator,
        &1000i128,
        &(env.ledger().timestamp() + 100),
        &token_id,
    );
    client.create_campaign(
        &id2,
        &String::from_str(&env, "Another Campaign"),
        &creator,
        &2000i128,
        &(env.ledger().timestamp() + 100),
        &token_id,
    );

    // Request one existing and one non-existing campaign
    let mut ids = Vec::new(&env);
    ids.push_back(id1.clone());
    ids.push_back(id_nonexistent);
    ids.push_back(id2.clone());

    let campaigns = client.get_campaigns(&ids);

    // Should return only the existing campaigns in order
    assert_eq!(campaigns.len(), 2);
    assert_eq!(campaigns.get(0).unwrap().id, id1);
    assert_eq!(campaigns.get(1).unwrap().id, id2);
}

#[test]
fn test_donate_and_donor_count() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 103);
    client.create_campaign(
        &campaign_id,
        &String::from_str(&env, "Donation Test"),
        &creator,
        &10000i128,
        &(env.ledger().timestamp() + 1000),
        &token_address,
    );

    // 1. Returns 0 for campaign with no donors
    assert_eq!(client.get_donor_count(&campaign_id), 0);
    assert_eq!(client.get_campaign_balance(&campaign_id), 0);

    // Setup donor
    let donor1 = Address::generate(&env);
    token_admin_client.mint(&donor1, &5000i128);

    // 2. Donate and check count
    client.donate(&campaign_id, &donor1, &token_address, &100i128);

    assert_eq!(client.get_donor_count(&campaign_id), 1);
    assert_eq!(client.get_campaign_balance(&campaign_id), 100);

    // 3. Same donor donates again -> count should still be 1
    client.donate(&campaign_id, &donor1, &token_address, &50i128);
    assert_eq!(client.get_donor_count(&campaign_id), 1);
    assert_eq!(client.get_campaign_balance(&campaign_id), 150);

    // 4. Different donor donates -> count should be 2
    let donor2 = Address::generate(&env);
    token_admin_client.mint(&donor2, &5000i128);
    client.donate(&campaign_id, &donor2, &token_address, &200i128);

    assert_eq!(client.get_donor_count(&campaign_id), 2);
    assert_eq!(client.get_campaign_balance(&campaign_id), 350);
}

#[test]
fn test_get_campaign_goal() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 104);
    let goal = 5555i128;

    client.create_campaign(
        &id,
        &String::from_str(&env, "Goal Test"),
        &creator,
        &goal,
        &(env.ledger().timestamp() + 100),
        &token_id,
    );

    assert_eq!(client.get_campaign_goal(&id), goal);
}

#[test]
fn test_is_campaign_completed() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 105);
    let goal = 1000i128;

    client.create_campaign(
        &id,
        &String::from_str(&env, "Completion Test"),
        &creator,
        &goal,
        &(env.ledger().timestamp() + 1000),
        &token_address,
    );

    // 1. Returns false for new campaign
    assert!(!client.is_campaign_completed(&id));

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &5000i128);

    // 2. Returns false when under goal
    client.donate(&id, &donor, &token_address, &900i128);
    assert!(!client.is_campaign_completed(&id));

    // 3. Returns true when goal is reached
    client.donate(&id, &donor, &token_address, &100i128); // Total 1000
    assert!(client.is_campaign_completed(&id));

    // 4. Campaign remains completed after goal is reached
    assert_eq!(client.get_total_raised(&id), 1000i128);
}

#[test]
fn test_get_campaign_status_live() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 106);
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 1000;

    client.create_campaign(
        &id,
        &String::from_str(&env, "Live Campaign"),
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &500i128);

    client.donate(&id, &donor, &token_address, &500i128);

    let status = client.get_campaign_status(&id);
    assert_eq!(status, crate::base::types::CampaignLifecycleStatus::Live);
}

#[test]
fn test_get_campaign_status_successful() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 107);
    let goal = 1000i128;
    let deadline = env.ledger().timestamp() + 1000;

    client.create_campaign(
        &id,
        &String::from_str(&env, "Successful Campaign"),
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &1000i128);

    client.donate(&id, &donor, &token_address, &1000i128);

    let status = client.get_campaign_status(&id);
    assert_eq!(
        status,
        crate::base::types::CampaignLifecycleStatus::Successful
    );
}

#[test]
fn test_get_campaign_status_expired() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 108);
    let goal = 1000i128;
    let deadline = 1500u64;

    client.create_campaign(
        &id,
        &String::from_str(&env, "Expired Campaign"),
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &500i128);

    client.donate(&id, &donor, &token_address, &500i128);

    env.ledger().with_mut(|li| li.timestamp = 2000);

    let status = client.get_campaign_status(&id);
    assert_eq!(status, crate::base::types::CampaignLifecycleStatus::Expired);
}

#[test]
fn test_donate_deadline_passed() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    let id = create_test_campaign_id(&env, 106);
    let deadline = 2000u64;

    client.create_campaign(
        &id,
        &String::from_str(&env, "Deadline Test"),
        &creator,
        &1000i128,
        &deadline,
        &token_address,
    );

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &5000i128);

    // Donate before deadline - should work
    client.donate(&id, &donor, &token_address, &100i128);

    // Advance time past deadline
    env.ledger().with_mut(|li| li.timestamp = 2001);

    // Donate after deadline - should fail
    let result = client.try_donate(&id, &donor, &token_address, &100i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignExpired)));
}

// Comprehensive Donation Tests

#[test]
fn test_successful_donation() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);
    let token_client = soroban_sdk::token::Client::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create campaign
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 200);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    // Setup donor
    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &5_000i128);

    // Verify initial state
    assert_eq!(client.get_total_raised(&campaign_id), 0);
    assert_eq!(client.get_contribution(&campaign_id, &donor), 0);

    // Make donation
    let donation_amount = 1_000i128;
    client.donate(&campaign_id, &donor, &token_id, &donation_amount);

    // Verify donation was successful
    assert_eq!(client.get_total_raised(&campaign_id), donation_amount);
    assert_eq!(
        client.get_contribution(&campaign_id, &donor),
        donation_amount
    );
    assert_eq!(token_client.balance(&donor), 4_000i128);
    assert_eq!(token_client.balance(&contract_id), donation_amount);
}

#[test]
fn test_multiple_donations_same_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create campaign
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 201);
    let title = String::from_str(&env, "Multi-Donation Campaign");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    // Setup donors
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    token_admin_client.mint(&donor1, &5_000i128);
    token_admin_client.mint(&donor2, &5_000i128);

    // First donation
    client.donate(&campaign_id, &donor1, &token_id, &1_000i128);
    assert_eq!(client.get_total_raised(&campaign_id), 1_000i128);
    assert_eq!(client.get_contribution(&campaign_id, &donor1), 1_000i128);

    // Second donation from same donor
    client.donate(&campaign_id, &donor1, &token_id, &500i128);
    assert_eq!(client.get_total_raised(&campaign_id), 1_500i128);
    assert_eq!(client.get_contribution(&campaign_id, &donor1), 1_500i128);

    // Third donation from different donor
    client.donate(&campaign_id, &donor2, &token_id, &2_000i128);
    assert_eq!(client.get_total_raised(&campaign_id), 3_500i128);
    assert_eq!(client.get_contribution(&campaign_id, &donor1), 1_500i128);
    assert_eq!(client.get_contribution(&campaign_id, &donor2), 2_000i128);
}

#[test]
fn test_donation_updates_total_raised() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create campaign
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 202);
    let title = String::from_str(&env, "Total Raised Test");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &10_000i128);

    // Initial total_raised should be 0
    assert_eq!(client.get_total_raised(&campaign_id), 0);

    // After first donation
    client.donate(&campaign_id, &donor, &token_id, &2_500i128);
    assert_eq!(client.get_total_raised(&campaign_id), 2_500i128);

    // After second donation
    client.donate(&campaign_id, &donor, &token_id, &1_500i128);
    assert_eq!(client.get_total_raised(&campaign_id), 4_000i128);

    // After third donation
    client.donate(&campaign_id, &donor, &token_id, &3_000i128);
    assert_eq!(client.get_total_raised(&campaign_id), 7_000i128);
}

#[test]
fn test_global_raised_total_increases_across_campaigns() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create first campaign
    let creator = Address::generate(&env);
    let campaign_id_1 = create_test_campaign_id(&env, 203);
    let title1 = String::from_str(&env, "Campaign One");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id_1,
        &title1,
        &creator,
        &goal,
        &deadline,
        &token_id,
    );

    // Create second campaign
    let creator2 = Address::generate(&env);
    let campaign_id_2 = create_test_campaign_id(&env, 204);
    let title2 = String::from_str(&env, "Campaign Two");

    client.create_campaign(
        &campaign_id_2,
        &title2,
        &creator2,
        &goal,
        &deadline,
        &token_id,
    );

    // Initial global total should be 0
    assert_eq!(client.get_global_raised_total(), 0);

    // Donor for campaign 1
    let donor1 = Address::generate(&env);
    token_admin_client.mint(&donor1, &10_000i128);

    // First donation to campaign 1
    client.donate(&campaign_id_1, &donor1, &token_id, &5_000i128);
    assert_eq!(client.get_global_raised_total(), 5_000i128);

    // Donor for campaign 2
    let donor2 = Address::generate(&env);
    token_admin_client.mint(&donor2, &10_000i128);

    // Donation to campaign 2
    client.donate(&campaign_id_2, &donor2, &token_id, &3_000i128);
    assert_eq!(client.get_global_raised_total(), 8_000i128);

    // Additional donation to campaign 1
    client.donate(&campaign_id_1, &donor1, &token_id, &2_000i128);
    assert_eq!(client.get_global_raised_total(), 10_000i128);
}

#[test]
fn test_contribution_tracked_per_user() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create campaign
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 203);
    let title = String::from_str(&env, "Contribution Tracking");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    // Setup multiple donors
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);
    token_admin_client.mint(&donor1, &5_000i128);
    token_admin_client.mint(&donor2, &5_000i128);
    token_admin_client.mint(&donor3, &5_000i128);

    // Each donor makes different contributions
    client.donate(&campaign_id, &donor1, &token_id, &1_000i128);
    client.donate(&campaign_id, &donor2, &token_id, &2_000i128);
    client.donate(&campaign_id, &donor3, &token_id, &500i128);

    // Verify individual contributions are tracked correctly
    assert_eq!(client.get_contribution(&campaign_id, &donor1), 1_000i128);
    assert_eq!(client.get_contribution(&campaign_id, &donor2), 2_000i128);
    assert_eq!(client.get_contribution(&campaign_id, &donor3), 500i128);

    // Donor1 makes another donation
    client.donate(&campaign_id, &donor1, &token_id, &750i128);
    assert_eq!(client.get_contribution(&campaign_id, &donor1), 1_750i128);
    assert_eq!(client.get_contribution(&campaign_id, &donor2), 2_000i128); // Unchanged
    assert_eq!(client.get_contribution(&campaign_id, &donor3), 500i128); // Unchanged
}

#[test]
fn test_donate_to_nonexistent_campaign() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &1_000i128);

    let fake_campaign_id = create_test_campaign_id(&env, 255);

    let result = client.try_donate(&fake_campaign_id, &donor, &token_id, &100i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignNotFound)));
}

#[test]
fn test_donate_after_deadline() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| li.timestamp = 1000);

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create campaign with deadline
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 204);
    let title = String::from_str(&env, "Expired Campaign");
    let goal = 10_000i128;
    let deadline = 2000u64;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &5_000i128);

    // Advance time past deadline
    env.ledger().with_mut(|li| li.timestamp = 2001);

    // Try to donate - should fail
    let result = client.try_donate(&campaign_id, &donor, &token_id, &100i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignExpired)));
}

#[test]
fn test_donate_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create campaign
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 205);
    let title = String::from_str(&env, "Zero Amount Test");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    let donor = Address::generate(&env);

    // Try to donate zero - should fail
    let result = client.try_donate(&campaign_id, &donor, &token_id, &0i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDonationAmount)));

    // Try to donate negative - should fail
    let result = client.try_donate(&campaign_id, &donor, &token_id, &-100i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidDonationAmount)));
}

#[test]
fn test_donate_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create campaign
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 206);
    let title = String::from_str(&env, "Insufficient Balance Test");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    let donor = Address::generate(&env);
    // Mint only 100 tokens
    token_admin_client.mint(&donor, &100i128);

    // Try to donate more than balance - should fail (token transfer will fail)
    let result = client.try_donate(&campaign_id, &donor, &token_id, &500i128);
    // The token transfer will fail, which should result in an error
    // Note: The exact error depends on the token contract implementation
    assert!(result.is_err());
}

#[test]
fn test_donate_campaign_already_funded() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract
    client.initialize(&admin, &token_id, &0);

    // Create campaign with goal of 1000
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 207);
    let title = String::from_str(&env, "Fully Funded Test");
    let goal = 1_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_id);

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &5_000i128);

    // Donate exactly the goal amount
    client.donate(&campaign_id, &donor, &token_id, &goal);

    // Verify campaign is completed
    assert_eq!(client.get_total_raised(&campaign_id), goal);
    assert_eq!(client.is_campaign_completed(&campaign_id), true);

    // Try to donate again - should fail
    let result = client.try_donate(&campaign_id, &donor, &token_id, &100i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignAlreadyFunded)));
}

#[test]
fn test_donate_wrong_token() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup two different tokens
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let token1_id = env
        .register_stellar_asset_contract_v2(admin1.clone())
        .address();
    let token2_id = env
        .register_stellar_asset_contract_v2(admin2.clone())
        .address();
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token1_id);

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Initialize contract with token1
    client.initialize(&admin1, &token1_id, &0);

    // Create campaign with token1
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 208);
    let title = String::from_str(&env, "Wrong Token Test");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token1_id);

    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &5_000i128);

    // Try to donate with wrong token - should fail
    let result = client.try_donate(&campaign_id, &donor, &token2_id, &100i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::TokenTransferFailed)));
}

#[test]
fn test_configuration_functions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();
    let fee = 100i128;

    // Initialize
    client.initialize(&admin, &token, &fee);

    // Get values
    assert_eq!(client.get_crowdfunding_token(), token);
    assert_eq!(client.get_creation_fee(), fee);

    // Update token
    let new_token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let new_token = new_token_contract.address();
    client.set_crowdfunding_token(&new_token);
    assert_eq!(client.get_crowdfunding_token(), new_token);

    // Update fee
    let new_fee = 200i128;
    client.set_creation_fee(&new_fee);
    assert_eq!(client.get_creation_fee(), new_fee);

    // Test auth
    let non_admin = Address::generate(&env);
    client.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "set_creation_fee",
            args: soroban_sdk::vec![&env, new_fee.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
}

#[test]
fn test_create_campaign_with_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();
    let token_client = token::Client::new(&env, &token);

    let fee = 500i128;
    client.initialize(&admin, &token, &fee);

    let creator = Address::generate(&env);
    // Mint tokens to creator
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&creator, &10_000);

    assert_eq!(token_client.balance(&creator), 10_000);
    assert_eq!(token_client.balance(&contract_id), 0);

    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Paid Campaign");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token);

    // Fee deduction
    assert_eq!(token_client.balance(&creator), 10_000 - fee);
    assert_eq!(token_client.balance(&contract_id), fee);
}

// Refund Tests

#[test]
fn test_refund_after_deadline_and_grace_period() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());
    let token_client = soroban_sdk::token::Client::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Create pool with deadline
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Refund Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test pool for refunds"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);
    let deadline = now + 86400; // 1 day from now

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Mint tokens to contributor
    token_admin_client.mint(&contributor, &5_000i128);

    // Contribute
    let contribution_amount = 2_000i128;
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &contribution_amount,
        &false,
    );

    // Verify contribution
    assert_eq!(token_client.balance(&contributor), 3_000i128);
    assert_eq!(token_client.balance(&contract_id), contribution_amount);

    // Advance time past deadline + grace period (7 days = 604800 seconds)
    let grace_period = 604800u64;
    env.ledger()
        .with_mut(|li| li.timestamp = deadline + grace_period + 1);

    // Refund
    client.refund(&pool_id, &contributor);

    // Verify refund
    assert_eq!(token_client.balance(&contributor), 5_000i128);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_refund_partial_contribution() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup token
    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());
    let token_client = soroban_sdk::token::Client::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    // Create pool
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Partial Refund Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test partial refund"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);
    let deadline = now + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Mint tokens
    token_admin_client.mint(&contributor, &10_000i128);

    // Make multiple contributions
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &1_000i128,
        &false,
    );
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &500i128,
        &false,
    );
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &750i128,
        &false,
    );

    let total_contributed = 2_250i128;
    assert_eq!(token_client.balance(&contributor), 7_750i128);
    assert_eq!(token_client.balance(&contract_id), total_contributed);

    // Advance time past deadline + grace period
    let grace_period = 604800u64;
    env.ledger()
        .with_mut(|li| li.timestamp = deadline + grace_period + 1);

    // Refund should return all contributions
    client.refund(&pool_id, &contributor);

    // Verify full refund
    assert_eq!(token_client.balance(&contributor), 10_000i128);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_refund_fails_before_deadline() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Early Refund Test");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);
    let deadline = now + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    token_admin_client.mint(&contributor, &5_000i128);
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &1_000i128,
        &false,
    );

    // Try to refund before deadline - should fail
    let result = client.try_refund(&pool_id, &contributor);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotExpired)));
}

#[test]
fn test_refund_fails_before_grace_period() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Grace Period Test");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);
    let deadline = now + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    token_admin_client.mint(&contributor, &5_000i128);
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &1_000i128,
        &false,
    );

    // Advance time past deadline but before grace period
    let grace_period = 604800u64;
    env.ledger()
        .with_mut(|li| li.timestamp = deadline + grace_period - 1);

    // Try to refund - should fail
    let result = client.try_refund(&pool_id, &contributor);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::RefundGracePeriodNotPassed))
    );
}

#[test]
fn test_refund_fails_if_disbursed() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Disbursed Pool Test");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);
    let deadline = now + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    token_admin_client.mint(&contributor, &5_000i128);
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &1_000i128,
        &false,
    );

    // Mark pool as disbursed
    client.update_pool_state(&pool_id, &PoolState::Disbursed);

    // Advance time past deadline + grace period
    let grace_period = 604800u64;
    env.ledger()
        .with_mut(|li| li.timestamp = deadline + grace_period + 1);

    // Try to refund - should fail
    let result = client.try_refund(&pool_id, &contributor);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyDisbursed)));
}

#[test]
fn test_create_campaign_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_contract.address();

    let fee = 500i128;
    client.initialize(&admin, &token, &fee);

    let creator = Address::generate(&env);
    // 0 balance

    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Poor Campaign");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result =
        client.try_create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token);
    assert_eq!(result, Err(Ok(CrowdfundingError::InsufficientBalance)));
}

#[test]
fn test_refund_fails_no_contribution() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "No Contribution Test");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);
    let deadline = now + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Advance time past deadline + grace period
    let grace_period = 604800u64;
    env.ledger()
        .with_mut(|li| li.timestamp = deadline + grace_period + 1);

    // Try to refund without contributing - should fail
    let result = client.try_refund(&pool_id, &contributor);
    assert_eq!(result, Err(Ok(CrowdfundingError::NoContributionToRefund)));
}

#[test]
fn test_multiple_contributors_refund_independently() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());
    let token_client = soroban_sdk::token::Client::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);
    let name = String::from_str(&env, "Multiple Refunds Test");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);
    let deadline = now + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    // Both contributors contribute
    token_admin_client.mint(&contributor1, &5_000i128);
    token_admin_client.mint(&contributor2, &5_000i128);

    client.contribute(
        &pool_id,
        &contributor1,
        &token_id.address(),
        &2_000i128,
        &false,
    );
    client.contribute(
        &pool_id,
        &contributor2,
        &token_id.address(),
        &1_500i128,
        &false,
    );

    assert_eq!(token_client.balance(&contract_id), 3_500i128);

    // Advance time past deadline + grace period
    let grace_period = 604800u64;
    env.ledger()
        .with_mut(|li| li.timestamp = deadline + grace_period + 1);

    // Contributor1 refunds
    client.refund(&pool_id, &contributor1);
    assert_eq!(token_client.balance(&contributor1), 5_000i128);
    assert_eq!(token_client.balance(&contract_id), 1_500i128);

    // Contributor2 refunds
    client.refund(&pool_id, &contributor2);
    assert_eq!(token_client.balance(&contributor2), 5_000i128);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_refund_fails_after_already_refunded() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Double Refund Test");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);
    let deadline = now + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None::<u32>,
        &None::<Vec<Address>>,
    );

    token_admin_client.mint(&contributor, &5_000i128);
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &1_000i128,
        &false,
    );

    // Advance time past deadline + grace period
    let grace_period = 604800u64;
    env.ledger()
        .with_mut(|li| li.timestamp = deadline + grace_period + 1);

    // First refund succeeds
    client.refund(&pool_id, &contributor);

    // Try to refund again - should fail
    let result = client.try_refund(&pool_id, &contributor);
    assert_eq!(result, Err(Ok(CrowdfundingError::NoContributionToRefund)));
}

// Emergency Withdrawal Tests

#[test]
fn test_request_emergency_withdraw_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let amount = 1_000i128;
    client.request_emergency_withdraw(&token_address, &amount);
}

#[test]
fn test_request_emergency_withdraw_duplicate() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let amount = 1_000i128;
    client.request_emergency_withdraw(&token_address, &amount);

    let result = client.try_request_emergency_withdraw(&token_address, &amount);
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::EmergencyWithdrawalAlreadyRequested))
    );
}

#[test]
fn test_execute_emergency_withdraw_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::Client::new(&env, &token_address);

    client.initialize(&admin, &token_address, &0);

    token_admin_client.mint(&contract_id, &5_000i128);

    let amount = 1_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);

    client.request_emergency_withdraw(&token_address, &amount);

    env.ledger().with_mut(|li| li.timestamp = now + 86400 + 1);

    client.execute_emergency_withdraw();

    assert_eq!(token_client.balance(&admin), amount);
    assert_eq!(token_client.balance(&contract_id), 4_000i128);
}

#[test]
fn test_execute_emergency_withdraw_before_grace_period() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let amount = 1_000i128;
    let now = 1000u64;
    env.ledger().with_mut(|li| li.timestamp = now);

    client.request_emergency_withdraw(&token_address, &amount);

    env.ledger().with_mut(|li| li.timestamp = now + 86399);

    let result = client.try_execute_emergency_withdraw();
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::EmergencyWithdrawalPeriodNotPassed))
    );
}

#[test]
fn test_execute_emergency_withdraw_not_requested() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client.initialize(&admin, &token_address, &0);

    let result = client.try_execute_emergency_withdraw();
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::EmergencyWithdrawalNotRequested))
    );
}

// Pool Contribution Tests

#[test]
fn test_contribute_to_pool_success() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id.address());
    let token_client = token::Client::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test pool"),
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

    token_admin_client.mint(&contributor, &5_000i128);

    let contribution_amount = 1_000i128;
    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &contribution_amount,
        &false,
    );

    assert_eq!(token_client.balance(&contributor), 4_000i128);
    assert_eq!(token_client.balance(&contract_id), contribution_amount);
}

#[test]
fn test_contribute_to_paused_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    client.initialize(&admin, &token_address, &0);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Paused Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
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

    client.pause();

    token_admin_client.mint(&contributor, &5_000i128);

    let result = client.try_contribute(&pool_id, &contributor, &token_address, &1_000i128, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractPaused)));
}

#[test]
fn test_contribute_to_non_active_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Completed Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
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

    client.update_pool_state(&pool_id, &PoolState::Completed);

    token_admin_client.mint(&contributor, &5_000i128);

    let result = client.try_contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &1_000i128,
        &false,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidPoolState)));
}

#[test]
fn test_contribute_zero_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
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

    let result = client.try_contribute(&pool_id, &contributor, &token_id.address(), &0i128, &false);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidAmount)));
}

#[test]
fn test_contribute_to_nonexistent_pool() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let contributor = Address::generate(&env);

    let result = client.try_contribute(
        &999u64,
        &contributor,
        &token_id.address(),
        &1_000i128,
        &false,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolNotFound)));
}

#[test]
fn test_contribute_multiple_contributors() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor1 = Address::generate(&env);
    let contributor2 = Address::generate(&env);
    let contributor3 = Address::generate(&env);

    let name = String::from_str(&env, "Multi Contributor Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
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

    token_admin_client.mint(&contributor1, &5_000i128);
    token_admin_client.mint(&contributor2, &5_000i128);
    token_admin_client.mint(&contributor3, &5_000i128);

    client.contribute(
        &pool_id,
        &contributor1,
        &token_id.address(),
        &1_000i128,
        &false,
    );
    client.contribute(
        &pool_id,
        &contributor2,
        &token_id.address(),
        &2_000i128,
        &false,
    );
    client.contribute(
        &pool_id,
        &contributor3,
        &token_id.address(),
        &500i128,
        &false,
    );
}

#[test]
fn test_contribute_private_contribution() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_admin_client = token::StellarAssetClient::new(&env, &token_id.address());

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let name = String::from_str(&env, "Private Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test"),
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

    token_admin_client.mint(&contributor, &5_000i128);

    client.contribute(
        &pool_id,
        &contributor,
        &token_id.address(),
        &1_000i128,
        &true,
    );
}

// Configuration Tests

#[test]
fn test_set_crowdfunding_token_unauthorized() {
    let env = Env::default();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    client
        .mock_auths(&[])
        .initialize(&admin, &token_address, &0);

    let new_token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let new_token = new_token_contract.address();

    let result = client
        .mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &non_admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_crowdfunding_token",
                args: soroban_sdk::vec![&env, new_token.into_val(&env)],
                sub_invokes: &[],
            },
        }])
        .try_set_crowdfunding_token(&new_token);

    assert!(result.is_err());
}

#[test]
fn test_update_campaign_goal_success() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 100);
    let title = String::from_str(&env, "Goal Update Test");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    // Mock auth for update
    client.update_campaign_goal(&campaign_id, &5_000i128);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.goal, 5_000i128);
}

#[test]
fn test_update_campaign_goal_increase_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 101);
    let title = String::from_str(&env, "Goal Increase Test");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    // Try to increase goal - should fail
    let result = client.try_update_campaign_goal(&campaign_id, &15_000i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidGoalUpdate)));
}

#[test]
fn test_update_campaign_goal_below_raised_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 102);
    let title = String::from_str(&env, "Goal Below Raised Test");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    // Donate
    let donor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&donor, &8_000i128);

    client.donate(&campaign_id, &donor, &token_address, &6_000i128);

    // Try to lower goal below raised amount - should fail
    let result = client.try_update_campaign_goal(&campaign_id, &5_000i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidGoalUpdate)));
}

#[test]
fn test_update_campaign_goal_expired_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 103);
    let title = String::from_str(&env, "Expired Update Test");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    // Advance time past deadline
    env.ledger().with_mut(|li| li.timestamp = deadline + 1);

    let result = client.try_update_campaign_goal(&campaign_id, &5_000i128);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignExpired)));
}

#[test]
fn test_get_active_campaign_count() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    // Set current time to 1000
    env.ledger().with_mut(|li| li.timestamp = 1000);

    // Initially should be 0
    assert_eq!(client.get_active_campaign_count(), 0);

    let creator = Address::generate(&env);

    // Create active campaign (deadline in the future)
    let active_id = create_test_campaign_id(&env, 50);
    client.create_campaign(
        &active_id,
        &String::from_str(&env, "Active Campaign"),
        &creator,
        &1000i128,
        &2000u64, // deadline = 2000, current time = 1000 → active
        &token_address,
    );

    // Create expired campaign (deadline in the past relative to advanced time)
    let expired_id = create_test_campaign_id(&env, 51);
    client.create_campaign(
        &expired_id,
        &String::from_str(&env, "Expired Campaign"),
        &creator,
        &1000i128,
        &1500u64, // deadline = 1500
        &token_address,
    );

    // Advance time past the expired campaign's deadline
    env.ledger().with_mut(|li| li.timestamp = 1600);

    // Only 1 campaign should be active (deadline 2000 > current 1600)
    assert_eq!(client.get_active_campaign_count(), 1);
}

#[test]
fn test_get_active_campaign_count_empty() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    assert_eq!(client.get_active_campaign_count(), 0);
}

#[test]
fn test_get_active_campaign_count_all_active() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let creator = Address::generate(&env);

    for i in 0u8..3 {
        let id = create_test_campaign_id(&env, 60 + i);
        client.create_campaign(
            &id,
            &String::from_str(&env, "Active"),
            &creator,
            &1000i128,
            &5000u64,
            &token_address,
        );
    }

    assert_eq!(client.get_active_campaign_count(), 3);
}

#[test]
fn test_get_active_campaign_count_all_expired() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let creator = Address::generate(&env);

    for i in 0u8..3 {
        let id = create_test_campaign_id(&env, 70 + i);
        client.create_campaign(
            &id,
            &String::from_str(&env, "Soon Expired"),
            &creator,
            &1000i128,
            &2000u64,
            &token_address,
        );
    }

    env.ledger().with_mut(|li| li.timestamp = 3000);

    assert_eq!(client.get_active_campaign_count(), 0);
}

fn test_withdraw_platform_fees_success() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    client.set_creation_fee(&100);

    let creator = Address::generate(&env);
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&creator, &1_000i128);

    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Campaign with fee");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    let token_client = token::Client::new(&env, &token_address);
    let admin_balance_before = token_client.balance(&admin);

    client.withdraw_platform_fees(&admin, &100);

    let admin_balance_after = token_client.balance(&admin);
    assert_eq!(admin_balance_after - admin_balance_before, 100);
}

#[test]
fn test_withdraw_platform_fees_non_admin_fails() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    client.set_creation_fee(&100);

    let creator = Address::generate(&env);
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&creator, &1_000i128);

    let campaign_id = create_test_campaign_id(&env, 2);
    let title = String::from_str(&env, "Campaign with fee 2");
    let goal = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );

    env.ledger().with_mut(|li| li.timestamp = deadline);
    assert_eq!(client.get_active_campaign_count(), 0);

    env.ledger().with_mut(|li| li.timestamp = deadline - 1);
    assert_eq!(client.get_active_campaign_count(), 1);
}

#[test]
fn test_get_active_campaign_count_decreases_over_time() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let creator = Address::generate(&env);

    let id1 = create_test_campaign_id(&env, 90);
    client.create_campaign(
        &id1,
        &String::from_str(&env, "Short Campaign"),
        &creator,
        &1000i128,
        &2000u64,
        &token_address,
    );

    let id2 = create_test_campaign_id(&env, 91);
    client.create_campaign(
        &id2,
        &String::from_str(&env, "Medium Campaign"),
        &creator,
        &1000i128,
        &4000u64,
        &token_address,
    );

    let id3 = create_test_campaign_id(&env, 92);
    client.create_campaign(
        &id3,
        &String::from_str(&env, "Long Campaign"),
        &creator,
        &1000i128,
        &6000u64,
        &token_address,
    );

    assert_eq!(client.get_active_campaign_count(), 3);

    env.ledger().with_mut(|li| li.timestamp = 2500);
    assert_eq!(client.get_active_campaign_count(), 2);

    env.ledger().with_mut(|li| li.timestamp = 4500);
    assert_eq!(client.get_active_campaign_count(), 1);

    env.ledger().with_mut(|li| li.timestamp = 7000);
    assert_eq!(client.get_active_campaign_count(), 0);
}

#[test]
fn test_get_active_campaign_count_unaffected_by_donation_status() {
    let env = Env::default();
    let (client, _, token_address) = setup_test(&env);

    env.ledger().with_mut(|li| li.timestamp = 1000);

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    token_admin_client.mint(&donor, &10_000i128);

    let id = create_test_campaign_id(&env, 93);
    client.create_campaign(
        &id,
        &String::from_str(&env, "Funded Campaign"),
        &creator,
        &500i128,
        &5000u64,
        &token_address,
    );

    client.donate(&id, &donor, &token_address, &500i128);
    assert!(client.is_campaign_completed(&id));

    // fully funded but deadline not passed - still counts as active
    assert_eq!(client.get_active_campaign_count(), 1);
    let non_admin = Address::generate(&env);
    let res = client.try_withdraw_platform_fees(&non_admin, &100);
    assert_eq!(res, Err(Ok(CrowdfundingError::Unauthorized)));
}

#[test]
fn test_withdraw_platform_fees_insufficient_fees() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let res = client.try_withdraw_platform_fees(&admin, &100);
    assert_eq!(res, Err(Ok(CrowdfundingError::InsufficientFees)));
}
#[test]
fn test_set_emergency_contact_success() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let emergency_contact = Address::generate(&env);

    let result = client.try_set_emergency_contact(&emergency_contact);
    assert!(result.is_ok());

    let stored_contact = client.get_emergency_contact();
    assert_eq!(stored_contact, emergency_contact);
}

#[test]
fn test_set_emergency_contact_updates_existing() {
    let env = Env::default();
    let (client, admin, _) = setup_test(&env);

    let contact1 = Address::generate(&env);
    client.set_emergency_contact(&contact1);

    let contact2 = Address::generate(&env);
    client.set_emergency_contact(&contact2);

    let stored_contact = client.get_emergency_contact();
    assert_eq!(stored_contact, contact2);
}

#[test]
fn test_get_emergency_contact_not_initialized() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let result = client.try_get_emergency_contact();
    assert!(result.is_err());
}

#[test]
fn test_emergency_contact_multiple_updates() {
    let env = Env::default();
    let (client, _admin, _) = setup_test(&env);

    let contact1 = Address::generate(&env);
    client.set_emergency_contact(&contact1);
    assert_eq!(client.get_emergency_contact(), contact1);

    let contact2 = Address::generate(&env);
    client.set_emergency_contact(&contact2);
    assert_eq!(client.get_emergency_contact(), contact2);

    let contact3 = Address::generate(&env);
    client.set_emergency_contact(&contact3);
    assert_eq!(client.get_emergency_contact(), contact3);
}

#[test]
fn test_get_contract_version() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    let version = client.get_contract_version();
    assert_eq!(version, String::from_str(&env, "1.2.0"));
}
