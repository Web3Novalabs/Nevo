use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::base::{
    errors::CrowdfundingError,
    types::{CampaignDetails, CampaignLifecycleStatus, PoolConfig, PoolMetadata, PoolState},
};

pub trait CrowdfundingTrait {
    fn create_campaign(
        env: Env,
        id: BytesN<32>,
        title: String,
        creator: Address,
        goal: i128,
        deadline: u64,
        token_address: Address,
    ) -> Result<(), CrowdfundingError>;

    fn get_campaign(env: Env, id: BytesN<32>) -> Result<CampaignDetails, CrowdfundingError>;

    fn get_campaigns(env: Env, ids: Vec<BytesN<32>>) -> Vec<CampaignDetails>;

    fn get_all_campaigns(env: Env) -> Vec<BytesN<32>>;

    fn get_donor_count(env: Env, campaign_id: BytesN<32>) -> Result<u32, CrowdfundingError>;

    fn get_campaign_balance(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError>;

    fn get_total_raised(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError>;

    fn get_contribution(
        env: Env,
        campaign_id: BytesN<32>,
        contributor: Address,
    ) -> Result<i128, CrowdfundingError>;

    fn get_campaign_goal(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError>;

    fn is_campaign_completed(env: Env, campaign_id: BytesN<32>) -> Result<bool, CrowdfundingError>;

    fn get_campaign_status(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<CampaignLifecycleStatus, CrowdfundingError>;

    fn donate(
        env: Env,
        campaign_id: BytesN<32>,
        donor: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError>;

    fn update_campaign_goal(
        env: Env,
        campaign_id: BytesN<32>,
        new_goal: i128,
    ) -> Result<(), CrowdfundingError>;

    fn cancel_campaign(env: Env, campaign_id: BytesN<32>) -> Result<(), CrowdfundingError>;

    fn refund_campaign(
        env: Env,
        campaign_id: BytesN<32>,
        contributor: Address,
    ) -> Result<(), CrowdfundingError>;

    fn extend_campaign_deadline(
        env: Env,
        campaign_id: BytesN<32>,
        new_deadline: u64,
    ) -> Result<(), CrowdfundingError>;

    fn get_campaign_fee_history(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<i128, CrowdfundingError>;

    fn create_pool(
        env: Env,
        creator: Address,
        config: PoolConfig,
    ) -> Result<u64, CrowdfundingError>;

    #[allow(clippy::too_many_arguments)]
    fn save_pool(
        env: Env,
        name: String,
        metadata: PoolMetadata,
        creator: Address,
        target_amount: i128,
        deadline: u64,
        required_signatures: Option<u32>,
        signers: Option<Vec<Address>>,
    ) -> Result<u64, CrowdfundingError>;

    fn get_pool(env: Env, pool_id: u64) -> Option<PoolConfig>;

    fn get_pool_metadata(env: Env, pool_id: u64) -> (String, String, String);

    fn update_pool_state(
        env: Env,
        pool_id: u64,
        new_state: PoolState,
    ) -> Result<(), CrowdfundingError>;

    fn set_crowdfunding_token(env: Env, token: Address) -> Result<(), CrowdfundingError>;

    fn get_crowdfunding_token(env: Env) -> Result<Address, CrowdfundingError>;

    fn set_creation_fee(env: Env, fee: i128) -> Result<(), CrowdfundingError>;

    fn get_creation_fee(env: Env) -> Result<i128, CrowdfundingError>;

    fn get_global_raised_total(env: Env) -> i128;

    fn get_top_contributor_for_campaign(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<Address, CrowdfundingError>;

    fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        creation_fee: i128,
    ) -> Result<(), CrowdfundingError>;

    fn pause(env: Env) -> Result<(), CrowdfundingError>;

    fn unpause(env: Env) -> Result<(), CrowdfundingError>;

    fn is_paused(env: Env) -> bool;

    fn contribute(
        env: Env,
        pool_id: u64,
        contributor: Address,
        asset: Address,
        amount: i128,
        is_private: bool,
    ) -> Result<(), CrowdfundingError>;

    fn refund(env: Env, pool_id: u64, contributor: Address) -> Result<(), CrowdfundingError>;

    fn request_emergency_withdraw(
        env: Env,
        token: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError>;

    fn execute_emergency_withdraw(env: Env) -> Result<(), CrowdfundingError>;

    fn close_pool(env: Env, pool_id: u64, caller: Address) -> Result<(), CrowdfundingError>;

    fn is_closed(env: Env, pool_id: u64) -> Result<bool, CrowdfundingError>;

    fn renounce_admin(env: Env) -> Result<(), CrowdfundingError>;

    fn get_active_campaign_count(env: Env) -> u32;
    fn verify_cause(env: Env, cause: Address) -> Result<(), CrowdfundingError>;

    fn is_cause_verified(env: Env, cause: Address) -> bool;

    fn withdraw_platform_fees(
        env: Env,
        admin: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError>;

    fn set_emergency_contact(env: Env, contact: Address) -> Result<(), CrowdfundingError>;

    fn get_emergency_contact(env: Env) -> Result<Address, CrowdfundingError>;

    fn get_contract_version(env: Env) -> String;

    fn blacklist_address(env: Env, address: Address) -> Result<(), CrowdfundingError>;

    fn unblacklist_address(env: Env, address: Address) -> Result<(), CrowdfundingError>;

    fn is_blacklisted(env: Env, address: Address) -> bool;

    fn update_pool_metadata_hash(
        env: Env,
        pool_id: u64,
        caller: Address,
        new_metadata_hash: String,
    ) -> Result<(), CrowdfundingError>;
}
