#![cfg(test)]

//! Coverage gap tests — exercises branches identified as uncovered.
//!
//! Each test targets a specific untested code path in crowdfunding.rs or
//! base/types.rs. Tests are grouped by the function they cover.

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{
            PoolConfig, PoolMetadata, PoolState, MAX_DESCRIPTION_LENGTH, MAX_HASH_LENGTH,
            MAX_URL_LENGTH,
        },
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{
    testutils::Address as _, testutils::Ledger as _, token::StellarAssetClient, vec, Address,
    BytesN, Env, String, Vec,
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &id);
    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    client.initialize(&admin, &token, &0);
    (client, admin, token)
}

fn campaign_id(env: &Env, n: u8) -> BytesN<32> {
    BytesN::from_array(env, &[n; 32])
}

fn make_pool_metadata(env: &Env) -> PoolMetadata {
    PoolMetadata {
        description: String::from_str(env, "desc"),
        external_url: String::from_str(env, "https://example.com"),
        image_hash: String::from_str(env, "abc123"),
    }
}

// ---------------------------------------------------------------------------
// create_pool — InvalidToken branch
// ---------------------------------------------------------------------------

/// create_pool with a token that doesn't match the platform token → InvalidToken.
#[test]
fn test_create_pool_wrong_token_fails() {
    let env = Env::default();
    let (client, creator, _platform_token) = setup(&env);

    let other_admin = Address::generate(&env);
    let wrong_token = env
        .register_stellar_asset_contract_v2(other_admin)
        .address();

    let config = PoolConfig {
        name: String::from_str(&env, "Bad Token Pool"),
        description: String::from_str(&env, "desc"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: wrong_token,
    };

    assert_eq!(
        client.try_create_pool(&creator, &config),
        Err(Ok(CrowdfundingError::InvalidToken))
    );
}

// ---------------------------------------------------------------------------
// save_pool — InvalidMetadata branch
// ---------------------------------------------------------------------------

/// save_pool with a description that exceeds MAX_DESCRIPTION_LENGTH → InvalidMetadata.
#[test]
fn test_save_pool_description_too_long_fails() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let long_desc = "x".repeat((MAX_DESCRIPTION_LENGTH + 1) as usize);
    let metadata = PoolMetadata {
        description: String::from_str(&env, &long_desc),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };

    assert_eq!(
        client.try_save_pool(
            &String::from_str(&env, "Pool"),
            &metadata,
            &creator,
            &1_000,
            &(env.ledger().timestamp() + 86_400),
            &None,
            &None,
        ),
        Err(Ok(CrowdfundingError::InvalidMetadata))
    );
}

/// save_pool with an external_url that exceeds MAX_URL_LENGTH → InvalidMetadata.
#[test]
fn test_save_pool_url_too_long_fails() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let long_url = "u".repeat((MAX_URL_LENGTH + 1) as usize);
    let metadata = PoolMetadata {
        description: String::from_str(&env, "ok"),
        external_url: String::from_str(&env, &long_url),
        image_hash: String::from_str(&env, ""),
    };

    assert_eq!(
        client.try_save_pool(
            &String::from_str(&env, "Pool"),
            &metadata,
            &creator,
            &1_000,
            &(env.ledger().timestamp() + 86_400),
            &None,
            &None,
        ),
        Err(Ok(CrowdfundingError::InvalidMetadata))
    );
}

/// save_pool with an image_hash that exceeds MAX_HASH_LENGTH → InvalidMetadata.
#[test]
fn test_save_pool_hash_too_long_fails() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let long_hash = "h".repeat((MAX_HASH_LENGTH + 1) as usize);
    let metadata = PoolMetadata {
        description: String::from_str(&env, "ok"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, &long_hash),
    };

    assert_eq!(
        client.try_save_pool(
            &String::from_str(&env, "Pool"),
            &metadata,
            &creator,
            &1_000,
            &(env.ledger().timestamp() + 86_400),
            &None,
            &None,
        ),
        Err(Ok(CrowdfundingError::InvalidMetadata))
    );
}

// ---------------------------------------------------------------------------
// save_pool — InvalidMultiSigConfig branches
// ---------------------------------------------------------------------------

/// save_pool with req_sigs == 0 → InvalidMultiSigConfig.
#[test]
fn test_save_pool_multisig_zero_required_sigs_fails() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let signer = Address::generate(&env);
    let signers: Vec<Address> = vec![&env, signer];

    assert_eq!(
        client.try_save_pool(
            &String::from_str(&env, "Pool"),
            &make_pool_metadata(&env),
            &creator,
            &1_000,
            &(env.ledger().timestamp() + 86_400),
            &Some(0u32), // req_sigs = 0 → invalid
            &Some(signers),
        ),
        Err(Ok(CrowdfundingError::InvalidMultiSigConfig))
    );
}

