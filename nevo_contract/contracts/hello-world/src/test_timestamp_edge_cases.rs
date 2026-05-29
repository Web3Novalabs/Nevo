#[cfg(test)]
mod timestamp_edge_case_tests {
    use super::*;

    #[test]
    fn test_deadline_exactly_at_current_time() {
        let now = current_timestamp();
        let result = validate_deadline(now);
        assert!(result.is_err(), "Deadline at current time should be rejected or expired");
    }

    #[test]
    fn test_grace_period_boundary_calculation() {
        let deadline = current_timestamp() - 1;
        let within_grace = is_within_grace_period(deadline, GRACE_PERIOD_SECS);
        assert!(within_grace, "One second past deadline should be within grace period");

        let outside_grace = is_within_grace_period(deadline - GRACE_PERIOD_SECS - 1, GRACE_PERIOD_SECS);
        assert!(!outside_grace, "Beyond grace period must be rejected");
    }

    #[test]
    fn test_timestamp_overflow_scenario() {
        let result = validate_deadline(u64::MAX);
        assert!(result.is_err(), "u64::MAX timestamp must be rejected as invalid");
    }

    #[test]
    fn test_past_timestamp_rejected() {
        let past = current_timestamp().saturating_sub(10_000);
        let result = set_deadline(past);
        assert!(result.is_err(), "Past timestamps must not be accepted as deadlines");
    }

    #[test]
    fn test_future_timestamp_within_limits() {
        let far_future = current_timestamp() + u64::MAX / 2;
        let result = validate_deadline(far_future);
        assert!(result.is_err(), "Unreasonably far future timestamp must be bounded");
    }
}
