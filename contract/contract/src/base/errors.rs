use soroban_sdk::contracterror;

/// Documentation for this item.
#[allow(missing_docs)]
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
/// Defines the possible states or errors for crowdfundingerror.
pub enum CrowdfundingError {
    /// CampaignNotFound = 1.
    CampaignNotFound = 1,
    /// InvalidTitle = 2.
    InvalidTitle = 2,
    /// InvalidGoal = 3.
    InvalidGoal = 3,
    /// InvalidDeadline = 4.
    InvalidDeadline = 4,
    /// CampaignAlreadyExists = 5.
    CampaignAlreadyExists = 5,
    /// PoolNotFound = 6.
    PoolNotFound = 6,
    /// InvalidPoolName = 7.
    InvalidPoolName = 7,
    /// InvalidPoolTarget = 8.
    InvalidPoolTarget = 8,
    /// InvalidPoolDeadline = 9.
    InvalidPoolDeadline = 9,
    /// PoolAlreadyExists = 10.
    PoolAlreadyExists = 10,
    /// InvalidPoolState = 11.
    InvalidPoolState = 11,
    /// ContractPaused = 12.
    ContractPaused = 12,
    /// ContractAlreadyPaused = 13.
    ContractAlreadyPaused = 13,
    /// ContractAlreadyUnpaused = 14.
    ContractAlreadyUnpaused = 14,
    /// ContractAlreadyInitialized = 15.
    ContractAlreadyInitialized = 15,
    /// InvalidAmount = 16.
    InvalidAmount = 16,
    /// TokenTransferFailed = 17.
    TokenTransferFailed = 17,
    /// InvalidMultiSigConfig = 18.
    InvalidMultiSigConfig = 18,
    /// NotAuthorizedSigner = 19.
    NotAuthorizedSigner = 19,
    /// AlreadyApproved = 20.
    AlreadyApproved = 20,
    /// DisbursementNotFound = 21.
    DisbursementNotFound = 21,
    /// DisbursementAlreadyExecuted = 22.
    DisbursementAlreadyExecuted = 22,
    /// InsufficientApprovals = 23.
    InsufficientApprovals = 23,
    /// SignerAlreadyExists = 24.
    SignerAlreadyExists = 24,
    /// SignerNotFound = 25.
    SignerNotFound = 25,
    /// CannotRemoveLastSigner = 26.
    CannotRemoveLastSigner = 26,
    /// InvalidSignerCount = 27.
    InvalidSignerCount = 27,
    /// NotInitialized = 28.
    NotInitialized = 28,
    /// Unauthorized = 29.
    Unauthorized = 29,
    /// InvalidMetadata = 30.
    InvalidMetadata = 30,
    /// CampaignExpired = 31.
    CampaignExpired = 31,
    /// InvalidDonationAmount = 32.
    InvalidDonationAmount = 32,
    /// CampaignAlreadyFunded = 33.
    CampaignAlreadyFunded = 33,
    /// EmergencyWithdrawalAlreadyRequested = 34.
    EmergencyWithdrawalAlreadyRequested = 34,
    /// EmergencyWithdrawalNotRequested = 35.
    EmergencyWithdrawalNotRequested = 35,
    /// EmergencyWithdrawalPeriodNotPassed = 36.
    EmergencyWithdrawalPeriodNotPassed = 36,
    /// InvalidToken = 37.
    InvalidToken = 37,
    /// InvalidFee = 38.
    InvalidFee = 38,
    /// InsufficientBalance = 39.
    InsufficientBalance = 39,
    /// RefundNotAvailable = 40.
    RefundNotAvailable = 40,
    /// PoolNotExpired = 41.
    PoolNotExpired = 41,
    /// PoolAlreadyDisbursed = 42.
    PoolAlreadyDisbursed = 42,
    /// NoContributionToRefund = 43.
    NoContributionToRefund = 43,
    /// RefundGracePeriodNotPassed = 44.
    RefundGracePeriodNotPassed = 44,
    /// PoolAlreadyClosed = 45.
    PoolAlreadyClosed = 45,
    /// PoolNotDisbursedOrRefunded = 46.
    PoolNotDisbursedOrRefunded = 46,
    /// InvalidGoalUpdate = 47.
    InvalidGoalUpdate = 47,
    /// InsufficientFees = 48.
    InsufficientFees = 48,
    /// UserBlacklisted = 49.
    UserBlacklisted = 49,
    /// CampaignCancelled = 50.
    CampaignCancelled = 50,
    DeadlinePassed = 51,
    VectorLimitExceeded = 52,
    /// SchoolNotFound = 53.
    SchoolNotFound = 53,
}

/// Documentation for this item.
#[allow(missing_docs)]
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
/// Defines the possible states or errors for secondcrowdfundingerror.
pub enum SecondCrowdfundingError {
    /// StringTooLong = 1.
    StringTooLong = 1,
    /// EventNotFound = 2.
    EventNotFound = 2,
    /// EventSoldOut = 3.
    EventSoldOut = 3,
    /// EventExpired = 4.
    EventExpired = 4,
    InsufficientSponsorBalance = 5,
    ApplicationNotFound = 6,
    ApplicationAlreadySubmitted = 7,
    ApplicationAlreadyReviewed = 8,
    InvalidApplicationCredentials = 9,
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
