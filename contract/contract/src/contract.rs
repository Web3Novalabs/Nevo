#![allow(clippy::too_many_arguments)]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};

use crate::base::{
    errors::CrowdfundingError,
    types::{
        CampaignDetails, CampaignLifecycleStatus, PoolConfig, PoolContribution, PoolMetadata,
        PoolState,
    },
};
use crate::crowdfunding::CrowdfundingContract;
use crate::interfaces::crowdfunding::CrowdfundingTrait;

/// FundEduContract is the entry-point contract for the FundEdu vertical.
///
/// It delegates every call to [`CrowdfundingContract`], which holds all
/// on-chain state and business logic.  This thin wrapper lets FundEdu be
/// deployed as its own Soroban contract address while sharing the battle-tested
/// implementation underneath.
#[contract]
pub struct FundEduContract;

#[contractimpl]
#[allow(clippy::too_many_arguments)]
impl CrowdfundingTrait for FundEduContract {
    fn create_campaign(
        env: Env,
        id: BytesN<32>,
        title: String,
        creator: Address,
        goal: i128,
        deadline: u64,
        token_address: Address,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::create_campaign(
            env,
            id,
            title,
            creator,
            goal,
            deadline,
            token_address,
        )
    }

    fn get_campaign(env: Env, id: BytesN<32>) -> Result<CampaignDetails, CrowdfundingError> {
        CrowdfundingContract::get_campaign(env, id)
    }

    fn get_campaigns(env: Env, ids: Vec<BytesN<32>>) -> Vec<CampaignDetails> {
        CrowdfundingContract::get_campaigns(env, ids)
    }

    fn get_all_campaigns(env: Env) -> Vec<BytesN<32>> {
        CrowdfundingContract::get_all_campaigns(env)
    }

    fn get_donor_count(env: Env, campaign_id: BytesN<32>) -> Result<u32, CrowdfundingError> {
        CrowdfundingContract::get_donor_count(env, campaign_id)
    }

    fn get_campaign_balance(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError> {
        CrowdfundingContract::get_campaign_balance(env, campaign_id)
    }

    fn get_total_raised(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError> {
        CrowdfundingContract::get_total_raised(env, campaign_id)
    }

    fn get_contribution(
        env: Env,
        campaign_id: BytesN<32>,
        contributor: Address,
    ) -> Result<i128, CrowdfundingError> {
        CrowdfundingContract::get_contribution(env, campaign_id, contributor)
    }

    fn get_campaign_goal(env: Env, campaign_id: BytesN<32>) -> Result<i128, CrowdfundingError> {
        CrowdfundingContract::get_campaign_goal(env, campaign_id)
    }

    fn is_campaign_completed(env: Env, campaign_id: BytesN<32>) -> Result<bool, CrowdfundingError> {
        CrowdfundingContract::is_campaign_completed(env, campaign_id)
    }

    fn get_campaign_status(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<CampaignLifecycleStatus, CrowdfundingError> {
        CrowdfundingContract::get_campaign_status(env, campaign_id)
    }

    fn donate(
        env: Env,
        campaign_id: BytesN<32>,
        donor: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::donate(env, campaign_id, donor, asset, amount)
    }

    fn update_campaign_goal(
        env: Env,
        campaign_id: BytesN<32>,
        new_goal: i128,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::update_campaign_goal(env, campaign_id, new_goal)
    }

    fn cancel_campaign(env: Env, campaign_id: BytesN<32>) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::cancel_campaign(env, campaign_id)
    }

    fn refund_campaign(
        env: Env,
        campaign_id: BytesN<32>,
        contributor: Address,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::refund_campaign(env, campaign_id, contributor)
    }

    fn extend_campaign_deadline(
        env: Env,
        campaign_id: BytesN<32>,
        new_deadline: u64,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::extend_campaign_deadline(env, campaign_id, new_deadline)
    }

    fn claim_campaign_funds(env: Env, campaign_id: BytesN<32>) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::claim_campaign_funds(env, campaign_id)
    }

    fn batch_claim_campaign_funds(
        env: Env,
        campaign_ids: Vec<BytesN<32>>,
    ) -> Vec<Result<(), CrowdfundingError>> {
        CrowdfundingContract::batch_claim_campaign_funds(env, campaign_ids)
    }

    fn get_campaign_fee_history(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<i128, CrowdfundingError> {
        CrowdfundingContract::get_campaign_fee_history(env, campaign_id)
    }

    fn create_pool(
        env: Env,
        creator: Address,
        config: PoolConfig,
    ) -> Result<u64, CrowdfundingError> {
        CrowdfundingContract::create_pool(env, creator, config)
    }

    fn save_pool(
        env: Env,
        name: String,
        metadata: PoolMetadata,
        creator: Address,
        target_amount: i128,
        deadline: u64,
        required_signatures: Option<u32>,
        signers: Option<Vec<Address>>,
    ) -> Result<u64, CrowdfundingError> {
        CrowdfundingContract::save_pool(
            env,
            name,
            metadata,
            creator,
            target_amount,
            deadline,
            required_signatures,
            signers,
        )
    }

    fn get_pool(env: Env, pool_id: u64) -> Option<PoolConfig> {
        CrowdfundingContract::get_pool(env, pool_id)
    }

    fn get_pool_balance(env: Env, pool_id: u64) -> Result<i128, CrowdfundingError> {
        CrowdfundingContract::get_pool_balance(env, pool_id)
    }

    fn get_pool_metadata(env: Env, pool_id: u64) -> (String, String, String) {
        CrowdfundingContract::get_pool_metadata(env, pool_id)
    }

    fn update_pool_metadata_hash(
        env: Env,
        pool_id: u64,
        caller: Address,
        new_hash: String,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::update_pool_metadata_hash(env, pool_id, caller, new_hash)
    }

    fn update_pool_state(
        env: Env,
        pool_id: u64,
        caller: Address,
        new_state: PoolState,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::update_pool_state(env, pool_id, caller, new_state)
    }

    fn set_crowdfunding_token(env: Env, token: Address) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::set_crowdfunding_token(env, token)
    }

    fn get_crowdfunding_token(env: Env) -> Result<Address, CrowdfundingError> {
        CrowdfundingContract::get_crowdfunding_token(env)
    }

    fn set_creation_fee(env: Env, fee: i128) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::set_creation_fee(env, fee)
    }

