#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, BytesN, Env, String,
};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::PoolMetadata,
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

    client.initialize(&admin, &token_address, &0);

    (client, admin, token_address)
}

// ============================================================================
// Module-Specific Pause Tests
// ============================================================================

#[test]
fn test_pause_pools_only() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Initially nothing is paused
    assert!(!client.is_paused());
    assert!(!client.is_pools_paused());
    assert!(!client.is_campaigns_paused());

    // Pause only pools
    client.pause_pools();

    // Verify only pools are paused
    assert!(!client.is_paused());
    assert!(client.is_pools_paused());
    assert!(!client.is_campaigns_paused());

    // Campaigns should still work
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Campaign While Pools Paused");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_address);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.title, title);

    // Pool operations should fail
    let pool_name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target = 10_000i128;
    let pool_deadline = env.ledger().timestamp() + 86400;

    let result = client.try_save_pool(
        &pool_name,
        &metadata,
        &creator,
        &target,
        &pool_deadline,
        &None,
        &None,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolsPaused)));
}

#[test]
fn test_pause_campaigns_only() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Initially nothing is paused
    assert!(!client.is_paused());
    assert!(!client.is_pools_paused());
    assert!(!client.is_campaigns_paused());

    // Pause only campaigns
    client.pause_campaigns();

    // Verify only campaigns are paused
    assert!(!client.is_paused());
    assert!(!client.is_pools_paused());
    assert!(client.is_campaigns_paused());

    // Pools should still work
    let creator = Address::generate(&env);
    let pool_name = String::from_str(&env, "Pool While Campaigns Paused");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &pool_name,
        &metadata,
        &creator,
        &target,
        &deadline,
        &None,
        &None,
    );

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.name, pool_name);

    // Campaign operations should fail
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let campaign_deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &campaign_deadline,
        &token_address,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignsPaused)));
}

#[test]
fn test_pause_both_modules() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Pause both modules
    client.pause_pools();
    client.pause_campaigns();

    // Verify both are paused
    assert!(client.is_pools_paused());
    assert!(client.is_campaigns_paused());

    // Both operations should fail
    let creator = Address::generate(&env);

    // Try campaign
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignsPaused)));

    // Try pool
    let pool_name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target = 10_000i128;

    let result = client.try_save_pool(
        &pool_name,
        &metadata,
        &creator,
        &target,
        &deadline,
        &None,
        &None,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolsPaused)));
}

#[test]
fn test_unpause_pools() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Pause pools
    client.pause_pools();
    assert!(client.is_pools_paused());

    // Unpause pools
    client.unpause_pools();
    assert!(!client.is_pools_paused());

    // Pool operations should work again
    let creator = Address::generate(&env);
    let pool_name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &pool_name,
        &metadata,
        &creator,
        &target,
        &deadline,
        &None,
        &None,
    );

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.name, pool_name);
}

#[test]
fn test_unpause_campaigns() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Pause campaigns
    client.pause_campaigns();
    assert!(client.is_campaigns_paused());

    // Unpause campaigns
    client.unpause_campaigns();
    assert!(!client.is_campaigns_paused());

    // Campaign operations should work again
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_address);

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.title, title);
}

#[test]
fn test_double_pause_pools_fails() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    // First pause should succeed
    client.pause_pools();

    // Second pause should fail
    let result = client.try_pause_pools();
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolsAlreadyPaused)));
}

#[test]
fn test_double_pause_campaigns_fails() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    // First pause should succeed
    client.pause_campaigns();

    // Second pause should fail
    let result = client.try_pause_campaigns();
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignsAlreadyPaused)));
}

#[test]
fn test_double_unpause_pools_fails() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    // Pause first
    client.pause_pools();
    client.unpause_pools();

    // Second unpause should fail
    let result = client.try_unpause_pools();
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolsAlreadyUnpaused)));
}

#[test]
fn test_double_unpause_campaigns_fails() {
    let env = Env::default();
    let (client, _, _) = setup_test(&env);

    // Pause first
    client.pause_campaigns();
    client.unpause_campaigns();

    // Second unpause should fail
    let result = client.try_unpause_campaigns();
    assert_eq!(
        result,
        Err(Ok(CrowdfundingError::CampaignsAlreadyUnpaused))
    );
}

