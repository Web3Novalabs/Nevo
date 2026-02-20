/// Integration tests for `calculate_platform_fee` via the Soroban contract client.
///
/// Unlike the unit tests in `base/fees.rs`, these tests exercise the full
/// contract dispatch path — i.e. they call through `CrowdfundingContractClient`
/// the same way a real off-chain caller would.
#[cfg(test)]
mod tests {
    use soroban_sdk::Env;
    use crate::{CrowdfundingContract, CrowdfundingContractClient};
    use crate::base::errors::CrowdfundingError;

    /// Spin up a minimal test environment and return a client.
    /// `calculate_platform_fee` is pure and does not touch storage, so we
    /// only need to register the contract — no `initialize` call is required.
    fn setup(env: &Env) -> CrowdfundingContractClient<'_> {
        env.mock_all_auths();
        let contract_id = env.register(CrowdfundingContract, ());
        CrowdfundingContractClient::new(env, &contract_id)
    }

    // ── happy-path ────────────────────────────────────────────────────────

    #[test]
    fn two_and_half_percent_small() {
        let env = Env::default();
        let client = setup(&env);
        // 250 bps on 1_000 → 25
        assert_eq!(client.calculate_platform_fee(&1_000, &250), 25);
    }

    #[test]
    fn two_and_half_percent_large() {
        let env = Env::default();
        let client = setup(&env);
        // 250 bps on 10_000_000_000 → 250_000_000
        assert_eq!(
            client.calculate_platform_fee(&10_000_000_000, &250),
            250_000_000
        );
    }

    #[test]
    fn one_percent() {
        let env = Env::default();
        let client = setup(&env);
        // 100 bps on 5_000 → 50
        assert_eq!(client.calculate_platform_fee(&5_000, &100), 50);
    }

    #[test]
    fn zero_bps_yields_zero_fee() {
        let env = Env::default();
        let client = setup(&env);
        assert_eq!(client.calculate_platform_fee(&999_999, &0), 0);
    }

    #[test]
    fn zero_amount_yields_zero_fee() {
        let env = Env::default();
        let client = setup(&env);
        assert_eq!(client.calculate_platform_fee(&0, &250), 0);
    }

    #[test]
    fn full_hundred_percent() {
        let env = Env::default();
        let client = setup(&env);
        // 10_000 bps = 100% → fee == amount
        assert_eq!(client.calculate_platform_fee(&888, &10_000), 888);
    }

    #[test]
    fn no_overflow_on_large_amount() {
        let env = Env::default();
        let client = setup(&env);
        // i128::MAX with 250 bps – must not panic and must be > 0
        let fee = client.calculate_platform_fee(&i128::MAX, &250);
        assert!(fee > 0);
        assert!(fee < i128::MAX);
    }

    #[test]
    fn realistic_xlm_stroop_amount() {
        let env = Env::default();
        let client = setup(&env);
        // 1 billion XLM in stroops at 250 bps
        let billion_xlm: i128 = 10_000_000_000_000_000;
        assert_eq!(
            client.calculate_platform_fee(&billion_xlm, &250),
            250_000_000_000_000 // 2.5%
        );
    }

    // ── error cases ───────────────────────────────────────────────────────

    #[test]
    fn rejects_negative_amount() {
        let env = Env::default();
        let client = setup(&env);
        let result = client.try_calculate_platform_fee(&-1, &250);
        assert_eq!(
            result.unwrap_err().unwrap(),
            CrowdfundingError::InvalidAmount
        );
    }

    #[test]
    fn rejects_bps_over_ten_thousand() {
        let env = Env::default();
        let client = setup(&env);
        let result = client.try_calculate_platform_fee(&1_000, &10_001);
        assert_eq!(
            result.unwrap_err().unwrap(),
            CrowdfundingError::InvalidFeeBasisPoints
        );
    }

    #[test]
    fn rejects_extreme_bps() {
        let env = Env::default();
        let client = setup(&env);
        let result = client.try_calculate_platform_fee(&1_000, &u32::MAX);
        assert_eq!(
            result.unwrap_err().unwrap(),
            CrowdfundingError::InvalidFeeBasisPoints
        );
    }
}
