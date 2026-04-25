use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScholarshipPool {
    pub name: String,
    pub sponsor: Address,
    pub target_amount: i128,
    pub token_address: Address,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Application {
    pub student: Address,
    pub requested_amount: i128,
    pub total_granted: i128,
    pub amount_claimed: i128,
    pub status: ApplicationStatus,
    pub milestone_index: u32,
}

#[contracttype]
pub enum DataKey {
    Pool(u64),
    NextPoolId,
    Application(u64, Address),
}
