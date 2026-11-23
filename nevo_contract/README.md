# Nevo Smart Contract

## Overview

Nevo is a Soroban-based smart contract that enables decentralized donation pools for educational sponsorships. The contract manages the complete lifecycle of funding pools: creation, donations, student applications, approvals, milestone-based disbursements, and fund claims.

### Key Features

- **Donation Pools**: Create and manage funding pools with specific educational goals
- **Student Applications**: Enable students to apply to pools with application data and milestones
- **School Integration**: Link pools to registered schools for application approval workflows
- **Milestone-Based Disbursements**: Support streamed payments through developer-defined milestones
- **Refund Mechanism**: Donors can request refunds after pool deadlines with grace periods
- **Admin Controls**: Platform administrators set fees, register schools, and oversee operations
- **Event System**: Comprehensive event emission for on-chain activity tracking and indexing

---

## Project Structure

```
nevo_contract/
├── contracts/
│   └── hello-world/
│       ├── src/
│       │   ├── lib.rs                        # Main contract implementation
│       │   ├── test.rs                       # Core functionality tests
│       │   ├── test_issues.rs                # Issue-specific validation tests
│       │   ├── test_auth_bypass.rs           # Authorization and security tests
│       │   ├── test_numeric_overflow.rs      # Numeric overflow edge cases
│       │   ├── test_timestamp_edge_cases.rs  # Deadline and timing edge cases
│       │   └── test_token_transfer_errors.rs # Token operation error handling
│       └── Cargo.toml
├── Cargo.toml                                # Workspace configuration
├── README.md                                 # This file
├── EVENTS_REFERENCE.md                       # Event system documentation
└── EVENT_EMISSION_TESTS.md                   # Event test documentation
```

---

## Data Model

### Core Entities

#### Pool

Represents a donation pool for a specific educational cause.

```rust
pub struct Pool {
    pub sponsor: Address,              // Pool creator/administrator
    pub goal: u128,                    // Target funding amount
    pub collected: u128,               // Current amount raised
    pub is_closed: bool,               // Whether pool has been closed
    pub state: PoolState,              // Current state (Active, Paused, Completed, etc.)
    pub application_deadline: u64,     // Timestamp deadline for student applications
}
```

**Pool States:**
- `Active`: Pool is accepting donations and applications
- `Paused`: Pool temporarily halted
- `Completed`: Goal has been reached
- `Cancelled`: Pool was cancelled
- `Disbursed`: Funds have been distributed to students
- `Closed`: Pool is permanently closed

`close_pool` requires sponsor authorization and accepts `Disbursed` or
`Cancelled` pools. Creation enters `Active`, and an accounting donation that
meets the goal enters `Completed`. Claims update disbursement accounting while
keeping the current state. Only the mock `set_pool_state` currently assigns
`Disbursed` or `Cancelled`. A production workflow still needs authorized
transitions with settlement and cancellation rules before the sponsor can
close a pool through that workflow.

Student disbursements count toward a pool-wide claim total. A successful
claim makes the pool ineligible for donor refunds. For an existing pool with
no aggregate, the contract can recover earlier claims from indexed
applications. Historical claims for students without an indexed application
require reconciliation from on-chain claim events before an upgrade.

#### Application

Tracks a student's approved funding, disbursement progress and approval status.

```rust
pub struct Application {
    pub approved_amount: i128,        // Total amount approved for the student
    pub amount_claimed: i128,         // Amount already withdrawn by the student
    pub status: ApplicationStatus,    // Pending | Approved | Rejected
}
```

#### ApplicationStatus

```rust
#[contracttype]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
}
```

The status is a typed enum, so invalid values (e.g. `"approved"`) cannot be
represented. It is stored only inside `Application`; there is no separate
status storage key.

**State machine:**

| From | To | Allowed when |
|------|----|--------------|
| `Pending` | `Approved` | Linked school (or admin) approves |
| `Pending` | `Rejected` | Linked school (or admin) rejects |
| `Approved` | `Rejected` | `amount_claimed == 0` (revocation) |
| `Rejected` | `Approved` | Linked school (or admin) re-approves |
| `Approved` | `Approved` | No-op / idempotent re-approval |
| `Rejected` | `Rejected` | No-op / idempotent re-rejection |

