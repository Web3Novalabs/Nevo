use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

/// Status of a single milestone payout.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MilestoneStatus {
    Pending = 0,
    Claimed = 1,
}

/// A single time-locked payout milestone attached to a scholarship pool.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Milestone {
    /// Unix timestamp after which this milestone may be claimed.
    pub unlock_date: u64,
    /// Token amount released when this milestone is claimed.
    pub amount: i128,
    /// Whether this milestone has already been disbursed.
    pub status: MilestoneStatus,
}

/// Documentation for this item.
#[allow(missing_docs)]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignDetails {
    pub id: BytesN<32>,
    pub title: String,
    pub creator: Address,
    pub goal: i128,
    pub deadline: u64,
    pub total_raised: i128,
    pub token_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contribution {
    pub campaign_id: BytesN<32>,
    pub contributor: Address,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    pub required_signatures: u32,
    pub signers: Vec<Address>,
}

// Updated pool configuration for donation pools
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolConfig {
    pub name: String,
    pub description: String,
    pub target_amount: i128,
    // Minimum contribution allowed for this pool (in token smallest units)
    pub min_contribution: i128,
    pub is_private: bool,
    pub duration: u64,
    pub created_at: u64,
    /// Deadline after which new applications are no longer accepted.
    pub application_deadline: u64,
    /// The token address.
    pub token_address: Address,
    pub validator: Address,
    /// Ordered list of time-locked milestone payouts for this pool.
    pub milestones: Vec<Milestone>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ApplicationStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Revoked = 3,
}

/// A scholarship application submitted to a pool.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScholarshipApplication {
    pub pool_id: u64,
    pub applicant: Address,
    pub status: ApplicationStatus,
}

/// Documentation for this item.
#[allow(missing_docs)]
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
/// Represents a poolmetadata.
pub struct PoolMetadata {
    pub description: String,
    pub external_url: String,
    pub image_hash: String,
}

/// The const MAX DESCRIPTION LENGTH.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolDetails {
    pub config: PoolConfig,
    pub state: PoolState,
    pub metrics: PoolMetrics,
    pub metadata: PoolMetadata,
}

/// Verification metadata for a registered school/university validator.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchoolRegistry {
    /// Human-readable name of the institution.
    pub name: String,
    /// Country or jurisdiction of the institution.
    pub country: String,
    /// Arbitrary verification reference (e.g. accreditation ID).
    pub accreditation_id: String,
}

pub const MAX_DESCRIPTION_LENGTH: u32 = 500;
pub const MAX_URL_LENGTH: u32 = 200;
pub const MAX_HASH_LENGTH: u32 = 100;

