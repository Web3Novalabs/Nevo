#![cfg(test)]
//! Grace period boundary conditions — issue #1353.
//!
//! Covers is_within_grace_period / validate_deadline / set_deadline:
//!   (1) Timestamp exactly at the deadline (grace inclusive, deadline exclusive).
//!   (2) One second before/after the grace period boundary.
//!   (3) Zero grace_period_secs.
//!   (4) validate_deadline rejects deadlines in the past.
//!   (5) current_timestamp is used consistently across all deadline checks.

use super::*;

// ── (1) Timestamp exactly at the deadline ───────────────────────────────────

#[test]
fn test_deadline_equal_to_now_is_within_grace_period() {
    let now = current_timestamp();
    assert!(
        is_within_grace_period(now, GRACE_PERIOD_SECS),
        "A deadline equal to now has elapsed by zero seconds and is inside the grace period"
    );
}

#[test]
fn test_deadline_equal_to_now_is_rejected_by_validate_and_set() {
    let now = current_timestamp();
    assert_eq!(validate_deadline(now), Err("Deadline in past or now"));
    assert_eq!(set_deadline(now), Err("Deadline must be in future"));
}

// ── (2) One second before / after the grace boundary ────────────────────────

#[test]
fn test_one_second_before_grace_boundary_is_within() {
    let deadline = current_timestamp() - (GRACE_PERIOD_SECS - 1);
    assert!(is_within_grace_period(deadline, GRACE_PERIOD_SECS));
}

#[test]
fn test_exactly_on_grace_boundary_is_within() {
    let deadline = current_timestamp() - GRACE_PERIOD_SECS;
    assert!(is_within_grace_period(deadline, GRACE_PERIOD_SECS));
}

#[test]
fn test_one_second_after_grace_boundary_is_outside() {
    let deadline = current_timestamp() - (GRACE_PERIOD_SECS + 1);
    assert!(!is_within_grace_period(deadline, GRACE_PERIOD_SECS));
}

#[test]
fn test_future_deadline_is_not_within_grace_period() {
    let deadline = current_timestamp() + 1;
    assert!(
        !is_within_grace_period(deadline, GRACE_PERIOD_SECS),
        "The grace period only starts once the deadline has passed"
    );
}

// ── (3) Zero grace period ───────────────────────────────────────────────────

#[test]
fn test_zero_grace_period_only_covers_the_deadline_itself() {
    let now = current_timestamp();
    assert!(is_within_grace_period(now, 0), "Zero grace still includes the exact deadline");
    assert!(
        !is_within_grace_period(now - 1, 0),
        "With zero grace, one second past the deadline is outside"
    );
    assert!(!is_within_grace_period(now + 1, 0));
}

// ── (4) validate_deadline rejects past deadlines ────────────────────────────

#[test]
fn test_validate_deadline_rejects_past_deadlines() {
    let now = current_timestamp();
    assert_eq!(validate_deadline(now - 1), Err("Deadline in past or now"));
    assert_eq!(validate_deadline(0), Err("Deadline in past or now"));
}

#[test]
fn test_validate_deadline_accepts_next_second() {
    assert_eq!(validate_deadline(current_timestamp() + 1), Ok(()));
}

#[test]
fn test_set_deadline_rejects_past_deadlines() {
    assert_eq!(set_deadline(current_timestamp() - 1), Err("Deadline must be in future"));
    assert_eq!(set_deadline(current_timestamp() + 1), Ok(()));
}

// ── (5) current_timestamp is used consistently ──────────────────────────────

#[test]
fn test_all_deadline_checks_agree_on_current_timestamp() {
    let now = current_timestamp();
    assert_eq!(now, current_timestamp(), "current_timestamp must be stable across calls");

    // At `now`: elapsed for the grace check, not in the future for the setters.
    assert!(is_within_grace_period(now, 0));
    assert!(validate_deadline(now).is_err());
    assert!(set_deadline(now).is_err());

    // At `now + 1`: in the future for every check.
    assert!(!is_within_grace_period(now + 1, GRACE_PERIOD_SECS));
    assert!(validate_deadline(now + 1).is_ok());
    assert!(set_deadline(now + 1).is_ok());
}
