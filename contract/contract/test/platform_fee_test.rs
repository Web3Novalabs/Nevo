use crate::crowdfunding::CrowdfundingContract;

#[cfg(test)]
extern crate std;

#[test]
fn test_calculate_platform_fee_zero_amount() {
    let fee = CrowdfundingContract::calculate_platform_fee(0, 250);
    assert_eq!(fee, 0, "fee on zero amount should be zero");
}

#[test]
fn test_calculate_platform_fee_zero_bps() {
    let fee = CrowdfundingContract::calculate_platform_fee(10_000, 0);
    assert_eq!(fee, 0, "fee with zero bps should be zero");
}

#[test]
fn test_calculate_platform_fee_standard_case() {
    // 2.5% (250 bps) of 10,000 = 250
    let fee = CrowdfundingContract::calculate_platform_fee(10_000, 250);
    assert_eq!(fee, 250);
}

#[test]
fn test_calculate_platform_fee_1_percent() {
    // 1% (100 bps) of 50,000 = 500
    let fee = CrowdfundingContract::calculate_platform_fee(50_000, 100);
    assert_eq!(fee, 500);
}

#[test]
fn test_calculate_platform_fee_5_percent() {
    // 5% (500 bps) of 20,000 = 1,000
    let fee = CrowdfundingContract::calculate_platform_fee(20_000, 500);
    assert_eq!(fee, 1_000);
}

#[test]
fn test_calculate_platform_fee_10_percent() {
    // 10% (1,000 bps) of 100,000 = 10,000
    let fee = CrowdfundingContract::calculate_platform_fee(100_000, 1_000);
    assert_eq!(fee, 10_000);
}

#[test]
fn test_calculate_platform_fee_100_percent() {
    // 100% (10,000 bps) of 5,000 = 5,000
    let fee = CrowdfundingContract::calculate_platform_fee(5_000, 10_000);
    assert_eq!(fee, 5_000);
}

#[test]
fn test_calculate_platform_fee_small_amount() {
    // 2.5% (250 bps) of 100 = 2 (rounded down)
    let fee = CrowdfundingContract::calculate_platform_fee(100, 250);
    assert_eq!(fee, 2);
}

#[test]
fn test_calculate_platform_fee_rounding_down() {
    // 2.5% (250 bps) of 101 = 2.525, rounds down to 2
    let fee = CrowdfundingContract::calculate_platform_fee(101, 250);
    assert_eq!(fee, 2);
}

#[test]
fn test_calculate_platform_fee_fractional_bps() {
    // 0.01% (1 bps) of 100,000 = 10
    let fee = CrowdfundingContract::calculate_platform_fee(100_000, 1);
    assert_eq!(fee, 10);
}

#[test]
fn test_calculate_platform_fee_large_amount() {
    // 2.5% (250 bps) of 1,000,000,000 = 25,000,000
    let fee = CrowdfundingContract::calculate_platform_fee(1_000_000_000, 250);
    assert_eq!(fee, 25_000_000);
}

#[test]
fn test_calculate_platform_fee_very_large_amount() {
    // Test with large amounts to verify no overflow
    // 1% (100 bps) of 1 trillion = 10 billion
    let fee = CrowdfundingContract::calculate_platform_fee(1_000_000_000_000, 100);
    assert_eq!(fee, 10_000_000_000);
}

#[test]
fn test_calculate_platform_fee_stellar_xlm_amounts() {
    // Stellar XLM has 7 decimal places (1 XLM = 10,000,000 stroops)
    // 2.5% (250 bps) of 100 XLM (1,000,000,000 stroops) = 2.5 XLM (25,000,000 stroops)
    let fee = CrowdfundingContract::calculate_platform_fee(1_000_000_000, 250);
    assert_eq!(fee, 25_000_000);
}

#[test]
fn test_calculate_platform_fee_multiple_scenarios() {
    // Test various realistic donation scenarios
    let test_cases = std::vec![
        (1_000, 250, 25),           // $10 donation, 2.5% fee = $0.25
        (5_000, 250, 125),          // $50 donation, 2.5% fee = $1.25
        (10_000, 250, 250),         // $100 donation, 2.5% fee = $2.50
        (100_000, 250, 2_500),      // $1,000 donation, 2.5% fee = $25
        (1_000_000, 250, 25_000),   // $10,000 donation, 2.5% fee = $250
        (10_000_000, 250, 250_000), // $100,000 donation, 2.5% fee = $2,500
    ];

    for (amount, bps, expected) in test_cases {
        let fee = CrowdfundingContract::calculate_platform_fee(amount, bps);
        assert_eq!(
            fee, expected,
            "Failed for amount={}, bps={}, expected={}",
            amount, bps, expected
        );
    }
}

