# Nevo Contract Error & Panic Catalogue

This document provides a comprehensive reference of all error conditions and panic messages in the Nevo smart contract. It's intended for off-chain integrators (frontend, server, indexers) to understand what can go wrong and how to handle it.

## Overview

The Nevo contract uses two error mechanisms:

1. **Typed Errors** (`#[contracterror]` enum): Machine-readable error codes that are XDR-encoded and stable across contract versions
2. **String Panics**: Ad-hoc panic messages (legacy, not recommended for new code)

For robust integrations, always handle typed errors first, then add fallbacks for string panic messages.

---

## Typed Errors

These errors are encoded as numeric codes and returned via `env.panic_with_error()`. They are the recommended way to signal error conditions.

### Error Enum Reference

| Code | Variant | HTTP Analogy | Description |
|------|---------|--------------|-------------|
| 1 | `PoolNotFound` | 404 | Attempted operation on a non-existent pool ID |
| 2 | `InvalidPoolState` | 422 | Operation not allowed in current pool state (e.g., donations to non-Active pool) |
| 3 | `UnauthorizedAdmin` | 403 | Caller is not the registered platform administrator |
| 4 | `PoolIsClosed` | 410 | Operation rejected because pool is permanently closed |
| 5 | `DuplicateApplication` | 409 | Student attempted to apply twice to the same pool |
| 6 | `StudentHasNotApplied` | 404 | Operation requires existing application, but student has not applied |
| 7 | `OnlyLinkedSchoolCanApprove` | 403 | Non-linked school attempted to approve applications for this pool |
| 8 | `PoolNotDisbursedOrRefunded` | 422 | Pool must reach Disbursed or Cancelled state before closing |
| 9 | `AdminNotSet` | 503 | No admin has been configured; call `set_admin()` first |
| 10 | `NoUnclaimedFees` | 404 | No accumulated protocol fees available to claim |
| 11 | `InvalidFee` | 400 | Fee value is invalid (negative or invalid type) |
| 12 | `PoolNotExpired` | 422 | Pool deadline has not passed or grace period hasn't elapsed |
| 13 | `NoContributionToRefund` | 404 | Donor has no recorded contribution to refund |
| 14 | `SchoolNotRegistered` | 404 | School address is not registered in the system |
| 15 | `PoolAlreadyClosed` | 409 | Attempted to close a pool that is already closed |

### Functions by Error

#### `PoolNotFound` (Code 1)

**Raised by:**
- `get_pool()` - Pool ID doesn't exist
- `get_total_raised()` - Pool ID doesn't exist
- `close_pool()` - Pool ID doesn't exist
- `donate()` - Pool ID doesn't exist
- `apply_to_pool()` - Pool ID doesn't exist
- `get_pool_school()` - Pool ID doesn't exist
- `setup_application_milestones()` - Pool ID doesn't exist
- `claim_funds()` - Pool ID doesn't exist
- `withdraw_unallocated_funds()` - Pool ID doesn't exist
- `request_emergency_withdrawal()` - Pool ID doesn't exist
- `confirm_emergency_withdrawal()` - Pool ID doesn't exist
- `set_pool_state()` - Pool ID doesn't exist

**Meaning:** The pool ID referenced does not exist in contract storage. Verify the pool ID is correct and the pool has been created.

