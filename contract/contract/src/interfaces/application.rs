use soroban_sdk::{Address, Bytes, Env, String};

use crate::base::{errors::CrowdfundingError, types::{ApplicationDetails}};

/// Defines the user and validator-facing application lifecycle for FundEdu pools.
///
/// This trait separates scholarship application flow from crowdfunding and pool
/// management concerns so that application state and approval logic can evolve
/// independently.
pub trait ApplicationTrait {
    /// Apply for a FundEdu scholarship pool.
    ///
    /// The caller must be the applicant and provide verifiable credentials as raw
    /// bytes that can be referenced off-chain by validators. The requested_amount
    /// specifies how much scholarship funding is being requested.
    ///
    /// Returns DuplicateApplication error if the applicant has already submitted
    /// an application for this pool. Returns InvalidAmount if requested_amount
    /// exceeds the remaining funds in the pool.
    fn apply_for_scholarship(
        env: Env,
        pool_id: u64,
        applicant: Address,
        application_credentials: Bytes,
        requested_amount: i128,
    ) -> Result<(), CrowdfundingError>;

    /// Approve a pending scholarship application.
    ///
    /// Validators should call this after reviewing the applicant's off-chain
    /// credentials. The `validator` address is recorded as the reviewing party.
    fn approve_application(
        env: Env,
        pool_id: u64,
        applicant: Address,
        validator: Address,
        review_note: Option<String>,
    ) -> Result<(), CrowdfundingError>;

    /// Reject a pending scholarship application.
    ///
    /// Validators call this when an application does not qualify for scholarship
    /// support. The rejection reason is stored as optional metadata.
    fn reject_application(
        env: Env,
        pool_id: u64,
        applicant: Address,
        validator: Address,
        rejection_reason: Option<String>,
    ) -> Result<(), CrowdfundingError>;

    /// Retrieve an application record by pool and applicant.
    fn get_application(
        env: Env,
        pool_id: u64,
        applicant: Address,
    ) -> Result<ApplicationDetails, CrowdfundingError>;
}
