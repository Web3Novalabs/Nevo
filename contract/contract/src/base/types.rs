use soroban_sdk::{contracttype, Address, BytesN, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CampaignDetails {
    pub id: BytesN<32>,
    pub title: String,
    pub creator: Address,
    pub goal: i128,
    pub deadline: u64,
}
