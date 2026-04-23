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
pub enum DataKey {
    Pool(u64),
    NextPoolId,
}
