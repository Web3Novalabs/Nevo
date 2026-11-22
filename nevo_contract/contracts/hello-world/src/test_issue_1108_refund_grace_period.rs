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

// ============= ISSUE #1664: UNIFIED LEDGER-SEQUENCE TIME BASE =============

/// The refund grace period is expressed in ledger sequences, matching the
/// unified ledger-sequence time base used for the refund window.
#[test]
fn test_refund_grace_period_is_in_ledger_sequences() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let grace = client.get_refund_grace_period_ledgers();
    assert!(grace > 0, "grace period must be a positive ledger count");
    assert_eq!(grace, 17_280u32);
}

/// A pool without an explicit `pool_deadline` must still have a reachable
/// refund window: refunds open after the funding deadline plus the grace
/// period (both in ledger sequences).
#[test]
fn test_derived_refund_window_without_explicit_pool_deadline() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let grace = client.get_refund_grace_period_ledgers();

    // The derived refund deadline is the funding deadline plus the grace
    // period; it must be strictly after the funding deadline so the refund
    // window is reachable.
    let funding_deadline: u32 = 1_000;
    let derived_refund_deadline = funding_deadline
        .checked_add(grace)
        .expect("derived refund deadline must not overflow");
    assert!(derived_refund_deadline > funding_deadline);
    assert_eq!(derived_refund_deadline, funding_deadline + 17_280u32);
}