Any other transition fails with a typed error. In particular, an application
with `amount_claimed > 0` can never be rejected or revoked, so a partially
claimed student's remaining allocation stays locked. Approve, reject and
revoke each emit a distinct event.

#### Milestone

Represents a payment milestone for streamed disbursements.

```rust
pub struct Milestone {
    pub amount: u128,  // Amount for this milestone
}
```

### Storage Key Namespacing

The contract uses a hierarchical key system to organize persistent storage:

| Key Prefix | Purpose | Example |
|-----------|---------|---------|
| `pool_count` | Total number of pools created | Counter for generating pool IDs |
| `p` | Legacy pool data (replaced by direct ID indexing) | Compatibility layer |
| `metadata` | Pool title and description | `("metadata", pool_id)` |
| `pool_school` | School linked to a pool | `("pool_school", pool_id)` |
| `a_count_*` | Application count for a pool | `("a_count_", pool_id)` |
| `a_*` | Individual applications | `("a_", pool_id, app_id)` |
| `ap_*` | Application tracking by student | `("ap_", pool_id, student)` |
| `claimed_amount` | Claimed funds tracking | `("claimed_amount", pool_id, student)` |
| `milestones` | Student milestones | `("milestones", pool_id, student)` |
| `pool_deadline` | Refund deadline (ledger sequence) | `("pool_deadline", pool_id)` |
| `school_reg` | Registered school mapping | `("school_reg", school_address)` |
| `admin` | Platform admin address | "admin" |
| `unclaimed_fees` | Accumulated protocol fees | "unclaimed_fees" |
| `creation_fee` | Pool creation fee | "creation_fee" |

### Storage Tiers and TTL Strategy

Soroban persistent storage entries expire after their time-to-live (TTL) elapses
and must be restored before they can be read again. To keep long-running pools
usable without manual restore operations, the contract splits state between
**instance** and **persistent** storage and actively extends TTLs.

#### Instance storage (contract-wide config)

Global configuration is stored in instance storage, which shares a single TTL
with the contract instance:

- `admin` — platform administrator address
- `creation_fee` — pool creation fee
- `crowdfunding_token` — token used for crowdfunding donations
- `pool_count` — total number of pools created (pool ID counter)

The instance TTL is extended on every write to these values, so the contract
instance stays alive as long as it is being used.

#### Persistent storage (per-entity data)

Entity data lives in persistent storage and has its own TTL per key:

- Pools (`pool`, `metadata`, `pool_school`, `pool_deadline`, ...)
- Contributions and donor markers (`contribution`, `(pool_id, "donor", donor)`)
- Applications (`a_*`, `ap_*`, `app_status`, `milestones`, `claimed_amount`)
- School registrations (`school_reg`) and accumulated fees (`unclaimed_fees`)

#### Named TTL constants

| Constant | Value (ledgers) | Purpose |
|----------|-----------------|---------|
| `PERSISTENT_TTL_THRESHOLD` | 100 | Extend persistent entries when their remaining TTL drops below this |
| `PERSISTENT_TTL_EXTEND` | 518_400 | Target TTL applied to persistent entries (~30 days at 5s/ledger) |
| `INSTANCE_TTL_THRESHOLD` | 100 | Extend the instance when its remaining TTL drops below this |
| `INSTANCE_TTL_EXTEND` | 518_400 | Target TTL applied to the instance (~30 days at 5s/ledger) |

#### Extension on read and write

Every read and write of a pool, contribution, application, or donor marker
calls `extend_ttl` with the persistent threshold/extend constants. Writes to
global config extend the instance TTL. As a result, active pools, contributions,
and applications stay readable without a manual restore operation; only entries
that have been completely idle for longer than the extend window need restoring.

---

## Refund Behaviour

`refund_donation` lowers `pool.collected` and then **recomputes the pool state**
so it stays consistent with the collected amount:

- If the pool was `Completed` and `collected` drops below `goal`, the state
  returns to `Active`.
- If `collected` still meets or exceeds `goal`, the state remains `Completed`.

