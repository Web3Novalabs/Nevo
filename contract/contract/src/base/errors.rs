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
    InvalidMultiSigConfig = 12,
    NotAuthorizedSigner = 13,
    AlreadyApproved = 14,
    DisbursementNotFound = 15,
    DisbursementAlreadyExecuted = 16,
    InsufficientApprovals = 17,
    SignerAlreadyExists = 18,
    SignerNotFound = 19,
    CannotRemoveLastSigner = 20,
    InvalidSignerCount = 21,
}
