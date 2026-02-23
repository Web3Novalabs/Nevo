# Minimum Donation Limit Feature Summary

## Overview

Added a minimum donation limit feature for pools that allows pool creators to set a minimum amount (e.g., 5 XLM) to prevent dust contributions.

## Changes Made

### 1. Error Type Addition

**File**: [contract/src/base/errors.rs](contract/src/base/errors.rs)

- Added new error variant: `BelowMinimumDonation = 34`
- Used when a contribution amount is below the pool's minimum donation requirement

### 2. PoolConfig Structure Update

**File**: [contract/src/base/types.rs](contract/src/base/types.rs)

- Added field: `pub minimum_donation: i128` to `PoolConfig` struct
- Updated `validate()` method to enforce:
  - `minimum_donation >= 0` (non-negative)
  - `minimum_donation <= target_amount` (cannot exceed pool target)
- Updated unit tests to include the new field

### 3. Trait Interface Update

**File**: [contract/src/interfaces/crowdfunding.rs](contract/src/interfaces/crowdfunding.rs)

- Updated `save_pool()` function signature to include `minimum_donation: i128` parameter
- Parameter order: `name`, `metadata`, `creator`, `target_amount`, `deadline`, `minimum_donation`, `required_signatures`, `signers`

### 4. save_pool Implementation

**File**: [contract/src/crowdfunding.rs](contract/src/crowdfunding.rs)

- Added validation:
  - Minimum donation cannot be negative: `if minimum_donation < 0`
  - Minimum donation cannot exceed target: `if minimum_donation > target_amount`
  - Both return `CrowdfundingError::InvalidAmount`
- Store minimum_donation in PoolConfig when creating pool

### 5. contribute Function Enhancement

**File**: [contract/src/crowdfunding.rs](contract/src/crowdfunding.rs)

- Fetch PoolConfig to access minimum_donation
- Added validation before token transfer:
  ```rust
  if amount < pool_config.minimum_donation {
      return Err(CrowdfundingError::BelowMinimumDonation);
  }
  ```
- Prevents contributions below the minimum

### 6. Comprehensive Test Coverage

**File**: [contract/test/crowdfunding_test.rs](contract/test/crowdfunding_test.rs)

#### New Tests Added:

1. **test_create_pool_with_minimum_donation**
   - Verifies pool creation with minimum donation setting
   - Confirms minimum_donation is properly stored in PoolConfig

2. **test_contribute_meets_minimum_donation**
   - Tests contribution exactly at minimum amount
   - Verifies successful token transfer

3. **test_contribute_above_minimum_donation**
   - Tests contribution above minimum amount
   - Confirms flexibility in donation amounts

4. **test_contribute_below_minimum_donation_fails**
   - Tests donation below minimum amount
   - Verifies `BelowMinimumDonation` error is returned
   - Key test: Attempts donation of 4 XLM when minimum is 5 XLM

5. **test_pool_with_zero_minimum_accepts_any_amount**
   - Tests pool with zero minimum_donation
   - Confirms backward compatibility for pools without minimum
   - Allows any positive contribution amount

6. **test_save_pool_minimum_exceeds_target_fails**
   - Validates that minimum_donation > target_amount is rejected
   - Returns `InvalidAmount` error

7. **test_save_pool_negative_minimum_fails**
   - Validates that negative minimum_donation is rejected
   - Returns `InvalidAmount` error

#### Test Updates:

- Updated all existing `save_pool` calls to include `minimum_donation` parameter
- Used sensible defaults (0) for pools without specific minimum requirements

## Test Results

✅ All 44 tests pass successfully:

- 4 existing unit tests (types module)
- 40 integration tests (crowdfunding contract)
- Including 7 new minimum donation tests

## Feature Details

### Pool Creation with Minimum

```rust
client.save_pool(
    &name,
    &metadata,
    &creator,
    &target_amount,      // e.g., 1,000,000,000i128 (1000 XLM)
    &deadline,
    &minimum_donation,    // e.g., 5,000,000i128 (5 XLM)
    &None,
    &None,
);
```

### Contribution Validation

- ✅ Contributions >= minimum_donation: **Accepted**
- ❌ Contributions < minimum_donation: **Rejected with BelowMinimumDonation error**
- ✅ minimum_donation = 0: **Allows any positive contribution** (backward compatible)

## Backward Compatibility

- Existing pools can be created with `minimum_donation = 0` (no minimum)
- All contributions to such pools are accepted as before
- No breaking changes to existing pool structures

## Usage Example

### Creating a pool with 5 XLM minimum:

```rust
let minimum_donation = 5_000_000i128; // 5 XLM (7 decimal places)
let pool_id = client.save_pool(
    &String::from_str(&env, "Education Fund"),
    &metadata,
    &creator,
    &100_000_000_000i128, // 100,000 XLM target
    &deadline,
    &minimum_donation,
    &None,
    &None,
)?;
```

### Contributing to the pool:

```rust
// Valid contributions (>= 5 XLM)
client.contribute(&pool_id, &donor, &token, &5_000_000i128)?;  ✅
client.contribute(&pool_id, &donor, &token, &10_000_000i128)?; ✅

// Invalid contribution (< 5 XLM)
client.contribute(&pool_id, &donor, &token, &4_999_999i128)?;  ❌ BelowMinimumDonation
```
