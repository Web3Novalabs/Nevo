use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CrowdfundingError {
    CampaignNotFound = 1,
    InvalidTitle = 2,
    InvalidGoal = 3,
    InvalidDeadline = 4,
    CampaignAlreadyExists = 5,
}
