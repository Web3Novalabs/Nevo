use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::base::{
    errors::CrowdfundingError,
    types::{CampaignDetails, DisbursementRequest, PoolConfig, PoolState},
};

pub trait CrowdfundingTrait {
    fn create_campaign(
        env: Env,
        id: BytesN<32>,
        title: String,
        creator: Address,
        goal: i128,
        deadline: u64,
    ) -> Result<(), CrowdfundingError>;

    fn get_campaign(env: Env, id: BytesN<32>) -> Result<CampaignDetails, CrowdfundingError>;

    fn save_pool(
        env: Env,
        name: String,
        description: String,
        creator: Address,
        target_amount: i128,
        deadline: u64,
        required_signatures: Option<u32>,
        signers: Option<Vec<Address>>,
    ) -> Result<u64, CrowdfundingError>;

    fn get_pool(env: Env, pool_id: u64) -> Option<PoolConfig>;

    fn update_pool_state(
        env: Env,
        pool_id: u64,
        new_state: PoolState,
    ) -> Result<(), CrowdfundingError>;

    fn request_disbursement(
        env: Env,
        pool_id: u64,
        amount: i128,
        recipient: Address,
        requester: Address,
    ) -> Result<u64, CrowdfundingError>;

    fn approve_disbursement(
        env: Env,
        pool_id: u64,
        disbursement_id: u64,
        signer: Address,
    ) -> Result<(), CrowdfundingError>;

    fn execute_disbursement(
        env: Env,
        pool_id: u64,
        disbursement_id: u64,
    ) -> Result<(), CrowdfundingError>;

    fn add_signer(
        env: Env,
        pool_id: u64,
        new_signer: Address,
        caller: Address,
    ) -> Result<(), CrowdfundingError>;

    fn remove_signer(
        env: Env,
        pool_id: u64,
        signer_to_remove: Address,
        caller: Address,
    ) -> Result<(), CrowdfundingError>;

    fn get_disbursement(
        env: Env,
        pool_id: u64,
        disbursement_id: u64,
    ) -> Option<DisbursementRequest>;
}
