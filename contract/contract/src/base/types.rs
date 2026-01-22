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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolConfig {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub creator: Address,
    pub target_amount: i128,
    pub deadline: u64,
    pub created_at: u64,
    pub multi_sig_config: Option<MultiSigConfig>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PoolState {
    Active = 0,
    Paused = 1,
    Completed = 2,
    Cancelled = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolMetrics {
    pub total_donations: i128,
    pub donor_count: u32,
    pub last_donation_at: u64,
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
    NextPoolId,
    DisbursementRequest(u64, u64),
    NextDisbursementId(u64),
}
