# fee-math-rounding Bugfix Design

## Overview

Integer division in `claim_funds` and `setup_application_milestones` silently truncates
remainders when a pool's collected balance is split among N recipients or milestones.
The truncated dust is never transferred to any address and is never tracked, leaving tokens
permanently stranded in the contract's token balance.

The fix introduces a per-pool `protocol_dust_balance: u128` storage variable. Whenever a
division produces a non-zero remainder, that remainder is added to `protocol_dust_balance`
rather than discarded. This restores the accounting invariant:

```
sum(all_claimed_amounts) + protocol_dust_balance == pool.collected
```

No existing transfer amounts, guard panics, or donation logic are changed.

## Glossary

- **Bug_Condition (C)**: The condition that triggers the bug — integer division of `collected`
  (or a milestone total) by N produces a non-zero remainder that is neither transferred nor
  tracked.
- **Property (P)**: The desired post-fix behavior — every remainder is credited to
  `protocol_dust_balance` so the full accounting invariant holds.
- **Preservation**: All existing behaviors for non-dust paths (exact claims, donations,
  overdraw guards, auth guards) must remain byte-for-byte identical.
- **`claim_funds`**: The function in `lib.rs` that disburses tokens to an approved student.
  Currently uses `i128` arithmetic; the per-student share is computed externally and passed in
  as `claim_amount`.
- **`setup_application_milestones`**: The function in `lib.rs` that stores per-milestone
  amounts. Currently enforces `sum(milestones) == pool.goal` but does not account for
  per-milestone integer-division remainders.
- **`protocol_dust_balance`**: The new per-pool `u128` storage field that absorbs all
  integer-division remainders, ensuring no token unit is unaccounted for.
- **dust**: The non-zero remainder produced by `collected % N` (or `goal % N`) when the
  division is not exact.

## Bug Details

### Bug Condition

The bug manifests when a pool's `collected` (or `goal`) is divided by N and the result is
not exact. The contract performs the division, uses only the quotient, and silently drops
the remainder. There is no storage variable to receive the remainder, so those token units
are permanently stranded.

**Formal Specification:**
```
FUNCTION isBugCondition(collected: u128, n_recipients: u32) -> bool
  INPUT:  collected     — total tokens in the pool (u128)
          n_recipients  — number of approved students or milestones (u32, > 0)
  OUTPUT: boolean

  remainder := collected % (n_recipients as u128)
  RETURN remainder != 0
END FUNCTION
```

### Examples

- `collected = 10, N = 3` → per-student share = 3, remainder = 1 stranded (bug)
- `collected = 100_000_001, N = 2` → per-student share = 50_000_000, remainder = 1 stranded (bug)
- `collected = 7, N = 7` → per-student share = 1, remainder = 0 → no dust (no bug)
- `collected = 9, N = 3` → per-student share = 3, remainder = 0 → no dust (no bug)

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- `donate` MUST continue to increment `pool.collected` by the exact donation amount with no
  rounding or truncation.
- A student calling `claim_funds` with an explicit `claim_amount` that is ≤ their remaining
  approved allocation MUST continue to receive exactly `claim_amount` tokens.
- `setup_application_milestones` MUST continue to accept milestone sets whose amounts sum
  exactly to the pool goal and store them unchanged.
- The overdraw guard (`amount_claimed + claim_amount > collected` → panic "Overdraw attempt")
  MUST remain in place.
- The auth guard (non-approved status → panic "Application is not approved") MUST remain in
  place.

**Scope:**
All inputs where `collected % N == 0` (exact division) are completely unaffected by this fix.
The only behavioral change is that when a remainder exists, it is credited to
`protocol_dust_balance` instead of being silently dropped.

## Hypothesized Root Cause

1. **No remainder tracking variable**: The contract has no field or storage key to hold
   integer-division remainders. When `claim_amount` is derived from `collected / N`, the
   caller computes only the quotient; the `%` remainder is never computed or stored anywhere
   in the contract.

2. **Implicit truncation in Rust integer arithmetic**: Rust's `/` operator on integer types
   truncates toward zero by definition. There is no compile-time or runtime warning when a
   remainder is discarded, so the loss is invisible.