**Frontend Handling:**
```javascript
try {
  pool = await contract.getPool(poolId);
} catch (e) {
  if (e.errorCode === 1) {
    showError(`Pool #${poolId} does not exist`);
  }
}
```

---

#### `InvalidPoolState` (Code 2)

**Raised by:**
- `donate()` - Pool state is not `Active`
- `request_emergency_withdrawal()` - Pool state is not `Active`

**Meaning:** The operation cannot proceed because the pool is in an incompatible state. Donations and emergency withdrawals only work on `Active` pools.

**Pool States:** `Active`, `Paused`, `Completed`, `Cancelled`, `Disbursed`, `Closed`

**Frontend Handling:**
```javascript
try {
  await contract.donate(poolId, donor, amount);
} catch (e) {
  if (e.errorCode === 2) {
    const pool = await contract.getPool(poolId);
    showError(`Cannot donate to pool in ${pool.state} state`);
  }
}
```

---

#### `UnauthorizedAdmin` (Code 3)

**Raised by:**
- `register_school()` - Caller is not the admin address
- `claim_protocol_fees()` - Caller is not the admin address
- `set_creation_fee()` - Caller is not the admin address
- `request_refund()` - Caller is not the admin (when called by admin)
- `admin_confirm_emergency_withdrawal()` - Caller is not the admin

**Meaning:** The operation requires administrator authorization, but the caller is not registered as the platform admin. Only the address set via `set_admin()` can perform admin-only operations.

**Frontend Handling:**
```javascript
try {
  await contract.registerSchool(admin, schoolAddr);
} catch (e) {
  if (e.errorCode === 3) {
    showError("Only the platform administrator can perform this action");
  }
}
```

---

#### `PoolIsClosed` (Code 4)

**Raised by:**
- `donate()` - Pool's `is_closed` flag is true
- `request_emergency_withdrawal()` - Pool's `is_closed` flag is true

**Meaning:** The pool has been permanently closed and no further operations (donations, withdrawals, etc.) are allowed. Check with the pool creator if this was intentional.

**Frontend Handling:**
```javascript
try {
  await contract.donate(poolId, donor, amount);
} catch (e) {
  if (e.errorCode === 4) {
    showError("This pool has been closed and is no longer accepting donations");
  }
}
```

---

#### `DuplicateApplication` (Code 5)

**Raised by:**
- `apply_to_pool()` - Student has already applied to this pool

**Meaning:** A student attempted to submit a second application to the same pool. Each student can only have one application per pool. If they want to modify their application, they must request a withdrawal or start a new pool cycle.

**Frontend Handling:**
```javascript
try {
  await contract.applyToPool(poolId, student, appData);
} catch (e) {
  if (e.errorCode === 5) {
    const status = await contract.getApplicationStatus(poolId, student);
    showError(`You already applied to this pool (status: ${status})`);
  }
}
```

---

#### `StudentHasNotApplied` (Code 6)

**Raised by:**
- `approve_application()` - Student has no application record for this pool
- `setup_application_milestones()` - Student has no application record for this pool

**Meaning:** Attempted to approve or set milestones for a student who has not applied to this pool. Verify the student address is correct or have the student apply first.

**Frontend Handling:**
```javascript
try {
  await contract.approveApplication(poolId, school, student, approved);
} catch (e) {
  if (e.errorCode === 6) {
    showError(`Student has not applied to this pool`);
  }
}
```

---

#### `OnlyLinkedSchoolCanApprove` (Code 7)

**Raised by:**
- `approve_application()` - Caller school is not linked to this pool

**Meaning:** Only the school explicitly linked to a pool can approve applications for it. Other schools cannot approve students even if they are registered.

**Frontend Handling:**
```javascript
try {
  await contract.approveApplication(poolId, school, student, true);
} catch (e) {
  if (e.errorCode === 7) {
    const linkedSchool = await contract.getPoolSchool(poolId);
    showError(`Only ${linkedSchool} can approve applications for this pool`);
  }
}
```

---

#### `PoolNotDisbursedOrRefunded` (Code 8)

**Raised by:**
- `close_pool()` - Pool state is not `Disbursed` or `Cancelled`

**Meaning:** A pool can only be closed after funds have been disbursed to students (state = `Disbursed`) or the pool has been cancelled (state = `Cancelled`). The pool must reach one of these terminal states before closure.

**Frontend Handling:**
```javascript
try {
  await contract.closePool(poolId);
} catch (e) {
  if (e.errorCode === 8) {
    showError("Pool must be in Disbursed or Cancelled state before closing");
  }
}
```

---

#### `AdminNotSet` (Code 9)

**Raised by:**
- `register_school()` - Admin address has not been set
- `claim_protocol_fees()` - Admin address has not been set
- `set_creation_fee()` - Admin address has not been set
- `set_pool_state()` - Admin address has not been set

**Meaning:** The contract has not been initialized with an admin address. Call `set_admin(admin_address)` first to configure the platform administrator.

**Frontend Handling:**
```javascript
try {
  await contract.registerSchool(admin, schoolAddr);
} catch (e) {
  if (e.errorCode === 9) {
    showError("Platform not initialized. Admin must call set_admin() first");
  }
}
```

---

#### `NoUnclaimedFees` (Code 10)

**Raised by:**
- `claim_protocol_fees()` - No accumulated fees in storage

**Meaning:** There are no protocol fees available to claim. Fees accumulate only when students claim funds. If fees are expected but this error occurs, fees may have already been claimed or not yet accumulated.

**Frontend Handling:**
```javascript
try {
  await contract.claimProtocolFees(admin);
} catch (e) {
  if (e.errorCode === 10) {
    showError("No protocol fees have been accumulated yet");
  }
}
```

---

#### `InvalidFee` (Code 11)

**Raised by:**
- `set_creation_fee()` - Fee value is negative

**Meaning:** The fee value provided is invalid. Fees must be non-negative integers.

**Frontend Handling:**
```javascript
try {
  await contract.setCreationFee(admin, feeAmount);
} catch (e) {
  if (e.errorCode === 11) {
    showError("Fee must be a non-negative number");
  }
}
```

---

#### `PoolNotExpired` (Code 12)

**Raised by:**
- `request_refund()` - Pool deadline hasn't passed or grace period hasn't elapsed

**Meaning:** Refund requests can only be made after the pool's application deadline has passed AND a grace period has elapsed (24 hours). This prevents premature refunds while the pool is still active.

**Frontend Handling:**
```javascript
try {
  await contract.requestRefund(poolId, donor);
} catch (e) {
  if (e.errorCode === 12) {
    const pool = await contract.getPool(poolId);
    const deadline = new Date(pool.deadline * 1000);
    showError(`Cannot refund yet. Pool deadline: ${deadline.toISOString()}`);
  }
}
```

---

#### `NoContributionToRefund` (Code 13)

**Raised by:**
- `request_refund()` - Donor has no recorded contribution to this pool

**Meaning:** The donor address has no contribution record for this pool, so there's nothing to refund. Verify the donor address is correct or that a donation was actually made.

**Frontend Handling:**
```javascript
try {
  await contract.requestRefund(poolId, donor);
} catch (e) {
  if (e.errorCode === 13) {
    showError("You have no contribution to refund from this pool");
  }
}
```

---

#### `SchoolNotRegistered` (Code 14)

**Raised by:**
- `create_pool_for_school()` - School address is not registered

**Meaning:** The school address provided has not been registered by the platform admin. Only registered schools can be linked to pools. Have the admin call `register_school(school_address)` first.

**Frontend Handling:**
```javascript
try {
  await contract.createPoolForSchool(creator, title, desc, goal, school, deadline);
} catch (e) {
  if (e.errorCode === 14) {
    showError("This school is not registered with the platform");
  }
}
```

---

#### `PoolAlreadyClosed` (Code 15)

**Raised by:**
- `close_pool()` - Pool has already been closed

**Meaning:** The pool's `is_closed` flag is already set, so it cannot be closed a second time.

---

## String Panic Messages

These are ad-hoc panic messages (not part of the `#[contracterror]` enum). While they should be replaced with typed errors over time, integrators should be aware of them as they may be encountered in production.