    fn get_creation_fee(env: Env) -> Result<i128, CrowdfundingError> {
        CrowdfundingContract::get_creation_fee(env)
    }

    fn get_global_raised_total(env: Env) -> i128 {
        CrowdfundingContract::get_global_raised_total(env)
    }

    fn get_top_contributor_for_campaign(
        env: Env,
        campaign_id: BytesN<32>,
    ) -> Result<Address, CrowdfundingError> {
        CrowdfundingContract::get_top_contributor_for_campaign(env, campaign_id)
    }

    fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        creation_fee: i128,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::initialize(env, admin, token, creation_fee)
    }

    fn pause(env: Env) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::pause(env)
    }

    fn unpause(env: Env) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::unpause(env)
    }

    fn is_paused(env: Env) -> bool {
        CrowdfundingContract::is_paused(env)
    }

    fn unpause_pool(env: Env, pool_id: u64, caller: Address) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::unpause_pool(env, pool_id, caller)
    }

    fn contribute(
        env: Env,
        pool_id: u64,
        contributor: Address,
        asset: Address,
        amount: i128,
        is_private: bool,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::contribute(env, pool_id, contributor, asset, amount, is_private)
    }

    fn refund(env: Env, pool_id: u64, contributor: Address) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::refund(env, pool_id, contributor)
    }

    fn request_emergency_withdraw(
        env: Env,
        token: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::request_emergency_withdraw(env, token, amount)
    }

    fn execute_emergency_withdraw(env: Env) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::execute_emergency_withdraw(env)
    }

    fn close_pool(env: Env, pool_id: u64, caller: Address) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::close_pool(env, pool_id, caller)
    }

    fn is_closed(env: Env, pool_id: u64) -> Result<bool, CrowdfundingError> {
        CrowdfundingContract::is_closed(env, pool_id)
    }

    fn renounce_admin(env: Env) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::renounce_admin(env)
    }

    fn get_active_campaign_count(env: Env) -> u32 {
        CrowdfundingContract::get_active_campaign_count(env)
    }

    fn verify_cause(env: Env, cause: Address) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::verify_cause(env, cause)
    }

    fn is_cause_verified(env: Env, cause: Address) -> bool {
        CrowdfundingContract::is_cause_verified(env, cause)
    }

    fn reject_cause(env: Env, cause: Address) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::reject_cause(env, cause)
    }

    fn withdraw_platform_fees(
        env: Env,
        to: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::withdraw_platform_fees(env, to, amount)
    }

    fn withdraw_event_fees(
        env: Env,
        admin: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::withdraw_event_fees(env, admin, to, amount)
    }

    fn set_emergency_contact(env: Env, contact: Address) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::set_emergency_contact(env, contact)
    }

    fn get_emergency_contact(env: Env) -> Result<Address, CrowdfundingError> {
        CrowdfundingContract::get_emergency_contact(env)
    }

    fn get_contract_version(env: Env) -> String {
        CrowdfundingContract::get_contract_version(env)
    }

    fn get_pool_contributions_paginated(
        env: Env,
        pool_id: u64,
        offset: u32,
        limit: u32,
    ) -> Result<Vec<PoolContribution>, CrowdfundingError> {
        CrowdfundingContract::get_pool_contributions_paginated(env, pool_id, offset, limit)
    }

    fn get_pool_remaining_time(env: Env, pool_id: u64) -> Result<u64, CrowdfundingError> {
        CrowdfundingContract::get_pool_remaining_time(env, pool_id)
    }

    fn set_platform_fee_bps(env: Env, fee_bps: u32) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::set_platform_fee_bps(env, fee_bps)
    }

    fn get_platform_fee_bps(env: Env) -> Result<u32, CrowdfundingError> {
        CrowdfundingContract::get_platform_fee_bps(env)
    }

    fn buy_ticket(
        env: Env,
        pool_id: u64,
        buyer: Address,
        asset: Address,
        price: i128,
    ) -> Result<(i128, i128), CrowdfundingError> {
        CrowdfundingContract::buy_ticket(env, pool_id, buyer, asset, price)
    }

    fn upgrade_contract(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::upgrade_contract(env, new_wasm_hash)
    }

    fn claim_pool_funds(env: Env, pool_id: u64, student: Address) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::claim_pool_funds(env, pool_id, student)
    }

    fn get_pool_liquid_balance(env: Env, pool_id: u64) -> Result<i128, CrowdfundingError> {
        CrowdfundingContract::get_pool_liquid_balance(env, pool_id)
    }

    fn withdraw_unallocated(
        env: Env,
        pool_id: u64,
        sponsor: Address,
        amount: i128,
    ) -> Result<(), CrowdfundingError> {
        CrowdfundingContract::withdraw_unallocated(env, pool_id, sponsor, amount)
    }
}
