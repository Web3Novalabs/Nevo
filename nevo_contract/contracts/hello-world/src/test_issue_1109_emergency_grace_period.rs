#![cfg(test)]

use super::*;
use soroban_sdk::Env;

// ============= ISSUE #1109: get_emergency_grace_period_secs GETTER =============

/// Assert that get_emergency_grace_period_secs returns the compile-time constant 86_400.
#[test]
fn test_get_emergency_grace_period_secs_returns_86400() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_emergency_grace_period_secs(), 86_400u64);
}