#[test]
fn test_calculate_platform_fee_edge_case_max_safe_amount() {
    // Test with a very large but safe amount
    // Using i128::MAX / 10_001 to ensure no overflow
    let safe_max = i128::MAX / 10_001;
    let fee = CrowdfundingContract::calculate_platform_fee(safe_max, 100);
    assert!(fee > 0, "fee should be positive for large amounts");
}

#[test]
#[should_panic(expected = "amount must be non-negative")]
fn test_calculate_platform_fee_negative_amount_panics() {
    CrowdfundingContract::calculate_platform_fee(-1000, 250);
}

#[test]
#[should_panic(expected = "fee_bps must be <= 10,000")]
fn test_calculate_platform_fee_invalid_bps_panics() {
    CrowdfundingContract::calculate_platform_fee(1000, 10_001);
}

#[test]
#[should_panic(expected = "fee calculation would overflow")]
fn test_calculate_platform_fee_overflow_panics() {
    // Attempt to cause overflow with maximum values
    CrowdfundingContract::calculate_platform_fee(i128::MAX, 10_000);
}

#[test]
fn test_calculate_platform_fee_precision() {
    // Test precision with various amounts
    // 0.5% (50 bps) of 1,000 = 5
    let fee = CrowdfundingContract::calculate_platform_fee(1_000, 50);
    assert_eq!(fee, 5);

    // 0.25% (25 bps) of 10,000 = 25
    let fee = CrowdfundingContract::calculate_platform_fee(10_000, 25);
    assert_eq!(fee, 25);

    // 0.1% (10 bps) of 100,000 = 100
    let fee = CrowdfundingContract::calculate_platform_fee(100_000, 10);
    assert_eq!(fee, 100);
}

#[test]
fn test_calculate_platform_fee_consistency() {
    // Verify that fee calculation is consistent
    let amount = 50_000;
    let bps = 250;

    let fee1 = CrowdfundingContract::calculate_platform_fee(amount, bps);
    let fee2 = CrowdfundingContract::calculate_platform_fee(amount, bps);

    assert_eq!(fee1, fee2, "fee calculation should be deterministic");
}

#[test]
fn test_calculate_platform_fee_proportionality() {
    // Verify that doubling the amount doubles the fee
    let amount = 10_000;
    let bps = 250;

    let fee1 = CrowdfundingContract::calculate_platform_fee(amount, bps);
    let fee2 = CrowdfundingContract::calculate_platform_fee(amount * 2, bps);

    assert_eq!(fee2, fee1 * 2, "fee should scale proportionally");
}

#[test]
fn test_calculate_platform_fee_additivity() {
    // Verify that fee(a) + fee(b) = fee(a+b)
    let amount_a = 5_000;
    let amount_b = 3_000;
    let bps = 250;

    let fee_a = CrowdfundingContract::calculate_platform_fee(amount_a, bps);
    let fee_b = CrowdfundingContract::calculate_platform_fee(amount_b, bps);
    let fee_combined = CrowdfundingContract::calculate_platform_fee(amount_a + amount_b, bps);

    assert_eq!(
        fee_combined,
        fee_a + fee_b,
        "fee should be additive (within rounding)"
    );
}

#[test]
fn test_calculate_platform_fee_realistic_pool_scenarios() {
    // Test realistic crowdfunding pool scenarios
    struct Scenario {
        description: &'static str,
        amount: i128,
        bps: u32,
        expected_fee: i128,
    }

    let scenarios = std::vec![
        Scenario {
            description: "Small community pool - $500 raised",
            amount: 500_00,      // $500 in cents
            bps: 250,            // 2.5%
            expected_fee: 12_50, // $12.50
        },
        Scenario {
            description: "Medium education fund - $5,000 raised",
            amount: 5_000_00,     // $5,000 in cents
            bps: 250,             // 2.5%
            expected_fee: 125_00, // $125
        },
        Scenario {
            description: "Large medical campaign - $50,000 raised",
            amount: 50_000_00,      // $50,000 in cents
            bps: 250,               // 2.5%
            expected_fee: 1_250_00, // $1,250
        },
        Scenario {
            description: "Mega disaster relief - $1,000,000 raised",
            amount: 1_000_000_00,    // $1,000,000 in cents
            bps: 250,                // 2.5%
            expected_fee: 25_000_00, // $25,000
        },
    ];

    for scenario in scenarios {
        let fee = CrowdfundingContract::calculate_platform_fee(scenario.amount, scenario.bps);
        assert_eq!(
            fee, scenario.expected_fee,
            "Failed for scenario: {}",
            scenario.description
        );
    }
}
