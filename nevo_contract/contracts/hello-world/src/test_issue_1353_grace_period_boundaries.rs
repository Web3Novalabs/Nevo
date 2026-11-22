#![cfg(test)]
//! Grace period boundary conditions — issue #1353.
//!
//! Covers is_within_grace_period / validate_deadline / set_deadline and the
//! derived refund window, all expressed in the unified ledger-sequence base:
//!   (1) Ledger exactly at the deadline (grace inclusive, deadline exclusive).
//!   (2) One ledger before/after the grace period boundary.
//!   (3) Zero grace_period_ledgers.
//!   (4) validate_deadline rejects deadlines at or before the current ledger.
//!   (5) current_ledger is used consistently across all deadline checks.
//!   (6) The refund window is derived from the funding deadline + grace.

use super::*;

// ── (1) Ledger exactly at the deadline ──────────────────────────────────────

#[test]
fn test_deadline_equal_to_now_is_within_grace_period() {
    let now = current_ledger();
    assert!(
        is_within_grace_period(now, GRACE_PERIOD_LEDGERS),
        "A deadline equal to now has elapsed by zero ledgers and is inside the grace period"
    );
}

#[test]
fn test_deadline_equal_to_now_is_rejected_by_validate_and_set() {
    let now = current_ledger();
    assert_eq!(validate_deadline(now), Err("Deadline in past or now"));
    assert_eq!(set_deadline(now), Err("Deadline must be in future"));
}

// ── (2) One ledger before / after the grace boundary ────────────────────────

#[test]
fn test_one_ledger_before_grace_boundary_is_within() {
    let deadline = current_ledger() - (GRACE_PERIOD_LEDGERS - 1);
    assert!(is_within_grace_period(deadline, GRACE_PERIOD_LEDGERS));
}

#[test]
fn test_exactly_on_grace_boundary_is_within() {
    let deadline = current_ledger() - GRACE_PERIOD_LEDGERS;
    assert!(is_within_grace_period(deadline, GRACE_PERIOD_LEDGERS));
}

#[test]
fn test_one_ledger_after_grace_boundary_is_outside() {
    let deadline = current_ledger() - (GRACE_PERIOD_LEDGERS + 1);
    assert!(!is_within_grace_period(deadline, GRACE_PERIOD_LEDGERS));
}

#[test]
fn test_future_deadline_is_not_within_grace_period() {
    let deadline = current_ledger() + 1;
    assert!(
        !is_within_grace_period(deadline, GRACE_PERIOD_LEDGERS),
        "The grace period only starts once the deadline has passed"
    );
}

// ── (3) Zero grace period ───────────────────────────────────────────────────

#[test]
fn test_zero_grace_period_only_covers_the_deadline_itself() {
    let now = current_ledger();
    assert!(is_within_grace_period(now, 0), "Zero grace still includes the exact deadline");
    assert!(
        !is_within_grace_period(now - 1, 0),
        "With zero grace, one ledger past the deadline is outside"
    );
    assert!(!is_within_grace_period(now + 1, 0));
}

// ── (4) validate_deadline rejects past deadlines ────────────────────────────

#[test]
fn test_validate_deadline_rejects_past_deadlines() {
    let now = current_ledger();
    assert_eq!(validate_deadline(now - 1), Err("Deadline in past or now"));
    assert_eq!(validate_deadline(0), Err("Deadline in past or now"));
}

#[test]
fn test_validate_deadline_accepts_next_ledger() {
    assert_eq!(validate_deadline(current_ledger() + 1), Ok(()));
}

#[test]
fn test_set_deadline_rejects_past_deadlines() {
    assert_eq!(set_deadline(current_ledger() - 1), Err("Deadline must be in future"));
    assert_eq!(set_deadline(current_ledger() + 1), Ok(()));
}

// ── (5) current_ledger is used consistently ─────────────────────────────────

#[test]
fn test_all_deadline_checks_agree_on_current_ledger() {
    let now = current_ledger();
    assert_eq!(now, current_ledger(), "current_ledger must be stable across calls");

    // At `now`: elapsed for the grace check, not in the future for the setters.
    assert!(is_within_grace_period(now, 0));
    assert!(validate_deadline(now).is_err());
    assert!(set_deadline(now).is_err());

    // At `now + 1`: in the future for every check.
    assert!(!is_within_grace_period(now + 1, GRACE_PERIOD_LEDGERS));
    assert!(validate_deadline(now + 1).is_ok());
    assert!(set_deadline(now + 1).is_ok());
}

// ── (6) Derived refund window ───────────────────────────────────────────────

#[test]
fn test_refund_window_is_derived_from_funding_deadline() {
    let funding_deadline = current_ledger() + 100;
    let refund_deadline = derive_refund_deadline(funding_deadline);
    assert_eq!(
        refund_deadline,
        funding_deadline + GRACE_PERIOD_LEDGERS,
        "The refund window must open after the funding deadline plus the grace period"
    );
    assert!(
        refund_deadline >= funding_deadline,
        "A pool can't have contradictory deadlines"
    );
}

#[test]
fn test_pool_without_explicit_refund_deadline_is_refundable_after_grace() {
    let funding_deadline = current_ledger() + 100;
    let refund_deadline = derive_refund_deadline(funding_deadline);

    // Before the funding deadline: donations still open, no refunds.
    assert!(!is_within_grace_period(funding_deadline, GRACE_PERIOD_LEDGERS));
    // At the funding deadline + grace: the refund window is reachable.
    assert!(is_within_grace_period(refund_deadline, GRACE_PERIOD_LEDGERS));
    // Past the grace period: refunds are no longer allowed.
    assert!(!is_within_grace_period(refund_deadline + 1, GRACE_PERIOD_LEDGERS));
}

#[test]
fn test_explicit_refund_deadline_before_funding_deadline_is_rejected() {
    let funding_deadline = current_ledger() + 100;
    assert!(
        validate_refund_deadline(funding_deadline - 1, funding_deadline).is_err(),
        "A refund deadline before the funding deadline would allow refunds while donations are open"
    );
    assert!(validate_refund_deadline(funding_deadline, funding_deadline).is_ok());
    assert!(validate_refund_deadline(funding_deadline + 1, funding_deadline).is_ok());
}
