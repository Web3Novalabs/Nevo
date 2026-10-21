#![cfg(test)]

//! Issue #944: timestamp / deadline edge cases.
//!
//! These exercise the deterministic deadline helpers exported from `lib.rs`
//! (`current_timestamp`, `validate_deadline`, `is_within_grace_period`,
//! `set_deadline`) so the boundary arithmetic is covered without depending on
//! external ledger state.

use super::*;

#[test]
fn test_deadline_exactly_at_current_time() {
    let now = current_timestamp();
    let result = validate_deadline(now);
    assert!(
        result.is_err(),
        "Deadline at current time should be rejected as expired"
    );
}

#[test]
fn test_grace_period_boundary_calculation() {
    let deadline = current_timestamp() - 1;
    assert!(
        is_within_grace_period(deadline, GRACE_PERIOD_SECS),
        "One second past deadline should be within grace period"
    );

    // Exactly on the boundary is still inside the grace period.
    let on_boundary = current_timestamp() - GRACE_PERIOD_SECS;
    assert!(
        is_within_grace_period(on_boundary, GRACE_PERIOD_SECS),
        "Deadline exactly GRACE_PERIOD_SECS ago is still within grace period"
    );

    // One second beyond the boundary is not.
    let past_boundary = current_timestamp() - GRACE_PERIOD_SECS - 1;
    assert!(
        !is_within_grace_period(past_boundary, GRACE_PERIOD_SECS),
        "Beyond grace period must be rejected"
    );
}

#[test]
fn test_future_deadline_is_not_within_grace_period() {
    let future = current_timestamp() + 1;
    assert!(
        !is_within_grace_period(future, GRACE_PERIOD_SECS),
        "A deadline that has not passed yet is not within the grace period"
    );
}

#[test]
fn test_timestamp_overflow_scenario() {
    let result = validate_deadline(u64::MAX);
    assert!(
        result.is_err(),
        "u64::MAX timestamp must be rejected as invalid"
    );
}

#[test]
fn test_past_timestamp_rejected() {
    let past = current_timestamp().saturating_sub(10_000);
    let result = set_deadline(past);
    assert!(
        result.is_err(),
        "Past timestamps must not be accepted as deadlines"
    );
}

#[test]
fn test_future_timestamp_beyond_limit_rejected() {
    let far_future = current_timestamp() + u64::MAX / 2;
    let result = validate_deadline(far_future);
    assert!(
        result.is_err(),
        "Unreasonably far future timestamp must be bounded"
    );
}

#[test]
fn test_reasonable_future_deadline_accepted() {
    // One day out is well inside the 10-year bound.
    let deadline = current_timestamp() + 86_400;
    assert!(validate_deadline(deadline).is_ok());
    assert!(set_deadline(deadline).is_ok());
}
