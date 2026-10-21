#![cfg(test)]

use super::*;
use soroban_sdk::Env;

// ============= ISSUE #1108: get_refund_grace_period_ledgers GETTER =============

/// Assert that get_refund_grace_period_ledgers returns the compile-time constant 17_280.
#[test]
fn test_get_refund_grace_period_ledgers_returns_17280() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_refund_grace_period_ledgers(), 17_280u32);
}
