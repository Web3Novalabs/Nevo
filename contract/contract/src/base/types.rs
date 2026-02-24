use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

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
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolMetadata {
    pub description: String,
    pub external_url: String,
    pub image_hash: String,
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
    CampaignFeeHistory(BytesN<32>),
    Blacklist(Address),

    ReentrancyLock(u64),
    EmergencyWithdrawalLock,
    PoolCreator(u64),
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
}
