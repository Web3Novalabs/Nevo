use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    PoolNotFound = 1,
    ApplicationNotFound = 2,
    ExceedsGrant = 3,
    PoolNotActive = 4,
    NotApproved = 5,
    InvalidAmount = 6,
}
