#![cfg(test)]
//! Tests for `get_campaign_balance` edge cases — issue #1269 (and #1061).
//!
//! Covers:
//!   1. New campaign returns 0 balance.
//!   2. Campaign with multiple donations returns the correct accumulated total.
//!   3. Nonexistent campaign returns `ContractError::PoolNotFound` (`Error(Contract, #1)`).
//!   4. Campaign balance precisely matches the mathematical sum of all individual donations.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ── helpers ──────────────────────────────────────────────────────────────

/// Create a pool and return its ID. Uses sensible defaults so test bodies
/// stay focused on the behaviour under test.
fn setup_campaign(env: &Env, client: &ContractClient) -> u32 {
    let creator = Address::generate(env);
    client.create_pool(
        &creator,
        &String::from_str(env, "Test Campaign"),
        &String::from_str(env, "A campaign used in unit tests"),
        &10_000_000_000u128, // goal: 10 billion stroops
        &100_000u64,         // application deadline (ledger sequence)
    )
}

// ── Test 1: new campaign returns 0 balance ────────────────────────────────

/// A freshly created campaign must report a balance of exactly 0 before
/// any donations have been made.
#[test]
fn test_new_campaign_balance_is_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    assert_eq!(
        client.get_campaign_balance(&pool_id),
        0u128,
        "A newly created campaign must start with a balance of 0"
    );
}

// ── Test 2: campaign with donations returns correct accumulated total ─────

/// After multiple donations the reported balance must equal their running sum.
/// Each subsequent call to `donate` must increment the stored total, not
/// replace it.
#[test]
fn test_campaign_balance_accumulates_multiple_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);

    let amount1 = 100_000_000u128; // 100 XLM in stroops
    let amount2 = 250_000_000u128; // 250 XLM in stroops
    let amount3 = 50_000_000u128;  //  50 XLM in stroops

    client.donate(&pool_id, &donor1, &amount1);
    // Balance must reflect the single donation immediately.
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1,
        "Balance after first donation must equal that donation"
    );

    client.donate(&pool_id, &donor2, &amount2);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1 + amount2,
        "Balance must accumulate after second donation"
    );

    client.donate(&pool_id, &donor3, &amount3);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1 + amount2 + amount3,
        "Balance must accumulate after third donation"
    );
}

// ── Test 3: querying a nonexistent campaign returns CampaignNotFound ──────

/// Calling `get_campaign_balance` with an ID that has never been created
/// must panic with `ContractError::PoolNotFound` (encoded as
/// `Error(Contract, #1)` in the Soroban XDR error format).
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_campaign_balance_nonexistent_campaign_returns_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Pool ID 9999 has never been created — must panic with PoolNotFound.
    let _ = client.get_campaign_balance(&9999u32);
}

/// Calling `try_get_campaign_balance` with a nonexistent campaign ID returns
/// `ContractError::PoolNotFound` error variant.
#[test]
fn test_campaign_balance_nonexistent_campaign_try_returns_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let res = client.try_get_campaign_balance(&9999u32);
    assert_eq!(
        res,
        Err(Ok(ContractError::PoolNotFound)),
        "Nonexistent campaign must return ContractError::PoolNotFound"
    );
}

// ── Test 4: balance equals mathematical sum of all donations ─────────────

/// The stored campaign balance must equal the exact arithmetic sum of every
/// individual donation amount, with no rounding, truncation, or off-by-one
/// errors. A larger set of donations surfaces any accumulation bugs.
#[test]
fn test_campaign_balance_equals_sum_of_all_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    // Individual donation amounts (in stroops)
    let donation_amounts: &[u128] = &[
        1u128,
        999u128,
        500_000_000u128,
        1_234_567_890u128,
        7u128,
        100_000_000u128,
        42u128,
        333_333_333u128,
    ];

    let mut expected_total: u128 = 0;
    for &amount in donation_amounts {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &amount);
        expected_total += amount;
    }

    let reported_balance = client.get_campaign_balance(&pool_id);

    assert_eq!(
        reported_balance,
        expected_total,
        "Campaign balance ({}) must exactly equal the arithmetic sum of all donations ({})",
        reported_balance,
        expected_total
    );
}

// ── Test 5: same donor making multiple donations increases balance ───────

