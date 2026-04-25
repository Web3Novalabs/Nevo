use soroban_sdk::{contracttype, contract, contractimpl};

/// Test struct documentation
#[allow(missing_docs)]
#[contracttype]
pub struct TestDoc {
    /// field doc
    pub a: u32,
}

/// Test contract documentation
#[allow(missing_docs)]
#[contract]
pub struct TestContract;

#[allow(missing_docs)]
#[contractimpl]
impl TestContract {
    /// test fn
    pub fn do_thing() {}
}