/// save_pool with req_sigs > signer count → InvalidMultiSigConfig.
#[test]
fn test_save_pool_multisig_req_exceeds_signers_fails() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let signer = Address::generate(&env);
    let signers: Vec<Address> = vec![&env, signer];

    assert_eq!(
        client.try_save_pool(
            &String::from_str(&env, "Pool"),
            &make_pool_metadata(&env),
            &creator,
            &1_000,
            &(env.ledger().timestamp() + 86_400),
            &Some(5u32), // 5 required but only 1 signer
            &Some(signers),
        ),
        Err(Ok(CrowdfundingError::InvalidMultiSigConfig))
    );
}

/// save_pool with only one of (required_signatures, signers) provided → InvalidMultiSigConfig.
#[test]
fn test_save_pool_multisig_partial_config_fails() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    // signers provided but no required_signatures
    let signer = Address::generate(&env);
    let signers: Vec<Address> = vec![&env, signer];

    assert_eq!(
        client.try_save_pool(
            &String::from_str(&env, "Pool"),
            &make_pool_metadata(&env),
            &creator,
            &1_000,
            &(env.ledger().timestamp() + 86_400),
            &None::<u32>,
            &Some(signers),
        ),
        Err(Ok(CrowdfundingError::InvalidMultiSigConfig))
    );
}

/// save_pool with valid multisig config → succeeds and stores config.
#[test]
fn test_save_pool_valid_multisig_succeeds() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);
    let signers: Vec<Address> = vec![&env, s1, s2];

    let pool_id = client.save_pool(
        &String::from_str(&env, "MultiSig Pool"),
        &make_pool_metadata(&env),
        &creator,
        &5_000,
        &(env.ledger().timestamp() + 86_400),
        &Some(2u32),
        &Some(signers),
    );
    assert!(pool_id > 0);
}

// ---------------------------------------------------------------------------
// update_pool_state — Completed and Cancelled terminal state branches
// ---------------------------------------------------------------------------

/// Transitioning from Completed state → InvalidPoolState.
#[test]
fn test_update_pool_state_from_completed_fails() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let pool_id = client.save_pool(
        &String::from_str(&env, "Pool"),
        &make_pool_metadata(&env),
        &creator,
        &1_000,
        &(env.ledger().timestamp() + 86_400),
        &None,
        &None,
    );

    // Drive to Completed
    client.update_pool_state(&pool_id, &PoolState::Completed);

    // Any further transition must fail
    assert_eq!(
        client.try_update_pool_state(&pool_id, &PoolState::Active),
        Err(Ok(CrowdfundingError::InvalidPoolState))
    );
}

/// Transitioning from Cancelled state → InvalidPoolState.
#[test]
fn test_update_pool_state_from_cancelled_fails() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let pool_id = client.save_pool(
        &String::from_str(&env, "Pool"),
        &make_pool_metadata(&env),
        &creator,
        &1_000,
        &(env.ledger().timestamp() + 86_400),
        &None,
        &None,
    );

    client.update_pool_state(&pool_id, &PoolState::Cancelled);

    assert_eq!(
        client.try_update_pool_state(&pool_id, &PoolState::Active),
        Err(Ok(CrowdfundingError::InvalidPoolState))
    );
}

/// Paused → Active transition succeeds.
#[test]
fn test_update_pool_state_paused_to_active_succeeds() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let pool_id = client.save_pool(
        &String::from_str(&env, "Pool"),
        &make_pool_metadata(&env),
        &creator,
        &1_000,
        &(env.ledger().timestamp() + 86_400),
        &None,
        &None,
    );

    client.update_pool_state(&pool_id, &PoolState::Paused);
    // Resume — should succeed
    client.update_pool_state(&pool_id, &PoolState::Active);
}

/// Active → Completed transition succeeds.
#[test]
fn test_update_pool_state_active_to_completed_succeeds() {
    let env = Env::default();
    let (client, creator, _) = setup(&env);

    let pool_id = client.save_pool(
        &String::from_str(&env, "Pool"),
        &make_pool_metadata(&env),
        &creator,
        &1_000,
        &(env.ledger().timestamp() + 86_400),
        &None,
        &None,
    );

    client.update_pool_state(&pool_id, &PoolState::Completed);
}

// ---------------------------------------------------------------------------
// extend_campaign_deadline — CampaignAlreadyFunded and max-duration branches
// ---------------------------------------------------------------------------

/// Extending deadline when campaign is already fully funded → CampaignAlreadyFunded.
#[test]
fn test_extend_deadline_already_funded_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal = 500i128;
    let deadline = 10_000u64;
    let id = campaign_id(&env, 1);

    StellarAssetClient::new(&env, &token).mint(&donor, &goal);
    client.create_campaign(
        &id,
        &String::from_str(&env, "Funded"),
        &creator,
        &goal,
        &deadline,
        &token,
    );
    client.donate(&id, &donor, &token, &goal);

    // Campaign is now fully funded — extending deadline must fail
    assert_eq!(
        client.try_extend_campaign_deadline(&id, &(deadline + 1)),
        Err(Ok(CrowdfundingError::CampaignAlreadyFunded))
    );
}