### Error Catalogue

| Panic Message | Function | Cause | Resolution |
|---------------|----------|-------|-----------|
| `"Description exceeds maximum length"` | `create_pool()` | Pool description > 500 characters | Shorten description to ≤ 500 characters |
| `"Milestones required"` | `setup_application_milestones()` | Empty milestones vector provided | Provide at least one milestone |
| `"Milestone total must equal pool goal"` | `setup_application_milestones()` | Sum of milestone amounts ≠ pool goal | Ensure milestone amounts sum to exactly pool goal |
| `"No surplus to withdraw"` | `withdraw_unallocated_funds()` | No unallocated funds (surplus = 0) | Check pool has unallocated funds before withdrawing |
| `"Claim amount must be positive"` | `claim_funds()` | Claim amount ≤ 0 | Use a positive claim amount |
| `"Application status not found"` | `claim_funds()` | Student has no application status record | Student must apply to pool first |
| `"Application is not approved"` | `claim_funds()` | Application status is not "Approved" | School must approve application before student can claim |
| `"Overdraw attempt"` | `claim_funds()` | `amount_claimed + claim_amount > collected` | Cannot claim more than available collected funds |
| `"Deadline must be in the future"` | `set_pool_deadline()` | Deadline ≤ current ledger sequence | Use a future timestamp |
| `"InvalidAmount"` | `set_pool_state()` | Amount argument is ≤ 0 | Provide positive amount |
| `"Error(Auth, InvalidAction)"` | `admin_confirm_emergency_withdrawal()` | Caller is not the admin (legacy error) | Only admin can perform this action |
| `"EmergencyWithdrawalAlreadyRequested"` | `request_emergency_withdrawal()` | Emergency withdrawal already pending | Complete or cancel existing request first |
| `"Grace period not elapsed"` | `confirm_emergency_withdrawal()` | Less than 24 hours have elapsed since request | Wait for grace period to complete |
| `"Insolvency: locked funds exceed collected"` | `withdraw_unallocated_funds()` | Pool accounting is inconsistent | This should never happen; report as bug |

