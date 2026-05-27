# Bugfix Requirements Document

## Introduction

When token amounts are split among multiple recipients or milestones using integer (i128/u128)
arithmetic, Rust's integer division silently truncates remainders. These dust fragments are
neither transferred to any recipient nor tracked by the protocol — they become permanently
stranded in the contract's token balance. The fix must ensure every remainder is explicitly
absorbed by a protocol-controlled balance variable so no capital is silently lost.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN a pool's collected amount is divided among N approved students and the amount is not
    evenly divisible by N THEN the system silently discards the remainder (dust), leaving those
    tokens permanently stranded in the contract with no owner.

1.2 WHEN milestone amounts are set up such that their individual values involve integer division
    (e.g., splitting an odd goal across milestones) THEN the system may accept a milestone set
    whose per-milestone disbursements do not fully account for every token unit in the pool goal.

1.3 WHEN `claim_funds` disburses a `claim_amount` that was derived from integer division of the
    pool's collected balance THEN the system transfers only the truncated quotient, and the
    remainder is never credited to any address or protocol variable.

### Expected Behavior (Correct)

2.1 WHEN a pool's collected amount is divided among N approved students and a non-zero remainder
    exists THEN the system SHALL credit the remainder to a protocol dust-absorption balance
    variable rather than leaving it unaccounted.

2.2 WHEN milestone disbursements are computed and a non-zero remainder results from integer
    division THEN the system SHALL route the remainder to the protocol dust-absorption balance
    so that `sum(per_milestone_transfer) + dust_absorbed == total_collected`.

2.3 WHEN `claim_funds` completes a disbursement cycle (all approved students have claimed their
    share) THEN the system SHALL ensure that
    `sum(all_claimed_amounts) + protocol_dust_balance == pool.collected`, with no tokens
    unaccounted for.

### Unchanged Behavior (Regression Prevention)

3.1 WHEN a pool's collected amount is evenly divisible by the number of recipients THEN the
    system SHALL CONTINUE TO transfer the full per-recipient share with zero dust absorbed.

3.2 WHEN a student submits a valid `claim_amount` that is less than or equal to their remaining
    approved allocation THEN the system SHALL CONTINUE TO transfer exactly `claim_amount` tokens
    to the student without modification.

3.3 WHEN `setup_application_milestones` is called with milestone amounts that sum exactly to the
    pool goal THEN the system SHALL CONTINUE TO accept the milestone set and store it unchanged.

3.4 WHEN `donate` is called on an open pool THEN the system SHALL CONTINUE TO increment
    `pool.collected` by the exact donation amount with no truncation or rounding.

3.5 WHEN an overdraw is attempted (amount_claimed + claim_amount > collected) THEN the system
    SHALL CONTINUE TO panic with "Overdraw attempt".

3.6 WHEN a student with a non-approved status attempts to call `claim_funds` THEN the system
    SHALL CONTINUE TO panic with "Application is not approved".
