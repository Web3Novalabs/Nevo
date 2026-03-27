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
    InvalidMultiSigConfig = 18,
    NotAuthorizedSigner = 19,
    AlreadyApproved = 20,
    DisbursementNotFound = 21,
    DisbursementAlreadyExecuted = 22,
    InsufficientApprovals = 23,
    SignerAlreadyExists = 24,
    SignerNotFound = 25,
    CannotRemoveLastSigner = 26,
    InvalidSignerCount = 27,
    NotInitialized = 28,
    Unauthorized = 29,
    InvalidMetadata = 30,
    CampaignExpired = 31,
    InvalidDonationAmount = 32,
    CampaignAlreadyFunded = 33,
    EmergencyWithdrawalAlreadyRequested = 34,
    EmergencyWithdrawalNotRequested = 35,
    EmergencyWithdrawalPeriodNotPassed = 36,
    InvalidToken = 37,
    InvalidFee = 38,
    InsufficientBalance = 39,
    RefundNotAvailable = 40,
    PoolNotExpired = 41,
    PoolAlreadyDisbursed = 42,
    NoContributionToRefund = 43,
    RefundGracePeriodNotPassed = 44,
    PoolAlreadyClosed = 45,
    PoolNotDisbursedOrRefunded = 46,
    InvalidGoalUpdate = 47,
    InsufficientFees = 48,
    UserBlacklisted = 49,
    CampaignCancelled = 50,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SecondCrowdfundingError {
    StringTooLong = 1,
    EventNotFound = 2,
    EventSoldOut = 3,
    EventExpired = 4,
}

#[cfg(test)]
mod tests {
    use super::SecondCrowdfundingError;

    #[test]
    fn event_not_found_discriminant() {
        assert_eq!(SecondCrowdfundingError::EventNotFound as u32, 2);
    }

    #[test]
    fn event_sold_out_discriminant() {
        assert_eq!(SecondCrowdfundingError::EventSoldOut as u32, 3);
    }

    #[test]
    fn event_expired_discriminant() {
        assert_eq!(SecondCrowdfundingError::EventExpired as u32, 4);
    }

    #[test]
    fn event_errors_are_distinct() {
        assert_ne!(
            SecondCrowdfundingError::EventNotFound,
            SecondCrowdfundingError::EventSoldOut
        );
        assert_ne!(
            SecondCrowdfundingError::EventSoldOut,
            SecondCrowdfundingError::EventExpired
        );
        assert_ne!(
            SecondCrowdfundingError::EventNotFound,
            SecondCrowdfundingError::EventExpired
        );
    }

    #[test]
    fn event_errors_ordering() {
        assert!(SecondCrowdfundingError::EventNotFound < SecondCrowdfundingError::EventSoldOut);
        assert!(SecondCrowdfundingError::EventSoldOut < SecondCrowdfundingError::EventExpired);
    }
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EventError {
    EventNotFound = 1,
    EventExpired = 2,
    EventNotActive = 3,
}