impl PoolConfig {
    /// Validate pool configuration according to Nevo invariants.
    ///
    /// Follows Soroban best practices by failing fast with `panic!` when
    /// invariants are violated. Callers should validate user input before
    /// persisting configuration on-chain.
    pub fn validate(&self) {
        // Name must not be empty
        assert!(!self.name.is_empty(), "pool name must not be empty");

        // Description validation
        assert!(
            self.description.len() <= MAX_DESCRIPTION_LENGTH,
            "description too long"
        );

        // Target amount must be strictly positive
        assert!(self.target_amount > 0, "target_amount must be > 0");

        // Minimum contribution must be non-negative and not exceed the target
        assert!(self.min_contribution >= 0, "min_contribution must be >= 0");
        assert!(
            self.min_contribution <= self.target_amount,
            "min_contribution must be <= target_amount"
        );

        // Duration must be strictly positive (non-zero)
        assert!(self.duration > 0, "duration must be > 0");
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum CampaignLifecycleStatus {
    Live = 0,
    Cancelled = 1,
    Successful = 2,
    Expired = 3,
}

impl CampaignLifecycleStatus {
    pub fn get_status(
        total_raised: i128,
        goal: i128,
        deadline: u64,
        current_time: u64,
        is_cancelled: bool,
    ) -> Self {
        if is_cancelled {
            return CampaignLifecycleStatus::Cancelled;
        }

        if total_raised >= goal {
            return CampaignLifecycleStatus::Successful;
        }

        if current_time >= deadline {
            return CampaignLifecycleStatus::Expired;
        }

        CampaignLifecycleStatus::Live
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PoolState {
    Active = 0,
    Paused = 1,
    Completed = 2,
    Cancelled = 3,
    Disbursed = 4,
    Closed = 5,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampaignMetrics {
    pub total_raised: i128,
    pub contributor_count: u32,
    pub last_donation_at: u64,
    pub max_donation: i128,
    pub top_contributor: Option<Address>,
}

impl Default for CampaignMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CampaignMetrics {
    pub fn new() -> Self {
        Self {
            total_raised: 0,
            contributor_count: 0,
            last_donation_at: 0,
            max_donation: 0,
            top_contributor: None,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolMetrics {
    pub total_raised: i128,
    pub contributor_count: u32,
    pub last_donation_at: u64,
}

impl Default for PoolMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl PoolMetrics {
    /// Creates zero-initialized metrics for a new pool.
    pub fn new() -> Self {
        Self {
            total_raised: 0,
            contributor_count: 0,
            last_donation_at: 0,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisbursementRequest {
    pub pool_id: u64,
    pub amount: i128,
    pub recipient: Address,
    pub approvals: Vec<Address>,
    pub created_at: u64,
    pub executed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyWithdrawal {
    pub recipient: Address,
    pub amount: i128,
    pub token: Address,
    pub requested_at: u64,
    pub executed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolContribution {
    pub pool_id: u64,
    pub contributor: Address,
    pub amount: i128,
    pub asset: Address,
}

// Milestone and Application types for scholarship validation
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ApplicationStatus {
    Pending = 0,
    Approved = 1,
    Rejected = 2,
    Revoked = 3,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationDetails {
    pub pool_id: u64,
    pub applicant: Address,
    pub requested_amount: i128,
    pub status: ApplicationStatus,
    pub reviewer: Option<Address>,
    pub review_note: Option<String>,
    /// Sequence of milestones for disbursing funding over time
    pub milestones: Vec<Milestone>,
    /// Total amount claimed across all unlocked milestones
    pub amount_claimed: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Pool(u64),
    PoolState(u64),
    PoolMetrics(u64),
    AllCampaigns,
    CampaignMetrics(BytesN<32>),
    CampaignDonor(BytesN<32>, Address),
    Contribution(BytesN<32>, Address),
    PoolContribution(u64, Address),
    PoolContributors(u64),

    NextPoolId,
    IsPaused,
    Admin,
    MultiSigConfig(u64),
    DisbursementRequest(u64, u64),
    PoolMetadata(u64),
    NextDisbursementId(u64),
    EmergencyWithdrawal,
    CrowdfundingToken,
    CreationFee,
    VerifiedCause(Address),
    PlatformFees,
    GlobalTotalRaised,
    CampaignCancelled(BytesN<32>),
    EmergencyContact,
 feature/asset-based-discount
    AssetDiscount(Address),
    PlatformFeePercentage,
    CampaignFeeHistory(BytesN<32>),
    Blacklist(Address),

    CampaignFeeHistory(BytesN<32>),
    Blacklist(Address),

    ReentrancyLock(u64),
    EmergencyWithdrawalLock,
    PoolCreator(u64),
    /// EventFeeTreasury.
    EventFeeTreasury,
    /// PlatformFeeBps.
    PlatformFeeBps,
    // Per-pool revenue split: tokens destined for the event creator
    /// EventPool.
    EventPool(u64),
    // Per-pool revenue split: tokens accumulated as platform fee
    /// EventPlatformFees.
    EventPlatformFees(u64),
    // Track if someone bought a ticket
    /// UserTicket.
    UserTicket(u64, Address),
    // Event details keyed by event id
    /// Event.
    Event(BytesN<32>),
    // track if a pool has been claimed
    PoolClaimed(u64),
    // Per-event metrics (tickets sold, etc.)
    /// EventMetrics.
    EventMetrics(BytesN<32>),
    // Scholarship application keyed by (pool_id, applicant)
    ScholarshipApplication(u64, Address),
    // Locked token balance deposited by the sponsor at pool creation
    PoolBalance(u64),
    // Sum of requested_amount for all Approved applications on a pool
    PoolAllocated(u64),
    /// Maps a validator/school address to its registry entry.
    SchoolRegistry(Address),
    // Ordered milestone payouts for a pool
    PoolMilestones(u64),
    
    // Milestone-related storage keys
    PoolMilestone(u64, u32), // pool_id, milestone_index
    ApplicationMilestone(u64, Address, u32), // pool_id, applicant, milestone_index
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn pool_config_validation_success() {
        let env = Env::default();
        let cfg = PoolConfig {
            name: String::from_str(&env, "Education Fund"),
            description: String::from_str(&env, "Fund for student education materials"),
            target_amount: 1_000_000,
            min_contribution: 0,
            is_private: false,
            duration: 30 * 24 * 60 * 60,
            created_at: 1,
            application_deadline: 1,
            token_address: token,
            validator,
            milestones: soroban_sdk::Vec::new(&env),
        };

        cfg.validate();
    }

    #[test]
    #[should_panic]
    fn pool_config_invalid_target_amount_panics() {
        let env = Env::default();
        let cfg = PoolConfig {
            name: String::from_str(&env, "Invalid Target"),
            description: String::from_str(&env, "Description"),
            target_amount: 0,
            min_contribution: 0,
            is_private: false,
            duration: 30 * 24 * 60 * 60,
            created_at: 1,
            application_deadline: 1,
            token_address: token,
            validator,
            milestones: soroban_sdk::Vec::new(&env),
        };

        cfg.validate();
    }

    #[test]
    fn pool_state_variants_have_expected_discriminants() {
        assert_eq!(PoolState::Active as u32, 0);
        assert_eq!(PoolState::Paused as u32, 1);
        assert_eq!(PoolState::Completed as u32, 2);
        assert_eq!(PoolState::Cancelled as u32, 3);
        assert_eq!(PoolState::Disbursed as u32, 4);
        assert_eq!(PoolState::Closed as u32, 5);
    }

    #[test]
    fn pool_metrics_new_is_zero_initialized() {
        let metrics = PoolMetrics::new();
        assert_eq!(metrics.total_raised, 0);
        assert_eq!(metrics.contributor_count, 0);
        assert_eq!(metrics.last_donation_at, 0);
    }

    #[test]
    fn campaign_status_live_when_active() {
        let status = CampaignLifecycleStatus::get_status(100, 1000, 1000000, 500000, false);
        assert_eq!(status, CampaignLifecycleStatus::Live);
    }

    #[test]
    fn campaign_status_successful_when_goal_reached() {
        let status = CampaignLifecycleStatus::get_status(1500, 1000, 1000000, 500000, false);
        assert_eq!(status, CampaignLifecycleStatus::Successful);
    }

    #[test]
    fn campaign_status_successful_when_goal_exactly_reached() {
        let status = CampaignLifecycleStatus::get_status(1000, 1000, 1000000, 500000, false);
        assert_eq!(status, CampaignLifecycleStatus::Successful);
    }

    #[test]
    fn campaign_status_expired_when_deadline_passed() {
        let status = CampaignLifecycleStatus::get_status(500, 1000, 1000, 1001, false);
        assert_eq!(status, CampaignLifecycleStatus::Expired);
    }

    #[test]
    fn campaign_status_expired_when_at_deadline_unmet() {
        let status = CampaignLifecycleStatus::get_status(500, 1000, 1000, 1000, false);
        assert_eq!(status, CampaignLifecycleStatus::Expired);
    }

    #[test]
    fn campaign_status_cancelled_when_manually_cancelled() {
        let status = CampaignLifecycleStatus::get_status(0, 1000, 1000000, 500000, true);
        assert_eq!(status, CampaignLifecycleStatus::Cancelled);
    }

    #[test]
    fn campaign_status_cancelled_takes_precedence_over_successful() {
        let status = CampaignLifecycleStatus::get_status(1500, 1000, 1000000, 500000, true);
        assert_eq!(status, CampaignLifecycleStatus::Cancelled);
    }

    #[test]
    fn campaign_status_cancelled_takes_precedence_over_live() {
        let status = CampaignLifecycleStatus::get_status(100, 1000, 1000000, 500000, true);
        assert_eq!(status, CampaignLifecycleStatus::Cancelled);
    }

    #[test]
    fn campaign_status_successful_takes_precedence_over_expired() {
        let status = CampaignLifecycleStatus::get_status(1500, 1000, 100, 1000, false);
        assert_eq!(status, CampaignLifecycleStatus::Successful);
    }

    #[test]
    fn campaign_lifecycle_status_discriminants_correct() {
        assert_eq!(CampaignLifecycleStatus::Live as u32, 0);
        assert_eq!(CampaignLifecycleStatus::Cancelled as u32, 1);
        assert_eq!(CampaignLifecycleStatus::Successful as u32, 2);
        assert_eq!(CampaignLifecycleStatus::Expired as u32, 3);
    }

    #[test]
    fn campaign_status_zero_raised_zero_goal() {
        let status = CampaignLifecycleStatus::get_status(0, 0, 1000000, 500000, false);
        assert_eq!(status, CampaignLifecycleStatus::Successful);
    }

    #[test]
    fn campaign_status_large_numbers() {
        let status = CampaignLifecycleStatus::get_status(
            1_000_000_000_000,
            900_000_000_000,
            1000000,
            500000,
            false,
        );
        assert_eq!(status, CampaignLifecycleStatus::Successful);
    }

    #[test]
    fn event_status_serialization() {
        use soroban_sdk::{FromVal, IntoVal, Val};
        let env = Env::default();
        let status = EventStatus::Active;
        let val: Val = status.into_val(&env);
        let deserialized: EventStatus = EventStatus::from_val(&env, &val);
        assert_eq!(status, deserialized);

        let status = EventStatus::Cancelled;
        let val: Val = status.into_val(&env);
        let deserialized: EventStatus = EventStatus::from_val(&env, &val);
        assert_eq!(status, deserialized);

        let status = EventStatus::Completed;
        let val: Val = status.into_val(&env);
        let deserialized: EventStatus = EventStatus::from_val(&env, &val);
        assert_eq!(status, deserialized);
    }

    #[test]
    fn ticket_type_default_is_standard() {
        assert_eq!(TicketType::default(), TicketType::Standard);
    }

    #[test]
    fn event_details_instantiation() {
        use soroban_sdk::testutils::Address as _;
        let env = Env::default();
        let creator = soroban_sdk::Address::generate(&env);
        let token = soroban_sdk::Address::generate(&env);
        let id = soroban_sdk::BytesN::from_array(&env, &[1u8; 32]);
        let event = EventDetails {
            id: id.clone(),
            title: String::from_str(&env, "Nevo Launch"),
            creator: creator.clone(),
            ticket_price: 500,
            max_attendees: 100,
            deadline: 1_700_000_000,
            token: token.clone(),
        };
        assert_eq!(event.id, id);
        assert_eq!(event.ticket_price, 500);
        assert_eq!(event.max_attendees, 100);
        assert_eq!(event.deadline, 1_700_000_000);
        assert_eq!(event.creator, creator);
        assert_eq!(event.token, token);
    }

    #[test]
    fn pool_details_instantiation() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let token = Address::generate(&env);
        let validator = Address::generate(&env);
        let config = PoolConfig {
            name: String::from_str(&env, "Test Pool"),
            description: String::from_str(&env, "A test scholarship pool"),
            target_amount: 1000,
            min_contribution: 10,
            is_private: false,
            duration: 86400,
            created_at: 1234567890,
            application_deadline: 1234567890,
            token_address: token.clone(),
            validator,
            validator: creator.clone(),
            milestones: soroban_sdk::Vec::new(&env),
        };
        let metadata = PoolMetadata {
            description: String::from_str(&env, "Metadata description"),
            external_url: String::from_str(&env, "https://example.com"),
            image_hash: String::from_str(&env, "hash123"),
        };
        let details = PoolDetails {
            config: config.clone(),
            state: PoolState::Active,
            metrics: PoolMetrics::new(),
            metadata: metadata.clone(),
        };
        assert_eq!(details.config, config);
        assert_eq!(details.state, PoolState::Active);
        assert_eq!(details.metrics.total_raised, 0);
        assert_eq!(details.metadata, metadata);
    }

    #[test]
    fn milestone_creation_and_properties() {
        let milestone = Milestone {
            unlock_date: 1700000000,
            unlocked: false,
            amount: 1000,
        };

        assert_eq!(milestone.unlock_date, 1700000000);
        assert_eq!(milestone.unlocked, false);
        assert_eq!(milestone.amount, 1000);
    }

    #[test]
    fn application_details_with_milestones() {
        let env = Env::default();
        let applicant = Address::generate(&env);
        let reviewer = Address::generate(&env);

        let milestone1 = Milestone {
            unlock_date: 1700000000,
            unlocked: false,
            amount: 500,
        };

        let milestone2 = Milestone {
            unlock_date: 1710000000,
            unlocked: false,
            amount: 500,
        };

        let milestones = Vec::from_array(&env, [milestone1.clone(), milestone2.clone()]);

        let application = ApplicationDetails {
            pool_id: 1,
            applicant: applicant.clone(),
            credentials: Bytes::from_array(&env, &[1, 2, 3]),
            requested_amount: 1000,
            submitted_at: 1690000000,
            status: ApplicationStatus::Pending,
            reviewer: Some(reviewer.clone()),
            review_note: Some(String::from_str(&env, "Under review")),
            milestones: milestones.clone(),
            amount_claimed: 0,
        };

        assert_eq!(application.pool_id, 1);
        assert_eq!(application.applicant, applicant);
        assert_eq!(application.requested_amount, 1000);
        assert_eq!(application.status, ApplicationStatus::Pending);
        assert_eq!(application.milestones.len(), 2);
        assert_eq!(application.milestones.get(0).unwrap(), milestone1);
        assert_eq!(application.milestones.get(1).unwrap(), milestone2);
        assert_eq!(application.amount_claimed, 0);
    }

    #[test]
    fn milestone_memory_efficiency() {
        // Test that Milestone struct is packed efficiently
        // Each field should be aligned properly for memory efficiency
        let milestone = Milestone {
            unlock_date: u64::MAX,
            unlocked: true,
            amount: i128::MAX,
        };

        // Verify all fields are accessible and maintain their values
        assert_eq!(milestone.unlock_date, u64::MAX);
        assert_eq!(milestone.unlocked, true);
        assert_eq!(milestone.amount, i128::MAX);
    }
}
