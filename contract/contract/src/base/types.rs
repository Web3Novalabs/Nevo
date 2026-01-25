use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignDetails {
    pub id: BytesN<32>,
    pub title: String,
    pub creator: Address,
    pub goal: i128,
    pub deadline: u64,
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
    pub target_amount: i128,
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

        // Target amount must be strictly positive
        assert!(self.target_amount > 0, "target_amount must be > 0");

        // Duration must be strictly positive (non-zero)
        assert!(self.duration > 0, "duration must be > 0");
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
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CampaignMetrics {
    pub total_raised: i128,
    pub contributor_count: u32,
    pub last_donation_at: u64,
}

impl CampaignMetrics {
    pub fn new() -> Self {
        Self {
            total_raised: 0,
            contributor_count: 0,
            last_donation_at: 0,
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
pub enum StorageKey {
    Pool(u64),
    PoolState(u64),
    PoolMetrics(u64),
    AllCampaigns,
    CampaignMetrics(BytesN<32>),
    CampaignDonor(BytesN<32>, Address),

    NextPoolId,
    IsPaused,
    Admin,
    MultiSigConfig(u64),
    DisbursementRequest(u64, u64),
    PoolMetadata(u64),
    NextDisbursementId(u64),
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
            target_amount: 1_000_000,
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
            target_amount: 0,
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
    }

    #[test]
    fn pool_metrics_new_is_zero_initialized() {
        let metrics = PoolMetrics::new();
        assert_eq!(metrics.total_raised, 0);
        assert_eq!(metrics.contributor_count, 0);
        assert_eq!(metrics.last_donation_at, 0);
    }
}