---

## Error Handling Best Practices

### Frontend Integration

1. **Distinguish Error Types**
   ```javascript
   async function handleContractError(error, context) {
     if (error.errorCode) {
       // Typed error - machine-readable and stable
       handleTypedError(error.errorCode, context);
     } else if (error.message.includes("Description exceeds")) {
       // String panic - handle as fallback
       handleStringPanic(error.message);
     } else {
       // Unknown error
       showGenericError(error);
     }
   }
   ```

2. **Implement Retry Logic**
   ```javascript
   // Typed errors usually indicate permanent failures, don't retry
   if ([1, 5, 6, 7].includes(error.errorCode)) {
     return fail(error);
   }
   
   // Some errors may be transient (server overload, ledger sync)
   if ([2, 12].includes(error.errorCode)) {
     return retry(delay);
   }
   ```

3. **User-Friendly Messages**
   ```javascript
   const userMessages = {
     1: "This pool doesn't exist. Check the pool ID.",
     3: "You don't have permission to do this.",
     5: "You've already applied to this pool.",
     12: "It's too early to request a refund. Please wait.",
   };
   ```

### Server/Backend Integration

1. **Log with Context**
   ```rust
   error!(
     error_code = error_code,
     pool_id = pool_id,
     user = user_address,
     operation = "donate",
     "Contract operation failed"
   );
   ```

2. **Implement Monitoring**
   ```rust
   metrics::counter!(
     "contract_error",
     "error_code" => error_code.to_string()
   ).increment(1);
   ```

3. **Handle Partial Failures**
   Some operations may fail mid-transaction. Always check post-condition state:
   ```javascript
   await contract.donate(poolId, donor, amount);
   const newTotal = await contract.getTotalRaised(poolId);
   if (!newTotal.includes(previousTotal + amount)) {
     // Donation failed despite no explicit error
     // Retry or investigate
   }
   ```

---

## Related Documentation

- [Contract README](./README.md) - Main contract documentation
- [EVENTS_REFERENCE.md](./EVENTS_REFERENCE.md) - Events emitted by operations
- [PoolState Enum](./README.md#data-model) - Pool state machine documentation
- [Soroban SDK Docs](https://soroban.stellar.org/docs) - Official Soroban documentation

---

## Updating This Catalogue

When adding new functions or modifying error handling:

1. **Add to typed errors** if it's a recoverable, well-defined condition
2. **Document in this file** immediately (before merging)
3. **Include error codes** in contract comments
4. **Update integration tests** to verify error codes

See the Contributing Guide for details.

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | July 29, 2026 | Initial error catalogue creation |

