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
    InvalidPoolState = 9,
    ContractPaused = 10,
    InvalidAmount = 11,
    TokenTransferFailed = 12,
    Unauthorized = 13,
    InvalidMetadata = 14,
    CampaignExpired = 15,
    InvalidDonationAmount = 16,
    CampaignAlreadyFunded = 17,
    InvalidToken = 18,
    InvalidFee = 19,
    InsufficientBalance = 20,
    RefundNotAvailable = 21,
    PoolAlreadyClosed = 22,
    UserBlacklisted = 23,
    CampaignCancelled = 24,
    ApplicationNotFound = 25,
    ApplicationAlreadySubmitted = 26,
    ApplicationAlreadyReviewed = 27,
    InvalidApplicationCredentials = 28,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ValidationError {
    ApplicationNotFound = 1,
    ApplicationAlreadyProcessed = 2,
    ApplicationAlreadyExists = 3,
    PoolNotFound = 4,
    Unauthorized = 5,
}

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