/// Extending deadline beyond the 90-day max from current time → InvalidDeadline.
#[test]
fn test_extend_deadline_exceeds_max_duration_fails() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    env.ledger().with_mut(|l| l.timestamp = 1_000);

    let creator = Address::generate(&env);
    let id = campaign_id(&env, 2);
    let deadline = 50_000u64;

    client.create_campaign(
        &id,
        &String::from_str(&env, "Long"),
        &creator,
        &100_000,
        &deadline,
        &token,
    );

    // 91 days from now exceeds the 90-day cap
    let ninety_one_days: u64 = 91 * 24 * 60 * 60;
    let too_far = env.ledger().timestamp() + ninety_one_days;

    assert_eq!(
        client.try_extend_campaign_deadline(&id, &too_far),
        Err(Ok(CrowdfundingError::InvalidDeadline))
    );
}

// ---------------------------------------------------------------------------
// get_pool_metadata — non-existent pool returns empty strings
// ---------------------------------------------------------------------------

/// get_pool_metadata for a pool that was never created returns three empty strings.
#[test]
fn test_get_pool_metadata_missing_pool_returns_empty() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let (desc, url, hash) = client.get_pool_metadata(&999u64);
    assert_eq!(desc, String::from_str(&env, ""));
    assert_eq!(url, String::from_str(&env, ""));
    assert_eq!(hash, String::from_str(&env, ""));
}

// ---------------------------------------------------------------------------
// is_cause_verified — returns false for unverified address
// ---------------------------------------------------------------------------

/// is_cause_verified returns false for an address that was never verified.
#[test]
fn test_is_cause_verified_returns_false_for_unknown() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let random = Address::generate(&env);
    assert!(!client.is_cause_verified(&random));
}

// ---------------------------------------------------------------------------
// PoolConfig::validate — panic paths
// ---------------------------------------------------------------------------

/// PoolConfig with empty name panics.
#[test]
#[should_panic(expected = "pool name must not be empty")]
fn test_pool_config_empty_name_panics() {
    let env = Env::default();
    let token = Address::generate(&env);
    PoolConfig {
        name: String::from_str(&env, ""),
        description: String::from_str(&env, "desc"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: 0,
        token_address: token,
    }
    .validate();
}

/// PoolConfig with zero target_amount panics.
#[test]
#[should_panic(expected = "target_amount must be > 0")]
fn test_pool_config_zero_target_panics() {
    let env = Env::default();
    let token = Address::generate(&env);
    PoolConfig {
        name: String::from_str(&env, "Pool"),
        description: String::from_str(&env, "desc"),
        target_amount: 0,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: 0,
        token_address: token,
    }
    .validate();
}

/// PoolConfig with min_contribution > target_amount panics.
#[test]
#[should_panic(expected = "min_contribution must be <= target_amount")]
fn test_pool_config_min_contribution_exceeds_target_panics() {
    let env = Env::default();
    let token = Address::generate(&env);
    PoolConfig {
        name: String::from_str(&env, "Pool"),
        description: String::from_str(&env, "desc"),
        target_amount: 100,
        min_contribution: 101,
        is_private: false,
        duration: 86_400,
        created_at: 0,
        token_address: token,
    }
    .validate();
}

/// PoolConfig with zero duration panics.
#[test]
#[should_panic(expected = "duration must be > 0")]
fn test_pool_config_zero_duration_panics() {
    let env = Env::default();
    let token = Address::generate(&env);
    PoolConfig {
        name: String::from_str(&env, "Pool"),
        description: String::from_str(&env, "desc"),
        target_amount: 1_000,
        min_contribution: 0,
        is_private: false,
        duration: 0,
        created_at: 0,
        token_address: token,
    }
    .validate();
}

// ---------------------------------------------------------------------------
// Guard 2 in withdraw_platform_fees — token balance below fee counter
// ---------------------------------------------------------------------------

/// If the contract's token balance is lower than the tracked fee counter
/// (accounting drift), the withdrawal must still fail with InsufficientPlatformFees.
#[test]
fn test_withdraw_platform_fees_token_balance_guard() {
    let env = Env::default();
    let (client, _, token) = setup(&env);

    // Seed the fee counter to 200 but only mint 100 tokens to the contract
    StellarAssetClient::new(&env, &token).mint(&client.address, &100);
    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .set(&crate::base::types::StorageKey::PlatformFees, &200i128);
    });

    let receiver = Address::generate(&env);
    // Requesting 150 — counter says ok (150 < 200) but balance says no (150 > 100)
    assert_eq!(
        client.try_withdraw_platform_fees(&receiver, &150),
        Err(Ok(CrowdfundingError::InsufficientPlatformFees))
    );
}

/// Same guard for withdraw_event_fees.
#[test]
fn test_withdraw_event_fees_token_balance_guard() {
    let env = Env::default();
    let (client, admin, token) = setup(&env);

    StellarAssetClient::new(&env, &token).mint(&client.address, &100);
    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .set(&crate::base::types::StorageKey::EventFeeTreasury, &200i128);
    });

    let receiver = Address::generate(&env);
    assert_eq!(
        client.try_withdraw_event_fees(&admin, &receiver, &150),
        Err(Ok(CrowdfundingError::InsufficientEventFees))
    );
}