#[test]
fn test_donate_blocked_when_campaigns_paused() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Create a campaign first
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_address);

    // Setup donor balance
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&donor, &1_000_000i128);

    // Pause campaigns
    client.pause_campaigns();

    // Try to donate - should fail
    let donation_amount = 500_000i128;
    let result = client.try_donate(&campaign_id, &donor, &token_address, &donation_amount);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignsPaused)));
}

#[test]
fn test_contribute_blocked_when_pools_paused() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Create a pool first
    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);
    let pool_name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &pool_name,
        &metadata,
        &creator,
        &target,
        &deadline,
        &None,
        &None,
    );

    // Setup contributor balance
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);
    token_admin_client.mint(&contributor, &100_000i128);

    // Pause pools
    client.pause_pools();

    // Try to contribute - should fail
    let contribution_amount = 5_000i128;
    let result = client.try_contribute(
        &pool_id,
        &contributor,
        &token_address,
        &contribution_amount,
        &false,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolsPaused)));
}

#[test]
fn test_global_pause_overrides_module_pause() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Unpause campaigns module
    assert!(!client.is_campaigns_paused());

    // Pause entire contract
    client.pause();

    // Even though campaigns module is not paused, global pause should block
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let result = client.try_create_campaign(
        &campaign_id,
        &title,
        &creator,
        &goal,
        &deadline,
        &token_address,
    );
    assert_eq!(result, Err(Ok(CrowdfundingError::ContractPaused)));
}

#[test]
fn test_module_pause_independent_of_global_pause() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Pause campaigns module
    client.pause_campaigns();

    // Global pause state should be independent
    assert!(!client.is_paused());
    assert!(client.is_campaigns_paused());

    // Pause global
    client.pause();

    // Both should be paused now
    assert!(client.is_paused());
    assert!(client.is_campaigns_paused());

    // Unpause global
    client.unpause();

    // Campaigns should still be paused
    assert!(!client.is_paused());
    assert!(client.is_campaigns_paused());
}

#[test]
fn test_campaign_operations_blocked_when_paused() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Create a campaign first
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_address);

    // Pause campaigns
    client.pause_campaigns();

    // Test various campaign operations are blocked
    let new_goal = 500_000i128;
    let result = client.try_update_campaign_goal(&campaign_id, &new_goal);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignsPaused)));

    let result = client.try_cancel_campaign(&campaign_id);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignsPaused)));

    let new_deadline = env.ledger().timestamp() + 172800;
    let result = client.try_extend_campaign_deadline(&campaign_id, &new_deadline);
    assert_eq!(result, Err(Ok(CrowdfundingError::CampaignsPaused)));
}

#[test]
fn test_pool_operations_blocked_when_paused() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Create a pool first
    let creator = Address::generate(&env);
    let pool_name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &pool_name,
        &metadata,
        &creator,
        &target,
        &deadline,
        &None,
        &None,
    );

    // Pause pools
    client.pause_pools();

    // Test various pool operations are blocked
    use crate::base::types::PoolState;
    let result = client.try_update_pool_state(&pool_id, &PoolState::Paused);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolsPaused)));
}

#[test]
fn test_read_operations_work_when_modules_paused() {
    let env = Env::default();
    let (client, admin, token_address) = setup_test(&env);

    // Create campaign and pool
    let creator = Address::generate(&env);
    let campaign_id = create_test_campaign_id(&env, 1);
    let title = String::from_str(&env, "Test Campaign");
    let goal = 1_000_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    client.create_campaign(&campaign_id, &title, &creator, &goal, &deadline, &token_address);

    let pool_name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test Description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target = 10_000i128;

    let pool_id = client.save_pool(
        &pool_name,
        &metadata,
        &creator,
        &target,
        &deadline,
        &None,
        &None,
    );

    // Pause both modules
    client.pause_campaigns();
    client.pause_pools();

    // Read operations should still work
    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.title, title);

    let pool = client.get_pool(&pool_id).unwrap();
    assert_eq!(pool.name, pool_name);

    let all_campaigns = client.get_all_campaigns();
    assert_eq!(all_campaigns.len(), 1);
}
