use crate::base::errors::CrowdfundingError;

/// One hundred percent expressed in basis points.
pub const MAX_BASIS_POINTS: u32 = 10_000;

/// Calculate a platform fee using basis points.
///
/// # Arguments
/// * `amount`       – The donation / contribution amount (in the token's smallest unit).
/// * `basis_points` – Fee rate in basis points, e.g. `250` = 2.50 %.
///                    Must be in the range `[0, 10_000]`.
///
/// # Returns
/// The fee portion that should be retained by the platform, rounded down.
///
/// # Errors
/// Returns [`CrowdfundingError::InvalidFeeBasisPoints`] when `basis_points > 10_000`.
/// Returns [`CrowdfundingError::InvalidAmount`] when `amount < 0`.
///
/// # Overflow safety
/// The internal multiplication is performed using `i128::checked_mul` so the
/// function never panics even for the maximum `i128` donation value.
pub fn calculate_platform_fee(
    amount: i128,
    basis_points: u32,
) -> Result<i128, CrowdfundingError> {
    if amount < 0 {
        return Err(CrowdfundingError::InvalidAmount);
    }

    if basis_points > MAX_BASIS_POINTS {
        return Err(CrowdfundingError::InvalidFeeBasisPoints);
    }

    // Zero fee short-circuit – avoids any arithmetic entirely.
    if basis_points == 0 || amount == 0 {
        return Ok(0);
    }

    // Overflow-safe path:
    //   fee = amount * basis_points / 10_000
    //
    // `i128::MAX * 10_000` overflows i128 (i128::MAX ≈ 1.7 × 10^38, and
    // 10_000 * 1.7 × 10^38 > i128::MAX), so we use checked_mul and fall
    // back to a wide-integer division when the intermediate value would
    // overflow.
    let fee = match amount.checked_mul(basis_points as i128) {
        Some(product) => product / MAX_BASIS_POINTS as i128,
        None => {
            // amount is too large to multiply directly – divide first to keep
            // values in range, then multiply.  This loses at most
            // (MAX_BASIS_POINTS - 1) units of precision but never overflows.
            (amount / MAX_BASIS_POINTS as i128) * basis_points as i128
        }
    };

    Ok(fee)
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    // ── happy-path ────────────────────────────────────────────────────────

    #[test]
    fn zero_fee_on_zero_bps() {
        assert_eq!(calculate_platform_fee(1_000_000, 0).unwrap(), 0);
    }

    #[test]
    fn zero_fee_on_zero_amount() {
        assert_eq!(calculate_platform_fee(0, 250).unwrap(), 0);
    }

    #[test]
    fn two_and_a_half_percent_small_amount() {
        // 250 bps on 1_000 → 25
        assert_eq!(calculate_platform_fee(1_000, 250).unwrap(), 25);
    }

    #[test]
    fn two_and_a_half_percent_large_amount() {
        // 250 bps on 10_000_000_000 → 250_000_000
        assert_eq!(
            calculate_platform_fee(10_000_000_000, 250).unwrap(),
            250_000_000
        );
    }

    #[test]
    fn one_percent() {
        // 100 bps on 5_000 → 50
        assert_eq!(calculate_platform_fee(5_000, 100).unwrap(), 50);
    }

    #[test]
    fn full_hundred_percent() {
        // 10_000 bps (100%) on 888 → 888
        assert_eq!(calculate_platform_fee(888, 10_000).unwrap(), 888);
    }

    #[test]
    fn flooring_behaviour() {
        // 300 bps on 1 → floor(0.03) = 0
        assert_eq!(calculate_platform_fee(1, 300).unwrap(), 0);
        // 5000 bps on 1 → floor(0.5) = 0
        assert_eq!(calculate_platform_fee(1, 5_000).unwrap(), 0);
    }

    #[test]
    fn max_i128_amount_does_not_overflow() {
        // i128::MAX with 250 bps – should not panic.
        let result = calculate_platform_fee(i128::MAX, 250);
        assert!(result.is_ok(), "should not overflow for i128::MAX");
        // fee must be positive and strictly less than the amount
        let fee = result.unwrap();
        assert!(fee > 0);
        assert!(fee < i128::MAX);
    }

    #[test]
    fn large_realistic_amount() {
        // 1 billion XLM in stroops (1 XLM = 10^7 stroops) at 250 bps
        // 1_000_000_000 × 10^7 = 10^16 stroops
        let one_billion_xlm_stroops: i128 = 10_000_000_000_000_000;
        let fee = calculate_platform_fee(one_billion_xlm_stroops, 250).unwrap();
        assert_eq!(fee, 250_000_000_000_000); // 2.5%
    }

    // ── error cases ───────────────────────────────────────────────────────

    #[test]
    fn rejects_negative_amount() {
        let err = calculate_platform_fee(-1, 250).unwrap_err();
        assert_eq!(err, CrowdfundingError::InvalidAmount);
    }

    #[test]
    fn rejects_basis_points_over_ten_thousand() {
        let err = calculate_platform_fee(1_000, 10_001).unwrap_err();
        assert_eq!(err, CrowdfundingError::InvalidFeeBasisPoints);
    }

    #[test]
    fn rejects_extreme_basis_points() {
        let err = calculate_platform_fee(1_000, u32::MAX).unwrap_err();
        assert_eq!(err, CrowdfundingError::InvalidFeeBasisPoints);
    }
}
