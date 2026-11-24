#![cfg(test)]
//! Issue #1332: `claim_protocol_fees` admin-only access and correctness.
//!
//! Scenarios:
//!   1. Only the admin can invoke `claim_protocol_fees` successfully.
//!   2. A non-admin caller fails with an authorization error (UnauthorizedAdmin).
//!   3. The correct fee amount is transferred to the admin.
//!   4. The unclaimed-fee balance is reset to zero after the claim.
//!   5. A repeated claim with a zero balance fails (NoUnclaimedFees).

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Env, String,
};

/// Register a Stellar asset contract, mint `amount` to `recipient`, and
/// return the token address.
fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

/// Configure an admin, set a nonzero creation fee, and create one pool via
/// `create_pool_with_fee` so that the contract holds `fee` tokens and the
/// `UNCLAIMED_FEES` accumulator equals `fee`.
fn setup_with_accrued_fees(env: &Env, fee: i128) -> (Address, ContractClient<'_>, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);
    client.set_creation_fee(&admin, &fee);

    let creator = Address::generate(env);
    let fee_token = create_token(env, fee, &creator);
    client.create_pool_with_fee(
        &creator,
        &String::from_str(env, "Fee Pool"),
        &String::from_str(env, "Accrues protocol fees"),
        &1_000_000_000u128,
        &200_000u64,
        &fee_token,
    );

    (admin, client, fee_token)
}

/// (1) + (3) + (4): the admin claims successfully, receives the exact fee, and
/// the accumulator is reset to zero.
#[test]
fn test_admin_claims_protocol_fees_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let fee: i128 = 500_000_000;
    let (admin, client, fee_token) = setup_with_accrued_fees(&env, fee);

    // The pool creation accrued exactly `fee` in unclaimed protocol fees.
    assert_eq!(client.get_unclaimed_fees(), fee);

    let claimed = client.claim_protocol_fees(&admin, &fee_token);
    assert_eq!(claimed, fee);

    // (3) the fee amount was transferred to the admin.
    let token_client = soroban_sdk::token::Client::new(&env, &fee_token);
    assert_eq!(token_client.balance(&admin), fee);

    // (4) the unclaimed-fee accumulator is reset to zero.
    assert_eq!(client.get_unclaimed_fees(), 0);
}

/// (2): a non-admin caller is rejected with `UnauthorizedAdmin` (#3).
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_non_admin_claim_fails_with_authorization_error() {
    let env = Env::default();
    env.mock_all_auths();
    let fee: i128 = 500_000_000;
    let (_admin, client, fee_token) = setup_with_accrued_fees(&env, fee);

    let non_admin = Address::generate(&env);
    let _ = client.claim_protocol_fees(&non_admin, &fee_token);
}

/// (5): after a successful claim the balance is zero, so a second claim fails
/// with `NoUnclaimedFees` (#10).
#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_repeated_claim_with_zero_balance_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let fee: i128 = 500_000_000;
    let (admin, client, fee_token) = setup_with_accrued_fees(&env, fee);

    // First claim drains the accumulated fees.
    let _ = client.claim_protocol_fees(&admin, &fee_token);
    // Second claim: accumulator is zero -> NoUnclaimedFees.
    let _ = client.claim_protocol_fees(&admin, &fee_token);
}
