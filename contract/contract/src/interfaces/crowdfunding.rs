use soroban_sdk::{Address, BytesN, Env, String};

use crate::base::{errors::CrowdfundingError, types::CampaignDetails};

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
}
