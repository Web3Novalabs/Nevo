use soroban_sdk::{Address, BytesN, Env, String};

use crate::base::errors::SecondCrowdfundingError;

/// A focused trait for `create_campaign` that surfaces string-validation
/// errors directly as [`SecondCrowdfundingError`] without remapping them to
/// the broader [`crate::base::errors::CrowdfundingError`] enum.
///
/// Use this trait (and its implementation on [`crate::crowdfunding::CrowdfundingContract`])
/// when you need to distinguish string-length violations from other contract
/// errors — for example, in unit-tests that verify title/metadata length
/// limits without going through the Soroban client dispatcher.
pub trait SecondCrowdfundingTrait {
    /// Validates the campaign title length and, if valid, creates the campaign.
    ///
    /// Returns [`SecondCrowdfundingError::StringTooLong`] when `title` exceeds
    /// the maximum allowed length (200 characters).  All other errors are
    /// outside the scope of this trait.
    fn create_campaign_checked(
        env: Env,
        id: BytesN<32>,
        title: String,
        creator: Address,
        goal: i128,
        deadline: u64,
        token_address: Address,
    ) -> Result<(), SecondCrowdfundingError>;
}
