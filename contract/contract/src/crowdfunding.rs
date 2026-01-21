use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String};

use crate::base::{errors::CrowdfundingError, events, types::CampaignDetails};
use crate::interfaces::crowdfunding::CrowdfundingTrait;

#[contract]
pub struct CrowdfundingContract;

#[contractimpl]
impl CrowdfundingTrait for CrowdfundingContract {
    fn create_campaign(
        env: Env,
        id: BytesN<32>,
        title: String,
        creator: Address,
        goal: i128,
        deadline: u64,
    ) -> Result<(), CrowdfundingError> {
        creator.require_auth();

        if title.len() == 0 {
            return Err(CrowdfundingError::InvalidTitle);
        }

        if goal <= 0 {
            return Err(CrowdfundingError::InvalidGoal);
        }

        if deadline <= env.ledger().timestamp() {
            return Err(CrowdfundingError::InvalidDeadline);
        }

        let campaign_key = (id.clone(),);
        if env.storage().instance().has(&campaign_key) {
            return Err(CrowdfundingError::CampaignAlreadyExists);
        }

        let campaign = CampaignDetails {
            id: id.clone(),
            title: title.clone(),
            creator: creator.clone(),
            goal,
            deadline,
        };

        env.storage().instance().set(&campaign_key, &campaign);

        events::campaign_created(&env, id, title, creator, goal, deadline);

        Ok(())
    }

    fn get_campaign(env: Env, id: BytesN<32>) -> Result<CampaignDetails, CrowdfundingError> {
        let campaign_key = (id,);
        env.storage()
            .instance()
            .get(&campaign_key)
            .ok_or(CrowdfundingError::CampaignNotFound)
    }
}