/// A single donor making multiple separate donations must result in the
/// balance reflecting the sum of all their contributions.
#[test]
fn test_campaign_balance_same_donor_multiple_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);
    let donor = Address::generate(&env);

    let amount1 = 100_000_000u128;
    let amount2 = 200_000_000u128;
    let amount3 = 150_000_000u128;

    client.donate(&pool_id, &donor, &amount1);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1,
        "Balance after first donation from same donor"
    );

    client.donate(&pool_id, &donor, &amount2);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1 + amount2,
        "Balance after second donation from same donor"
    );

    client.donate(&pool_id, &donor, &amount3);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount1 + amount2 + amount3,
        "Balance must accumulate all donations from same donor"
    );
}

// ── Test 6: balance with minimal donation amount (1 stroop) ──────────────

/// The smallest possible donation (1 stroop) must be tracked correctly
/// and reflected in the campaign balance without truncation.
#[test]
fn test_campaign_balance_with_minimal_donation() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);
    let donor = Address::generate(&env);

    let minimal_amount = 1u128;

    client.donate(&pool_id, &donor, &minimal_amount);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        minimal_amount,
        "Balance must correctly track even a 1-stroop donation"
    );
}

// ── Test 7: balance with very large donation amount ──────────────────────

/// A very large donation amount (close to the practical upper limit)
/// must be stored and retrieved correctly without overflow or precision loss.
#[test]
fn test_campaign_balance_with_very_large_donation() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);
    let donor = Address::generate(&env);

    // A very large amount: 100 trillion stroops (1 million XLM)
    let large_amount = 100_000_000_000_000u128;

    client.donate(&pool_id, &donor, &large_amount);
    assert_eq!(
        client.get_campaign_balance(&pool_id),
        large_amount,
        "Balance must correctly store and retrieve very large donation amounts"
    );
}

// ── Test 8: balance remains independent across multiple campaigns ────────

/// Each campaign must maintain its own independent balance. Donations to
/// one campaign must not affect the balance of another campaign.
#[test]
fn test_campaign_balances_are_independent() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id_1 = setup_campaign(&env, &client);
    let pool_id_2 = setup_campaign(&env, &client);

    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    let amount1 = 100_000_000u128;
    let amount2 = 250_000_000u128;

    // Donate to first campaign
    client.donate(&pool_id_1, &donor1, &amount1);
    
    // Donate to second campaign
    client.donate(&pool_id_2, &donor2, &amount2);

    // Verify each campaign has its own correct balance
    assert_eq!(
        client.get_campaign_balance(&pool_id_1),
        amount1,
        "First campaign balance must only reflect its own donations"
    );

    assert_eq!(
        client.get_campaign_balance(&pool_id_2),
        amount2,
        "Second campaign balance must only reflect its own donations"
    );

    // Add another donation to first campaign
    client.donate(&pool_id_1, &donor2, &amount2);
    
    // Verify first campaign increased while second remains unchanged
    assert_eq!(
        client.get_campaign_balance(&pool_id_1),
        amount1 + amount2,
        "First campaign balance must accumulate only its donations"
    );

    assert_eq!(
        client.get_campaign_balance(&pool_id_2),
        amount2,
        "Second campaign balance must remain unchanged"
    );
}

// ── Test 9: balance after many small donations ───────────────────────────

/// A campaign receiving many small donations must accurately track the
/// cumulative total without precision loss or accumulation errors over
/// a large number of operations.
#[test]
fn test_campaign_balance_many_small_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    // 50 donations of 1000 stroops each
    let donation_count = 50u128;
    let donation_amount = 1_000u128;

    for _ in 0..donation_count {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &donation_amount);
    }

    let expected_total = donation_count * donation_amount;
    let reported_balance = client.get_campaign_balance(&pool_id);

    assert_eq!(
        reported_balance,
        expected_total,
        "Balance after {} small donations must equal {} (got {})",
        donation_count,
        expected_total,
        reported_balance
    );
}

// ── Test 10: balance with mixed donation sizes ───────────────────────────

/// A campaign receiving a mix of very small, medium, and large donations
/// must correctly accumulate all amounts regardless of their magnitude.
#[test]
fn test_campaign_balance_mixed_donation_sizes() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    let donations: &[u128] = &[
        1u128,                          // 1 stroop (minimal)
        10_000_000u128,                 // 10 XLM
        100_000_000_000u128,            // 100,000 XLM (large)
        5u128,                          // 5 stroops (very small)
        50_000_000u128,                 // 50 XLM (medium)
        1_000_000_000_000u128,          // 1 million XLM (very large)
        999_999u128,                    // ~1 XLM (odd number)
    ];

    let mut expected_total = 0u128;
    for &amount in donations {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &amount);
        expected_total += amount;
    }

    assert_eq!(
        client.get_campaign_balance(&pool_id),
        expected_total,
        "Balance must correctly accumulate donations of vastly different sizes"
    );
}

