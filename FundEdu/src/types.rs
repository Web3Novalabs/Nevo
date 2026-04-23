use soroban_sdk::{contracttype, Address};

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

#[contracttype]
pub enum DataKey {
    Pool(u64),
    Application(u64, Address),
    NextPoolId,
}
