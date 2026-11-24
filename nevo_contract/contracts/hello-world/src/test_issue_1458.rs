#![cfg(test)]
//! Issue #1458 — get_donor_count never decrements after a full refund_donation.
//!
//! The donor-existence flag (pool_id, "donor", donor) and d_count are set
//! permanently on first donation. refund_donation zeroes the contribution key
//! but never clears the flag or decrements d_count.
//!
//! These tests document the current behaviour: the count is a lifetime
//! unique-donor count, not a current-donor count.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// Issue #1458, test 1.
///
/// A single donor contributes to a pool and then fully refunds. The contract
/// zeroes the contribution record but never clears the donor-existence flag or
/// decrements d_count. get_donor_count therefore still returns 1 after the
/// refund.
#[test]
fn test_donor_count_persists_after_full_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let token = create_token(&env, 1_000_000, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1458 Pool"),
        &String::from_str(&env, "Donor count after refund"),
        &1_000_000i128,
        &1_000_000u64,
    );

    // Set an early ledger-sequence deadline so the grace period is reachable
    // without advancing the timestamp.
    client.set_pool_deadline(&pool_id, &1u32);
    client.set_pool_token(&pool_id, &token);

    client.donate_with_token(&pool_id, &donor, &token, &500_000i128);

    // Confirm the donor is counted before refund.
    assert_eq!(client.get_donor_count(&pool_id), 1);

    // Advance past deadline (1) + grace period (17280 ledgers).
    env.ledger().set_sequence_number(17_281);

    client.refund_donation(&pool_id, &donor, &token);

    // Bug: d_count was never decremented. Count is still 1, not 0.
    assert_eq!(client.get_donor_count(&pool_id), 1);
}

/// Issue #1458, test 2.
///
/// Two donors each contribute and then both fully refund. get_donor_count
/// still reports 2 because the counter is incremented on first donation and
/// never decremented on refund.
#[test]
fn test_donor_count_nonzero_after_all_donors_refunded() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_a = Address::generate(&env);
    let donor_b = Address::generate(&env);

    let token_a = create_token(&env, 500_000, &donor_a);
    // donor_b needs the same token; mint to them too.
    StellarAssetClient::new(&env, &token_a).mint(&donor_b, &500_000i128);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1458 Two-Donor Pool"),
        &String::from_str(&env, "Both donors refund, count stays at 2"),
        &1_000_000i128,
        &1_000_000u64,
    );

    client.set_pool_deadline(&pool_id, &1u32);
    client.set_pool_token(&pool_id, &token_a);

    client.donate_with_token(&pool_id, &donor_a, &token_a, &400_000i128);
    client.donate_with_token(&pool_id, &donor_b, &token_a, &300_000i128);

    assert_eq!(client.get_donor_count(&pool_id), 2);

    env.ledger().set_sequence_number(17_281);

    client.refund_donation(&pool_id, &donor_a, &token_a);
    client.refund_donation(&pool_id, &donor_b, &token_a);

    // Bug: count is never decremented. After both donors refund fully, the
    // contract still reports 2.
    assert_eq!(client.get_donor_count(&pool_id), 2);
}

/// Issue #1458, test 3.
///
/// After a full refund, get_contribution correctly returns 0 (the contribution
/// record is zeroed). get_donor_count, however, still returns 1 because the
/// donor-existence flag was never cleared. This shows the inconsistency
/// between the two getters.
#[test]
fn test_get_contribution_zeroed_but_donor_flag_keeps_count() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let token = create_token(&env, 1_000_000, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Issue 1458 Consistency Pool"),
        &String::from_str(&env, "Contribution zeroed, count not"),
        &1_000_000i128,
        &1_000_000u64,
    );

    client.set_pool_deadline(&pool_id, &1u32);
    client.set_pool_token(&pool_id, &token);

    client.donate_with_token(&pool_id, &donor, &token, &750_000i128);
    assert_eq!(client.get_contribution(&pool_id, &donor), 750_000i128);
    assert_eq!(client.get_donor_count(&pool_id), 1);

    env.ledger().set_sequence_number(17_281);

    client.refund_donation(&pool_id, &donor, &token);

    // Contribution is correctly zeroed after refund.
    assert_eq!(client.get_contribution(&pool_id, &donor), 0i128);

    // Bug: donor count is not decremented — still 1 even though no active
    // contribution exists.
    assert_eq!(client.get_donor_count(&pool_id), 1);
}
