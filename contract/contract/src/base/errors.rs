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
    PoolNotFound = 6,
    InvalidPoolName = 7,
    InvalidPoolTarget = 8,
    InvalidPoolDeadline = 9,
    PoolAlreadyExists = 10,
    InvalidPoolState = 11,
    ContractPaused = 12,
    ContractAlreadyPaused = 13,
    ContractAlreadyUnpaused = 14,
    ContractAlreadyInitialized = 15,
    InvalidAmount = 16,
    TokenTransferFailed = 17,
}