3. **Missing invariant assertion**: No post-claim assertion verifies
   `sum(claimed) + dust == collected`. Without this check, the accounting gap is never
   surfaced during normal operation or existing tests.

4. **Milestone validation only checks sum == goal**: `setup_application_milestones` enforces
   `sum(milestones) == goal` but does not account for the case where individual milestone
   amounts were themselves derived from integer division, potentially leaving a remainder
   untracked at disbursement time.

## Correctness Properties

Property 1: Bug Condition — Full Accounting Invariant After All Claims

_For any_ pool where `isBugCondition(collected, N)` is true (i.e., `collected % N != 0`),
after all N approved students have called `claim_funds` for their share, the fixed contract
SHALL satisfy:

```
sum(application.amount_claimed for all students) + protocol_dust_balance(pool_id) == pool.collected
```

No token unit may be unaccounted for.

**Validates: Requirements 2.1, 2.2, 2.3**

Property 2: Preservation — Exact-Division Pools Produce Zero Dust

_For any_ pool where `isBugCondition(collected, N)` is false (i.e., `collected % N == 0`),
the fixed contract SHALL produce `protocol_dust_balance(pool_id) == 0` and each student
SHALL receive exactly `collected / N` tokens, identical to the behavior of the original
contract on the same input.

**Validates: Requirements 3.1, 3.2**

## Fix Implementation

### Changes Required

**File**: `Nevo/nevo_contract/contracts/hello-world/src/lib.rs`

**New storage key constant:**
```rust
const DUST_BALANCE_PREFIX: &str = "dust_bal";
```

**New public accessor (add to `impl Contract`):**
```rust
pub fn get_protocol_dust_balance(env: Env, pool_id: u32) -> u128 {
    let key = (Symbol::new(&env, DUST_BALANCE_PREFIX), pool_id);
    env.storage().persistent().get::<_, u128>(&key).unwrap_or(0)
}
```

**Specific Changes to `claim_funds`:**

1. **Compute remainder at claim time**: After the overdraw check passes, compute
   `remainder = (pool.collected as u128) % (n_approved_students as u128)` where
   `n_approved_students` is the count of approved applications for the pool.

2. **Credit remainder to `protocol_dust_balance`**: On the final claim (when
   `application.amount_claimed + claim_amount == collected`), add the remainder to the
   per-pool dust balance:
   ```rust
   let dust_key = (Symbol::new(&env, DUST_BALANCE_PREFIX), pool_id);
   let current_dust: u128 = env.storage().persistent()
       .get::<_, u128>(&dust_key).unwrap_or(0);
   env.storage().persistent().set(&dust_key, &(current_dust + remainder));
   ```

3. **Alternative simpler approach** (recommended for minimal diff): Rather than tracking
   "final claim", compute the remainder on every claim cycle and store it. The invariant
   check can be done in a separate `assert_accounting_invariant` helper used in tests.

**Specific Changes to `setup_application_milestones`:**

4. **Relax the strict equality check**: Change `if sum != pool_data.1` to allow
   `sum + remainder == pool_data.1` where `remainder` is credited to `protocol_dust_balance`
   at milestone setup time, OR keep the strict check and require callers to pass
   milestone amounts that already account for the remainder in the last milestone.

5. **Recommended approach**: Keep the strict `sum == goal` check for milestones (requirement
   3.3 preservation) and handle dust only at `claim_funds` disbursement time, since that is
   where the actual token transfer occurs.

**Summary of minimal change set:**
- Add `DUST_BALANCE_PREFIX` constant
- Add `get_protocol_dust_balance` accessor
- In `claim_funds`, after updating `application.amount_claimed`, compute
  `let remainder = (collected as u128) % (n_recipients as u128)` and persist it to
  `protocol_dust_balance` when the remainder is non-zero and this is the cycle-closing claim

## Testing Strategy

### Validation Approach

Two-phase approach: first run exploratory tests on the unfixed code to surface the accounting
gap as a concrete counterexample, then verify the fix satisfies Property 1 and Property 2.

### Exploratory Bug Condition Checking

**Goal**: Surface a counterexample demonstrating the stranded-dust bug on the unfixed code.
Confirm that `sum(claimed) < pool.collected` when `collected % N != 0`.

