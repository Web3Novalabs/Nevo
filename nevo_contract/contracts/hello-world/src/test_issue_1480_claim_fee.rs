#![cfg(test)]

//! Issue #1480: `claim_funds` 1% fee is `claim_amount / 100` (integer floor).
//!
//! Repository policy: the contract comment says "1% of claim amount" and the
//! implementation uses truncating integer division. Existing claim tests use
//! amounts ≥ 100 so they never observe a zero fee. Nothing documents a
//! minimum fee or ceil rounding, so production arithmetic is left unchanged.

use super::*;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, BytesN, Env, String,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn unclaimed_fees(env: &Env, contract_id: &Address) -> i128 {
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .get::<_, i128>(&Symbol::new(env, UNCLAIMED_FEES))
            .unwrap_or(0)
    })
}

fn setup_claimable(
    env: &Env,
    collected: u128,
) -> (Address, ContractClient<'_>, u32, Address, Address) {
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.set_admin(&admin);

    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[6u8; 32]));

    let creator = Address::generate(env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(env, "Fee Pool"),
        &String::from_str(env, "Fee table"),
        &collected,
        &school,
        &200_000u64,
    );
    client.donate(&pool_id, &creator, &collected);
    let token = create_token(env, collected as i128, &contract_id);

    let student = Address::generate(env);
    client.set_application_status(
        &admin,
        &pool_id,
        &student,
        &String::from_str(env, "Approved"),
    );

    (contract_id, client, pool_id, student, token)
}

fn expected_fee(claim_amount: i128) -> i128 {
    claim_amount / 100
}

/// Table-driven exact splits for 1, 50, 99, 100, 101 plus nearby boundaries.
#[test]
fn test_claim_funds_fee_table_truncating_integer_division() {
    let env = Env::default();
    env.mock_all_auths();

    let cases: [(i128, i128, i128); 7] = [
        (1, 0, 1),
        (50, 0, 50),
        (99, 0, 99),
        (100, 1, 99),
        (101, 1, 100),
        (199, 1, 198),
        (200, 2, 198),
    ];
    let total: i128 = cases.iter().map(|(a, _, _)| *a).sum();

    let (contract_id, client, pool_id, student, token) = setup_claimable(&env, total as u128);
    let token_client = token::Client::new(&env, &token);
    let mut fees_before = unclaimed_fees(&env, &contract_id);
    let mut student_before = token_client.balance(&student);

    for (claim_amount, fee, net) in cases {
        assert_eq!(fee, expected_fee(claim_amount));
        assert_eq!(fee + net, claim_amount);
        assert!(fee >= 0 && fee <= claim_amount);
        assert!(net >= 0);

        client.claim_funds(&student, &pool_id, &claim_amount, &token);

        let fees_after = unclaimed_fees(&env, &contract_id);
        let student_after = token_client.balance(&student);
        assert_eq!(fees_after - fees_before, fee);
        assert_eq!(student_after - student_before, net);
        fees_before = fees_after;
        student_before = student_after;
    }
}

#[test]
#[should_panic(expected = "Claim amount must be positive")]
fn test_claim_funds_zero_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (_contract_id, client, pool_id, student, token) = setup_claimable(&env, 1_000u128);
    client.claim_funds(&student, &pool_id, &0i128, &token);
}

/// Many sub-100 claims contribute 0 to UNCLAIMED_FEES; one equal aggregate
/// claim contributes floor(total / 100).
#[test]
fn test_claim_funds_sub_100_fragmentation_avoids_fees() {
    let env = Env::default();
    env.mock_all_auths();

    let fragment: i128 = 50;
    let fragments: i128 = 40;
    let fragmented_total = fragment * fragments;
    let aggregate = fragmented_total;
    let collected = (fragmented_total + aggregate) as u128;

    let (contract_id, client, pool_id, student, token) = setup_claimable(&env, collected);
    let fees_start = unclaimed_fees(&env, &contract_id);

    for _ in 0..fragments {
        client.claim_funds(&student, &pool_id, &fragment, &token);
    }
    let fees_after_fragments = unclaimed_fees(&env, &contract_id);
    assert_eq!(
        fees_after_fragments - fees_start,
        0,
        "each claim of {fragment} floors to fee 0, so {fragments} fragments add nothing"
    );

    client.claim_funds(&student, &pool_id, &aggregate, &token);
    let fees_after_aggregate = unclaimed_fees(&env, &contract_id);
    assert_eq!(
        fees_after_aggregate - fees_after_fragments,
        aggregate / 100,
        "one claim of {aggregate} pays the truncated 1% that the fragments skipped"
    );
    assert_eq!(
        fees_after_aggregate - fees_start,
        aggregate / 100,
        "fragmentation under 100 units avoids fee that an equivalent lump claim pays"
    );
}
