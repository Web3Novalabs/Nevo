use soroban_sdk::contracterror;

/// Primary error type for pool, campaign, and financial operations.
/// Hard-capped at 50 variants by the Soroban XDR spec (SCSpecUDTErrorEnumV0).
#[allow(missing_docs)]
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
    ContractAlreadyInit = 15,
    InvalidAmount = 16,
    TokenTransferFailed = 17,
    InvalidMultiSigConfig = 18,
    NotAuthorizedSigner = 19,
    AlreadyApproved = 20,
    DisbursementNotFound = 21,
    DisbursementExecuted = 22,
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
    EmergencyWithdrawReq = 34,
    EmergencyWithdrawMiss = 35,
    EmergencyWithdrawEarly = 36,
    InvalidToken = 37,
    InvalidFee = 38,
    InsufficientBalance = 39,
    RefundNotAvailable = 40,
    PoolNotExpired = 41,
    PoolAlreadyDisbursed = 42,
    NoContributionToRefund = 43,
    RefundGracePeriod = 44,
    PoolAlreadyClosed = 45,
    PoolNotDisbursed = 46,
    InvalidGoalUpdate = 47,
    InsufficientFees = 48,
    UserBlacklisted = 49,
    CampaignCancelled = 50,
}

/// Secondary error type for event, application, and milestone operations.
/// Discriminants 51–60 are reserved for overflow from CrowdfundingError.
#[allow(missing_docs)]
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SecondCrowdfundingError {
    StringTooLong = 1,
    EventNotFound = 2,
    EventSoldOut = 3,
    EventExpired = 4,
    InsufficientSponsorBalance = 5,
    ApplicationNotFound = 6,
    ApplicationAlreadySubmitted = 7,
    ApplicationAlreadyReviewed = 8,
    InvalidApplicationCredentials = 9,
    /// Deadline for applications has passed.
    DeadlinePassed = 51,
    /// Operation would exceed the maximum vector size.
    VectorLimitExceeded = 52,
    /// No application record found for this pool + applicant.
    AppNotFound = 53,
    /// Sum of milestone amounts does not equal requested_amount.
    MilestoneAmountMismatch = 54,
    /// Milestone schedule already set; cannot overwrite.
    MilestonesAlreadySet = 55,
    /// Milestone list must not be empty.
    EmptyMilestones = 56,
}

/// Errors specific to the scholarship application validation flow.
#[allow(missing_docs)]
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ValidationError {
    PoolNotFound = 100,
    Unauthorized = 101,
    ApplicationAlreadyExists = 102,
    ApplicationNotFound = 103,
    ApplicationAlreadyProcessed = 104,
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