// ── Test 11: balance check immediately after each donation ───────────────

/// After each individual donation, the balance must reflect that exact
/// donation immediately without any delay or state inconsistency.
#[test]
fn test_campaign_balance_immediate_consistency() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    let donations: &[u128] = &[
        12_345_678u128,
        98_765_432u128,
        11_111_111u128,
        22_222_222u128,
        33_333_333u128,
    ];

    let mut running_total = 0u128;
    
    for &amount in donations {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &amount);
        running_total += amount;
        
        // Check balance immediately after each donation
        let current_balance = client.get_campaign_balance(&pool_id);
        assert_eq!(
            current_balance,
            running_total,
            "Balance must be immediately consistent after each donation (expected {}, got {})",
            running_total,
            current_balance
        );
    }
}

// ── Test 12: balance with sequential campaign IDs ────────────────────────

/// When multiple campaigns are created sequentially, each must maintain
/// its own independent balance correctly identified by its unique pool ID.
#[test]
fn test_campaign_balance_sequential_campaign_ids() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Create 5 campaigns sequentially
    let pool_ids: Vec<u32> = (0..5)
        .map(|_| setup_campaign(&env, &client))
        .collect();

    // Donate different amounts to each campaign
    let donation_amounts: &[u128] = &[
        100_000_000u128,
        200_000_000u128,
        300_000_000u128,
        400_000_000u128,
        500_000_000u128,
    ];

    for (idx, &pool_id) in pool_ids.iter().enumerate() {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &donation_amounts[idx]);
    }

    // Verify each campaign has its correct balance
    for (idx, &pool_id) in pool_ids.iter().enumerate() {
        assert_eq!(
            client.get_campaign_balance(&pool_id),
            donation_amounts[idx],
            "Campaign {} must have balance {} (got {})",
            pool_id,
            donation_amounts[idx],
            client.get_campaign_balance(&pool_id)
        );
    }
}

// ── Test 13: balance precision with prime number amounts ─────────────────

/// Donations using prime number amounts should accumulate without any
/// rounding or precision loss, verifying exact arithmetic.
#[test]
fn test_campaign_balance_prime_number_amounts() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    // Prime number donation amounts (in stroops)
    let prime_donations: &[u128] = &[
        2u128,
        3u128,
        5u128,
        7u128,
        11u128,
        13u128,
        17u128,
        19u128,
        23u128,
        29u128,
        31u128,
        37u128,
        41u128,
        43u128,
        47u128,
        53u128,
        59u128,
        61u128,
        67u128,
        71u128,
    ];

    let mut expected_total = 0u128;
    for &amount in prime_donations {
        let donor = Address::generate(&env);
        client.donate(&pool_id, &donor, &amount);
        expected_total += amount;
    }

    let reported_balance = client.get_campaign_balance(&pool_id);

    assert_eq!(
        reported_balance,
        expected_total,
        "Balance with prime number donations must be exact (expected {}, got {})",
        expected_total,
        reported_balance
    );

    // Also verify the mathematical sum for transparency
    let sum_of_primes: u128 = prime_donations.iter().sum();
    assert_eq!(
        expected_total,
        sum_of_primes,
        "Expected total must match direct sum of prime donations"
    );
}

// ── Test 14: balance check with zero balance before any operations ───────

/// Multiple sequential balance checks on a newly created campaign with no
/// donations must consistently return zero without any state corruption.
#[test]
fn test_campaign_balance_multiple_zero_checks() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id = setup_campaign(&env, &client);

    // Check balance multiple times before any donations
    for iteration in 0..10 {
        let balance = client.get_campaign_balance(&pool_id);
        assert_eq!(
            balance,
            0u128,
            "Balance check iteration {} must return 0 for campaign with no donations",
            iteration
        );
    }

    // Now make a donation and verify it's tracked
    let donor = Address::generate(&env);
    let amount = 50_000_000u128;
    client.donate(&pool_id, &donor, &amount);

    assert_eq!(
        client.get_campaign_balance(&pool_id),
        amount,
        "Balance must correctly reflect donation after multiple zero checks"
    );
}
