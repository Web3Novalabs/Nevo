use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pool {
    pub sponsor: Address,
    pub token: Address,
    pub total_funds: i128,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application {
    pub student: Address,
    pub requested_amount: i128,
    pub status: ApplicationStatus,
    pub milestone_index: u32,
}

/// Verification metadata for a registered school/university.
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

#[contracttype]
pub enum DataKey {
    Pool(u64),
    Application(u64, Address),
    NextPoolId,
    /// Maps a school/validator address to its registry entry.
    SchoolRegistry(Address),
}