**Test Plan**: Set up a pool with `collected = 10`, approve 3 students, have each claim
`10 / 3 = 3` tokens, then assert `sum(claimed) + protocol_dust_balance == 10`. On unfixed
code this assertion fails because `protocol_dust_balance` does not exist and `sum(claimed) = 9`.

**Test Cases:**
1. **Odd split (3 students, 10 tokens)**: collected=10, N=3, each claims 3 → sum=9, 1 stranded
   (will fail invariant check on unfixed code)
2. **Large remainder (2 students, 100_000_001 tokens)**: collected=100_000_001, N=2, each
   claims 50_000_000 → sum=100_000_000, 1 stranded (will fail on unfixed code)
3. **Milestone remainder**: goal=7, 3 milestones of [2,2,2] → sum=6, 1 stranded (will fail
   on unfixed code)
4. **Edge case — single student, indivisible**: collected=1, N=1 → no remainder, no bug
   (should pass on both unfixed and fixed code)

**Expected Counterexamples:**
- `sum(claimed) + 0 < pool.collected` — the gap equals `collected % N`
- No `protocol_dust_balance` storage key exists on unfixed code

### Fix Checking

**Goal**: Verify Property 1 — for all inputs where `isBugCondition` is true, the invariant
holds after the fix.

**Pseudocode:**
```
FOR ALL (collected, n_recipients) WHERE collected % n_recipients != 0 DO
  pool := create_pool_with_collected(collected)
  FOR i IN 1..=n_recipients DO
    claim_funds(student_i, pool_id, collected / n_recipients)
  END FOR
  dust := get_protocol_dust_balance(pool_id)
  total_claimed := sum(get_claimed_amount(pool_id, student_i) for i in 1..=n_recipients)
  ASSERT total_claimed + dust == collected
END FOR
```

### Preservation Checking

**Goal**: Verify Property 2 — for all inputs where `isBugCondition` is false (`collected % N == 0`),
the fixed contract behaves identically to the original.

**Pseudocode:**
```
FOR ALL (collected, n_recipients) WHERE collected % n_recipients == 0 DO
  ASSERT get_protocol_dust_balance(pool_id) == 0
  ASSERT each student receives exactly collected / n_recipients
  ASSERT claim_funds_original(input) == claim_funds_fixed(input)
END FOR
```

**Testing Approach**: Property-based testing is recommended because:
- It generates many `(collected, N)` pairs automatically, covering the full u128 domain
- It catches off-by-one errors in remainder computation that unit tests might miss
- It provides strong guarantees that exact-division behavior is unchanged

**Test Cases:**
1. **Exact-division preservation**: collected=9, N=3 → each student gets 3, dust=0
2. **Donation preservation**: `donate` still increments `collected` exactly
3. **Overdraw guard preservation**: claiming more than `collected` still panics
4. **Auth guard preservation**: non-approved student still panics

### Unit Tests

- `test_dust_absorbed_odd_split`: collected=10, N=3, assert invariant holds after fix
- `test_no_dust_even_split`: collected=9, N=3, assert `protocol_dust_balance == 0`
- `test_single_student_no_dust`: collected=100, N=1, assert dust=0
- `test_large_remainder`: collected=100_000_001, N=2, assert dust=1 after both claims
- `test_overdraw_still_panics`: existing test must continue to pass unchanged
- `test_claim_funds_rejected_still_panics`: existing test must continue to pass unchanged

### Property-Based Tests

- Generate random `(collected: u128, n: u32 in 1..=10)` pairs; after all N claims assert
  `sum(claimed) + dust == collected` (validates Property 1)
- Generate random `(collected: u128, n: u32)` where `collected % n == 0`; assert `dust == 0`
  and each student receives exactly `collected / n` (validates Property 2)
- Generate random valid `claim_amount` values ≤ remaining allocation; assert transfer amount
  is unchanged by the fix (preservation of explicit claims)

### Integration Tests

- Full pool lifecycle: create → donate (odd amount) → approve N students → all claim → assert
  invariant
- Mixed claims: some students claim partial amounts across multiple calls → assert invariant
  holds at each intermediate step and at completion
- Milestone flow: setup milestones with exact sum → disburse → assert no dust introduced by
  milestone path