On a **full refund** (the donor's contribution reaches zero), the donor marker
`(pool_id, "donor", donor)` is cleared and the pool's donor count is decremented,
so a later re-donation by the same address counts as a new donor.

---

## Contract Functions

### Admin Operations

#### `set_admin(env, admin)`
Sets the platform administrator address.
- **Authorization**: Requires admin address to sign
- **Events**: Emits `ADMIN_SET` event

#### `register_school(env, admin, school)`
Registers a school for pool linkage and application approvals.
- **Authorization**: Requires admin to sign
- **Errors**: `UnauthorizedAdmin`, `AdminNotSet`
- **Events**: Emits `SCHOOL_REG` event

#### `is_school_registered(env, school) -> bool`
Checks if a school is registered in the system.

---

### Pool Management

#### `create_pool(env, creator, title, description, goal, application_deadline) -> u32`
Creates a new donation pool.
- **Returns**: Pool ID
- **Constraints**: Description must be ≤ 500 characters; `application_deadline` is a ledger sequence
- **Events**: Emits `POOL_CREATED` event with creator, goal, title, and description

#### `create_pool_for_school(env, creator, title, description, goal, school, application_deadline) -> u32`
Creates a pool linked to a registered school.
- **Authorization**: Requires creator to sign
- **Errors**: `SchoolNotRegistered`
- **Events**: Emits `POOL_CREATED` event and links school

#### `set_pool_deadline(env, pool_id, deadline)`
Sets the explicit refund deadline for a pool.
- **Authorization**: Requires pool sponsor to sign
- **Constraints**: `deadline >= application_deadline` (ledger sequences); contradictory deadlines are rejected
- **Events**: Emits `ddln_set` event

#### `get_pool(env, pool_id) -> (u32, Address, u128, u128, bool, u64)`
Retrieves pool information as a tuple.
- **Returns**: (pool_id, sponsor, goal, collected, is_closed, deadline)
- **Errors**: `PoolNotFound`

#### `get_pool_metadata(env, pool_id) -> (String, String)`
Retrieves pool title and description.

#### `get_pool_school(env, pool_id) -> Address`
Retrieves the school linked to a pool.

#### `get_pool_count(env) -> u32`
Returns the total number of pools created.

#### `get_total_raised(env, pool_id) -> u128`
Returns the total amount donated to a pool.

#### Read Getters

The following read-only getters expose pool and protocol state so clients can
render it without guessing or reconstructing values from events.

##### `get_pool_token(env, pool_id) -> Address`
Returns the token address configured for a pool via `set_pool_token`.
- **Returns**: The pool's token `Address`
- **Errors**: `PoolNotFound` if the pool does not exist

##### `get_pool_state(env, pool_id) -> PoolState`
Returns the current `PoolState` of a pool (`Active`, `Paused`, `Completed`,
`Cancelled`, `Disbursed`, or `Closed`). This value is not included in
`get_pool`.
- **Returns**: The pool's `PoolState`
- **Errors**: `PoolNotFound` if the pool does not exist

##### `get_unclaimed_fees(env) -> i128`
Returns the accumulated protocol fees that have not yet been claimed.
- **Returns**: The unclaimed fee balance as `i128` (0 when none accrued)

##### `get_emergency_withdrawal(env, pool_id) -> EmergencyWithdrawalRequest`
Returns the pending emergency withdrawal request for a pool.
- **Returns**: The `EmergencyWithdrawalRequest` for the pool
- **Errors**: `PoolNotFound` if the pool does not exist;
  `EmergencyWithdrawalNotRequested` if no emergency withdrawal is pending

---
=======

### Donation Operations

#### `donate(env, pool_id, donor, amount)`
Donates to a pool.
- **Constraints**: Pool must be Active, not Closed
- **Tracking**: Tracks individual donor contributions
- **Events**: Emits `DONATION_MADE` event with donor, amount, and new collected total

#### `get_donor_count(env, pool_id) -> u32`
Returns the number of unique donors for a pool.

#### `get_contribution(env, pool_id, donor) -> u128`
Returns the total contribution amount for a specific donor in a pool.

---

### Application Management

#### `apply_to_pool(env, pool_id, student, application_data)`
Allows a student to apply to a pool.
- **Authorization**: Requires student to sign
- **Constraints**: Student can only apply once per pool
- **Status**: Sets initial status to `ApplicationStatus::Pending`
- **Errors**: `PoolNotFound`, `DuplicateApplication`
- **Events**: Emits `APPLICATION_SUBMITTED` event with student and app count

#### `approve_application(env, pool_id, school, student, approved)`
Allows the linked school (or the platform admin) to approve or reject an application.
- **Authorization**: Requires the linked school or admin to sign
- **Transitions**: Enforced per the state machine above; rejecting an application with `amount_claimed > 0` fails
- **Errors**: `OnlyLinkedSchoolCanApprove`, `StudentHasNotApplied`, invalid-transition error
- **Events**: Emits distinct approve, reject and revoke events with student and new status

#### `get_application_status(env, pool_id, student) -> ApplicationStatus`
Returns the typed status of a student's application (`Pending`, `Approved` or `Rejected`).
- **Errors**: `StudentHasNotApplied`

---

### Milestone & Disbursement Operations

#### `setup_application_milestones(env, pool_id, student, milestones: Vec<Milestone>)`
Sets up payment milestones for an approved student.
- **Authorization**: Requires the linked school (or the platform admin) to sign — the student cannot configure their own schedule
- **Constraints**: Sum of milestone amounts must equal pool goal
- **Errors**: `PoolNotFound`, `OnlyLinkedSchoolCanApprove`
- **Events**: Emits `MILESTONES_SET` event

#### `get_milestones(env, pool_id, student) -> Vec<Milestone>`
Retrieves the milestones for a student.

#### `claim_funds(env, student, pool_id, claim_amount, token_address)`
Allows an approved student to claim funds in installments.
- **Authorization**: Requires student to sign
- **Constraints**: Can only claim up to approved_amount - amount_claimed
- **Events**: Emits `FUNDS_CLAIMED` event
- **Returns**: New claimed total

#### `get_claimed_amount(env, pool_id, student) -> i128`
Returns the total amount a student has claimed from a pool.

#### `get_application(env, pool_id, student) -> Option<Application>`
Returns the full Application record with approved and claimed amounts.

---

### Advanced Operations

#### `withdraw_unallocated_funds(env, pool_id, token_address)`
Allows pool sponsor to withdraw surplus funds not locked by active applications.
- **Authorization**: Requires pool sponsor to sign
- **Calculation**: Surplus = collected - sum(approved_amount - amount_claimed for Approved/Pending apps)
- **Constraints**: Pool must exist, surplus must be > 0
- **Events**: Updates pool state

#### `set_application_status(env, pool_id, student, status: ApplicationStatus)`
Sets the approval status for a student application (internal function). Accepts
only the typed `ApplicationStatus` enum and enforces the state machine above.

---

## Error Handling

All contract errors are defined in `ContractError` enum with numeric codes:

| Code | Error | Description |
|------|-------|-------------|
| 1 | `PoolNotFound` | Pool with given ID does not exist |
| 2 | `InvalidPoolState` | Pool is not in the required state for operation |
| 3 | `UnauthorizedAdmin` | Caller is not the platform admin |
| 4 | `PoolIsClosed` | Operation rejected because pool is closed |
| 5 | `DuplicateApplication` | Student already applied to this pool |
| 6 | `StudentHasNotApplied` | Student has not applied to this pool |
| 7 | `OnlyLinkedSchoolCanApprove` | Only linked school can approve applications |
| 8 | `PoolNotDisbursedOrRefunded` | Pool not in required state for closure |
| 9 | `AdminNotSet` | No admin has been configured |
| 10 | `NoUnclaimedFees` | No accumulated protocol fees |
| 11 | `InvalidFee` | Fee value is invalid |
| 12 | `PoolNotExpired` | Pool deadline has not passed |
| 13 | `NoContributionToRefund` | Donor has no contribution to refund |
| 14 | `SchoolNotRegistered` | School is not registered |

---

## Event System

The contract emits events for all state-changing operations, enabling real-time tracking and indexing.

### Event Topics

| Event | Symbol | Emitted By |
|-------|--------|-----------|
| Pool Created | `pool_crtd` | `create_pool()` |
| Donation Made | `donation` | `donate()` |
| Contribution | `contrib` | (token-based donations) |
| Pool Closed | `pool_cls` | `close_pool()` |
| Application Submitted | `app_sub` | `apply_to_pool()` |
| Application Approved | `app_aprvd` | `approve_application()` |
| Milestones Set | `mile_set` | `setup_application_milestones()` |
| Funds Claimed | `fund_clmd` | `claim_funds()` |
| Fees Claimed | `fees_clmd` | (fee claiming operations) |
| Donation Refund | `don_refnd` | (refund operations) |
| Deadline Set | `ddln_set` | (deadline setting) |
| Pool State Set | `pool_stat` | (state changes) |
| School Registered | `schl_reg` | `register_school()` |
| Admin Set | `admin_set` | `set_admin()` |
| Fee Updated | `fee_upd` | (fee updates) |

For detailed event documentation, see [EVENTS_REFERENCE.md](./EVENTS_REFERENCE.md).

---

## Building and Testing

### Prerequisites

- Rust toolchain with Soroban support
- Soroban CLI
- `cargo` package manager

### Build

```bash
cd nevo_contract
cargo build --release --target wasm32-unknown-unknown
```

### Run Tests

#### All Tests
```bash
cargo test
```

#### Specific Test Suite
```bash
cargo test --test test_issues
cargo test --test test_auth_bypass
cargo test --test test_numeric_overflow
cargo test --test test_timestamp_edge_cases
cargo test --test test_token_transfer_errors
```

#### With Output
```bash
cargo test -- --nocapture
```

---

## Test Files

The contract includes comprehensive test coverage organized by concern:

### `test.rs` - Core Functionality Tests
Tests the main contract functionality:
- Pool creation and retrieval
- Donation operations
- Student applications and approvals
- Milestone setup and management
- Fund claiming and disbursements
- Donor tracking and contributions

### `test_issues.rs` - Issue-Specific Validation Tests
Tests for specific GitHub issues and requirements:
- Emergency withdrawal grace period validation (Issue #460)
- Numeric overflow prevention
- Deadline validation edge cases
- State transition validation
- Contribution tracking accuracy

### `test_auth_bypass.rs` - Authorization and Security Tests
Tests that security mechanisms are properly enforced:
- Admin-only function protection
- Authentication requirement validation
- Cross-user authorization prevention
- Mock auth bypass prevention
- Role-based access control

### `test_numeric_overflow.rs` - Numeric Overflow Edge Cases
Tests arithmetic overflow handling:
- Addition overflow in fund collection
- Subtraction underflow in refunds
- Multiplication overflow in calculations
- Safe arithmetic operation verification
- Boundary condition handling

### `test_timestamp_edge_cases.rs` - Deadline and Timing Edge Cases
Tests timestamp and deadline logic:
- Grace period boundary conditions
- Deadline validation (future vs. past)
- Refund eligibility timing
- Deadline overflow prevention
- Deterministic timestamp handling in tests

### `test_token_transfer_errors.rs` - Token Operation Error Handling
Tests token transfer failure scenarios:
- Insufficient balance handling
- Invalid token contract rejection
- Failed transfer state preservation
- Partial transfer prevention
- Token operation error recovery

---

## Deployment

### Deploy to Testnet

```bash
# Set up environment
export SOROBAN_ACCOUNT=<your-account>
export SOROBAN_RPC_URL=https://soroban-testnet.stellar.org

# Deploy contract
soroban contract deploy \
  --wasm nevo_contract/target/wasm32-unknown-unknown/release/nevo_contract.wasm \
  --source-account $SOROBAN_ACCOUNT
```

### Initialize Contract

After deployment, initialize the admin:

```bash
soroban contract invoke \
  --id <contract-id> \
  --source-account $SOROBAN_ACCOUNT \
  -- set_admin --admin <admin-address>
```

---

## Common Workflows

### Creating a Pool for a School

1. **Register School** (Admin)
   ```rust
   register_school(env, admin, school_address);
   ```

2. **Create Pool** (Pool Creator)
   ```rust
   let pool_id = create_pool_for_school(
       env,
       creator,
       "STEM Scholarship Fund",
       "Support for underprivileged STEM students",
       100_000_000,  // 100 XLM in stroops
       school_address,
       deadline_timestamp
   );
   ```

3. **Track Donations** (Donors)
   ```rust
   donate(env, pool_id, donor, 5_000_000);
   ```

### Student Application and Funding Flow

1. **Student Applies**
   ```rust
   apply_to_pool(env, pool_id, student, application_data);
   ```

2. **School Approves**
   ```rust
   approve_application(env, pool_id, school, student, true);
   ```

3. **Set Up Milestones** (Linked School or Admin)
   ```rust
   let milestones = Vec::from_array(&env, [
       Milestone { amount: 25_000_000 },
       Milestone { amount: 25_000_000 },
       Milestone { amount: 25_000_000 },
       Milestone { amount: 25_000_000 },
   ]);
   setup_application_milestones(env, pool_id, student, milestones);
   ```

4. **Claim Funds** (Student, in installments)
   ```rust
   claim_funds(env, student, pool_id, 25_000_000, token_address);
   ```

---

## Configuration Constants

Key configuration values defined in the contract:

| Constant | Value | Purpose |
|----------|-------|---------|
| `MAX_DESCRIPTION_LENGTH` | 500 | Max characters in pool description |
| `MAX_URL_LENGTH` | 256 | Max characters for URL fields |
| `MAX_IMAGE_HASH_LENGTH` | 64 | Max characters for image hash |
| `GRACE_PERIOD_SECS` | 86400 | 24-hour grace period (legacy, seconds) |
| `REFUND_GRACE_PERIOD_LEDGERS` | 17280 | ~24 hours in ledger blocks; grace period applied to the refund window |

---

## Design Note: Unified Deadline Model

All pool deadlines use a **single time base: the ledger sequence**. The
`application_deadline` set at pool creation is a ledger sequence that gates
donations and applications, and the refund deadline (`pool_deadline`) is also a
ledger sequence.

- **No contradictory deadlines**: `set_pool_deadline` rejects any value with
  `pool_deadline < application_deadline`.
- **Every pool has a reachable refund window**: when no explicit refund
  deadline is set, the refund window opens at
  `application_deadline + REFUND_GRACE_PERIOD_LEDGERS`. A pool is therefore
  always refundable after its funding deadline plus the grace period, and the
  `deadline == 0` → `PoolNotExpired` dead end no longer exists.
- **Getters and events** report the effective refund deadline (explicit value
  if set, otherwise the derived one).
| `GRACE_PERIOD_SECS` | 86400 | 24-hour grace period for refunds |
| `REFUND_GRACE_PERIOD_LEDGERS` | 17280 | ~24 hours in ledger blocks |
| `PERSISTENT_TTL_THRESHOLD` | 100 | Min remaining TTL before extending persistent entries |
| `PERSISTENT_TTL_EXTEND` | 518400 | Target TTL for persistent entries (~30 days) |
| `INSTANCE_TTL_THRESHOLD` | 100 | Min remaining TTL before extending the instance |
| `INSTANCE_TTL_EXTEND` | 518400 | Target TTL for the instance (~30 days) |

---

## Future Enhancements

Potential improvements tracked in GitHub issues:

- [ ] Dynamic fee configuration
- [ ] Batch donation processing
- [ ] Advanced milestone conditions
- [ ] Integration with external data feeds
- [ ] Multi-signature pool operations
- [ ] Pool transfer functionality
- [ ] Dispute resolution mechanisms

---

## Security Considerations

### Authorization

- All state-changing operations require the appropriate party to sign
- Admin functions are protected by the `set_admin` mechanism
- School-only functions verify school registration

### Arithmetic Safety

- All arithmetic operations use checked methods to prevent overflow
- Numeric tests verify edge cases and boundary conditions

### Deadline Validation

- All deadlines use the ledger sequence as the single time base
- `pool_deadline >= application_deadline` is enforced; contradictory deadlines are rejected
- Pools without an explicit refund deadline become refundable at `application_deadline + REFUND_GRACE_PERIOD_LEDGERS`
- Grace periods prevent manipulation of refund eligibility
- Deterministic ledger helpers used in testing

### Token Handling

- Token transfers are properly validated
- Failed transfers don't corrupt contract state
- Insufficient balance scenarios are handled safely

### Storage TTL

- Global config lives in instance storage; entity data in persistent storage
- TTLs are extended on read and write using the named TTL constants
- Active pools, contributions, and applications stay readable without manual restore
- Tests advance the ledger past the default TTL to verify entries remain readable

---

## Support and Documentation

- **Event System**: See [EVENTS_REFERENCE.md](./EVENTS_REFERENCE.md) for complete event documentation
- **Event Tests**: See [EVENT_EMISSION_TESTS.md](./EVENT_EMISSION_TESTS.md) for event test details
- **Soroban Docs**: [Soroban Documentation](https://soroban.stellar.org/docs)
- **Issue Tracking**: Check GitHub issues for known limitations and future work

---

## License

See the root repository LICENSE file for licensing information.
