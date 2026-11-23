//! Nevo donation pool contract.
//!
//! Sponsors create donation pools with a funding goal and application
//! deadline; donors contribute funds (native or via `donate_with_token`)
//! that accumulate toward the pool's goal. Schools register with the
//! platform admin and can be linked to pools so that students may apply
//! for support. Approved applications are funded via milestone-based
//! disbursements: a school/admin sets a list of `Milestone`s whose amounts
//! must sum to the student's `approved_amount` (so several students can
//! share one pool), the linked school releases milestones as work is
//! completed, and approved students claim funds up to the total released
//! for them.
//!
//! All monetary amounts are represented as `i128` to match the token
//! interface; conversions from other integer widths use checked
//! `try_into()` and surface `ContractError::AmountOverflow` rather than
//! silently wrapping.
//!
//! The contract is upgradeable: the platform admin may install a new WASM
//! via [`Contract::upgrade`], and [`Contract::version`] reports the
//! current release version.
//!
//! Storage is organized around a handful of key prefixes (pool records,
//! per-pool metadata, application and applicant records, milestones,
//! school registrations, claimed/protocol-fee accumulators, and pool
//! deadlines), each combined with the relevant `pool_id`/`Address` to form
//! a unique persistent storage key. A single platform `admin` address
//! gates privileged operations such as school registration and protocol
//! fee configuration/claiming.

#![cfg_attr(not(test), no_std)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, BytesN, Env,
    String, Symbol, Vec,
};

// Storage key constants
const POOL_COUNT: &str = "pool_count";
const APPLICATION_COUNT_PREFIX: &str = "a_count_";
const APPLICATION_PREFIX: &str = "a_";
const APPLICANT_PREFIX: &str = "ap_";
const MILESTONES_PREFIX: &str = "milestones";
const ADMIN_KEY: &str = "admin";
const SCHOOL_REG_PREFIX: &str = "school_reg";
const POOL_SCHOOL_PREFIX: &str = "pool_school";
const POOL_TOKEN_PREFIX: &str = "pool_token";
const SAVED_METADATA_PREFIX: &str = "saved_metadata";

// TODO: Replace with real implementation from issue #XYZ
// Emergency withdrawal storage keys
const EMERGENCY_WITHDRAWAL_PREFIX: &str = "emergency_withdraw";
const GRACE_PERIOD_SECS: u64 = 86400; // 24 hours

// Application and claim tracking constants
const CLAIMED_AMOUNT_PREFIX: &str = "claimed_amount";
const POOL_CLAIMED_PREFIX: &str = "pool_claimed";

// Protocol fees accumulator - tracks unclaimed fees collected from operations
const UNCLAIMED_FEES: &str = "unclaimed_fees";

// Creation fee key - stores the fee charged when creating a new pool
const CREATION_FEE_KEY: &str = "creation_fee";
const CROWDFUNDING_TOKEN_KEY: &str = "crowdfunding_token";

// Refund deadline constants
// Donors may request a refund only after the pool deadline has passed AND
// the grace period (REFUND_GRACE_PERIOD_LEDGERS) has elapsed.
const POOL_DEADLINE_PREFIX: &str = "pool_deadline";
const REFUND_GRACE_PERIOD_LEDGERS: u32 = 17_280; // ~24 hours at 5s/ledger

// ─── TTL Strategy (Issue #1656) ──────────────────────────────────────────
//
// Soroban persistent entries expire after the network default TTL unless
// their TTL is extended. Long-running pools would otherwise be archived and
// require a manual restore before donors/students could interact again.
//
// - Global config (admin, creation fee, crowdfunding token, pool count) lives
//   in *instance* storage; its TTL is extended on every write.
// - Pool, contribution, application and donor entries live in *persistent*
//   storage; their TTL is extended on both read and write paths.
//
// Thresholds are the minimum remaining TTL before an extension is triggered;
// extend values are the TTL applied when extending. Both are expressed in
// ledgers (~5s each).
const INSTANCE_TTL_THRESHOLD: u32 = 17_280; // ~24 hours
const INSTANCE_TTL_EXTEND: u32 = 518_400; // ~30 days
const PERSISTENT_TTL_THRESHOLD: u32 = 17_280; // ~24 hours
const PERSISTENT_TTL_EXTEND: u32 = 518_400; // ~30 days
/// Maximum horizon `set_pool_deadline` accepts: 10 years of ledgers
/// (~63 072 000 ledgers at 5 s/ledger). Bounding deadlines keeps
/// `deadline + REFUND_GRACE_PERIOD_LEDGERS` from overflowing `u32` in
/// `refund_donation` (release builds run with `overflow-checks = true`).
const MAX_POOL_DEADLINE_LEDGERS: u32 = 10 * 365 * 24 * 3600 / 5;

// Pool metadata validation constraints
const MAX_TITLE_LENGTH: u32 = 256;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const POOL_METADATA_PREFIX: &str = "metadata";
const MAX_URL_LENGTH: usize = 256;
const MAX_IMAGE_HASH_LENGTH: usize = 64;

// ─── Event Topics ────────────────────────────────────────────────────────

const POOL_CREATED: Symbol = symbol_short!("pool_crtd");
const DONATION_MADE: Symbol = symbol_short!("donation");
const CONTRIBUTION: Symbol = symbol_short!("contrib");
const POOL_CLOSED: Symbol = symbol_short!("pool_cls");
const APPLICATION_SUBMITTED: Symbol = symbol_short!("app_sub");
const SCHOOL_REGISTERED: Symbol = symbol_short!("schl_reg");

// Issue #954: named constants for previously-uneventful state-changing functions
const APP_APPROVED: Symbol = symbol_short!("app_aprvd");
const APP_REJECTED: Symbol = symbol_short!("app_rjctd");
const APP_REVOKED: Symbol = symbol_short!("app_rvk");
const MILESTONES_SET: Symbol = symbol_short!("mile_set");
const FUNDS_CLAIMED: Symbol = symbol_short!("fund_clmd");
const FEES_CLAIMED: Symbol = symbol_short!("fees_clmd");
const DONATION_REFUND: Symbol = symbol_short!("don_refnd");
const DEADLINE_SET: Symbol = symbol_short!("ddln_set");
const POOL_STATE_SET: Symbol = symbol_short!("pool_stat");
const ADMIN_SET: Symbol = symbol_short!("admin_set");
// Issue #954: shared constant replacing inline Symbol::new(&env, "creation_fee_updated")
const FEE_UPDATED: Symbol = symbol_short!("fee_upd");
const CROWDFUNDING_TOKEN_SET: Symbol = symbol_short!("tok_set");
const FEE_PAID: Symbol = symbol_short!("fee_paid");
const EMERGENCY_WITHDRAWAL_REQUESTED: Symbol = symbol_short!("emerg_req");
const EMERGENCY_WITHDRAWAL_EXECUTED: Symbol = symbol_short!("emerg_exe");
// Issue #1657: upgrade + versioning
const CONTRACT_UPGRADED: Symbol = symbol_short!("upgraded");
// Issue #1661: milestone release tracking
const MILESTONE_RELEASED: Symbol = symbol_short!("mile_rel");

/// Current contract release version. Bumped on every release so off-chain
/// clients can detect which contract build is deployed.
pub const VERSION: u32 = 1;

// ─── Typed Error Enum (Issue #955) ───────────────────────────────────────

/// All contract-level error conditions, encoded as XDR for off-chain callers.
///
/// Use `env.panic_with_error(ContractError::Variant)` instead of
/// `panic!("string literal")` for these conditions so that the error is
/// stable across contract versions and machine-readable.
#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    /// Pool with the given ID does not exist in storage.
    PoolNotFound = 1,
    /// Pool is not in the state required for this operation.
    InvalidPoolState = 2,
    /// Caller is not the stored platform admin.
    UnauthorizedAdmin = 3,
    /// Operation rejected because the pool is already closed.
    PoolIsClosed = 4,
    /// Student has already submitted an application for this pool.
    DuplicateApplication = 5,
    /// Student has not applied to this pool.
    StudentHasNotApplied = 6,
    /// Only the school linked to the pool may approve applications.
    OnlyLinkedSchoolCanApprove = 7,
    /// Pool must be in Disbursed or Cancelled state before it can be closed.
    PoolNotDisbursedOrRefunded = 8,
    /// No admin address has been configured in storage.
    AdminNotSet = 9,
    /// There are no accumulated protocol fees to claim.
    NoUnclaimedFees = 10,
    /// Fee value is invalid (e.g. negative).
    InvalidFee = 11,
    /// Pool deadline has not yet passed (or grace period not elapsed).
    PoolNotExpired = 12,
    /// Donor has no recorded contribution in this pool to refund.
    NoContributionToRefund = 13,
    /// School address has not been registered by an admin.
    SchoolNotRegistered = 14,
    /// Pool has no linked school.
    PoolHasNoLinkedSchool = 15,
    /// Pool has already been closed and cannot be closed again.
    PoolAlreadyClosed = 16,
    /// Pool title/name is empty.
    InvalidPoolName = 17,
    /// Pool funding goal is zero.
    InvalidPoolTarget = 18,
    /// Pool application deadline is zero or already in the past.
    InvalidPoolDeadline = 19,
    /// Donation attempted at or after the pool's application deadline.
    CampaignExpired = 20,
    /// No emergency withdrawal request exists for the pool.
    EmergencyWithdrawalNotRequested = 21,
    /// Emergency withdrawal grace period has not yet elapsed.
    GracePeriodNotElapsed = 22,
    /// Claim amount is zero or negative.
    InvalidClaimAmount = 23,
    /// Student has no application status record for this pool.
    ApplicationStatusNotFound = 24,
    /// Application status is not "Approved", so funds cannot be claimed.
    ApplicationNotApproved = 25,
    /// Claim would exceed the collected funds available to the pool.
    OverdrawAttempt = 26,
    /// Donation amount must be positive (greater than zero).
    InvalidAmount = 27,
    /// Token address does not match the pool's configured token.
    TokenTransferFailed = 28,
    /// Token address is invalid (e.g., contract's own address).
    InvalidTokenAddress = 29,
    /// The requested application status transition is not permitted.
    InvalidStatusTransition = 30,
    /// The refund deadline would precede the funding (application) deadline.
    InvalidRefundDeadline = 31,
    /// A monetary amount could not be converted without loss (e.g. a value
    /// outside the `i128` range).
    AmountOverflow = 34,
    /// Milestone total does not match the student's approved amount.
    MilestoneTotalMismatch = 35,
    /// Claim would exceed the total amount released for the student.
    MilestonesNotReleased = 32,
    /// A Cancelled pool still holds donor funds that must be refunded first.
    PoolHasUnrefundedFunds = 33,
}

// Helper functions for timestamp/deadline edge-case tests
// These are deterministic, test-oriented helpers used by unit tests
// to avoid reliance on external ledger state in the test harness.

/// Return a deterministic current timestamp for unit tests.
pub fn current_timestamp() -> u64 {
    // A stable timestamp greater than GRACE_PERIOD_SECS to avoid underflow
    100_000u64
}

/// Check whether a given deadline is within the provided grace period
/// relative to the deterministic current timestamp.
pub fn is_within_grace_period(deadline: u64, grace_period_secs: u64) -> bool {
    let now = current_timestamp();
    if now < deadline {
        return false;
    }
    now.saturating_sub(deadline) <= grace_period_secs
}

/// Validate that a deadline is strictly in the future and within a sane bound.
pub fn validate_deadline(deadline: u64) -> Result<(), &'static str> {
    let now = current_timestamp();
    if deadline <= now {
        return Err("Deadline in past or now");
    }
    // Bound future deadlines to 10 years from `now` to catch unreasonable values
    let max = now.saturating_add(10u64 * 365 * 24 * 3600);
    if deadline > max {
        return Err("Deadline too far in future");
    }
    Ok(())
}

/// Minimal setter simulation that enforces deadline must be in the future.
pub fn set_deadline(deadline: u64) -> Result<(), &'static str> {
    let now = current_timestamp();
    if deadline <= now {
        return Err("Deadline must be in future");
    }
    Ok(())
}

/// Lifecycle status of a student's application to a pool.
///
/// # State Machine
///
/// - `Pending` (initial state, set by `apply_to_pool`)
/// - `Pending -> Approved` (school/admin approves)
/// - `Pending -> Rejected` (school/admin rejects)
/// - `Approved -> Revoked` (only while `amount_claimed == 0`)
/// - `Rejected -> Approved` (re-approval by the school/admin)
///
/// Any other transition is rejected with
/// `ContractError::InvalidStatusTransition`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationStatus {
    /// Application submitted, awaiting a decision.
    Pending,
    /// Application approved; the student may claim funds.
    Approved,
    /// Application rejected; the student may not claim funds.
    Rejected,
    /// Previously approved application whose approval was withdrawn
    /// before any funds were claimed.
    Revoked,
}

/// Tracks a student's approved funding and how much has been streamed so far.
///
/// `amount_claimed` starts at zero and increments with each partial withdrawal,
/// allowing the contract to enforce the invariant:
///   amount_claimed + new_claim <= approved_amount
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Application {
    pub pool_id: u32,
    pub student: Address,
    pub application_data: String,
    pub status: ApplicationStatus,
    /// The total amount the student is approved to receive from this pool.
    pub approved_amount: i128,
    /// Running total of funds already disbursed to the student.
    /// Starts at 0; incremented on every successful partial claim.
    pub amount_claimed: i128,
}

/// Pool state machine enum representing the lifecycle of a donation pool.
///
/// # State Machine
///
/// The pool progresses through states as follows:
/// - **Active** (initial state): Pool accepts donations and applications
/// - **Paused**: Pool temporarily halted, donations and applications rejected
/// - **Completed**: Funding goal reached
/// - **Cancelled**: Pool was cancelled by sponsor or admin
/// - **Disbursed**: Funds have been distributed to approved students
/// - **Closed**: Pool is permanently closed, no further operations allowed
///
/// # State Transitions
///
/// Current state transition functions:
/// - `create_pool()` / `create_pool_for_school()`: Initializes pool to `Active`
/// - `donate()`: Moves an active pool to `Completed` when it reaches its goal
/// - `close_pool()`: Requires pool to be `Disbursed` or `Cancelled` before closing
///
/// # Validation Rules
///
/// - `donate()` only accepts donations if state is `Active`
/// - `close_pool()` only allows closing from `Disbursed` or `Cancelled` states
/// - The mock state setter is the only path into `Disbursed` or `Cancelled`
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PoolState {
    /// Pool is active and accepting donations and student applications.
    Active,
    /// Pool is temporarily paused; donations and applications are rejected.
    Paused,
    /// Funding goal has been reached.
    Completed,
    /// Pool has been cancelled and is no longer accepting funds or applications.
    Cancelled,
    /// Funds have been dispersed to approved students; no new disbursements allowed.
    Disbursed,
    /// Pool is permanently closed; no further operations are permitted.
    Closed,
}

/// Pool information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pool {
    /// The sponsor address that created and owns this pool.
    pub sponsor: Address,
    /// The funding goal amount for this pool.
    pub goal: i128,
    /// Total amount of funds collected so far through donations.
    pub collected: i128,
    /// Whether the pool has been permanently closed.
    pub is_closed: bool,
    /// Current state of the pool in its lifecycle (Active, Paused, Completed, Cancelled, Disbursed, Closed).
    pub state: PoolState,
    /// Ledger timestamp after which student applications are no longer accepted.
    pub application_deadline: u64,
    /// Ledger timestamp of the most recent contribution to this pool.
    /// Zero until the pool receives its first donation.
    pub last_donation_at: u64,
}

/// Milestone for streaming disbursements
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub amount: i128,
}

/// Pending emergency withdrawal of tokens from a pool.
///
/// Created by `request_emergency_withdraw` and stored until
/// `execute_emergency_withdraw` runs after the grace period elapses.
/// Tokens are sent to `requested_by`, not necessarily the caller who executes.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyWithdrawalRequest {
    /// The pool from which the emergency withdrawal was requested.
    pub pool_id: u32,
    /// Token contract to transfer when the withdrawal is executed.
    pub token_address: Address,
    /// Amount of tokens to withdraw, in the token's smallest unit.
    pub amount: i128,
    /// Ledger timestamp when the request was recorded; used for the grace-period check.
    pub request_timestamp: u64,
    /// Address that requested the withdrawal and will receive the tokens.
    pub requested_by: Address,
}

/// Validate an application status transition.
///
/// Allowed transitions:
/// - `Pending -> Approved`
/// - `Pending -> Rejected`
/// - `Rejected -> Approved` (re-approval)
/// - `Approved -> Revoked` only when `amount_claimed == 0`
///
/// Returns `Err(ContractError::InvalidStatusTransition)` otherwise.
pub fn validate_transition(
    from: &ApplicationStatus,
    to: &ApplicationStatus,
    amount_claimed: i128,
) -> Result<(), ContractError> {
    let allowed = match (from, to) {
        (ApplicationStatus::Pending, ApplicationStatus::Approved) => true,
        (ApplicationStatus::Pending, ApplicationStatus::Rejected) => true,
        (ApplicationStatus::Rejected, ApplicationStatus::Approved) => true,
        (ApplicationStatus::Approved, ApplicationStatus::Revoked) => amount_claimed == 0,
        _ => false,
    };
    if allowed {
        Ok(())
    } else {
        Err(ContractError::InvalidStatusTransition)
    }
}

/// Extend the TTL of the contract instance (global config) if it is close to
/// expiring. Called on every instance-storage write.
fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND);
}

/// Extend the TTL of a persistent entry if it is close to expiring.
///
/// Absent keys are skipped because `extend_ttl` traps on them, which would turn
/// a legitimately empty value (e.g. a pool with no applications) into an error.
fn extend_persistent_ttl<K: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, key: &K) {
    let storage = env.storage().persistent();
    if storage.has(key) {
        storage.extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND);
    }
}

/// Convention for pool-scoped getters (#1750): a nonexistent pool always fails
/// with `ContractError::PoolNotFound`. Empty values mean the pool exists.
fn require_pool_exists(env: &Env, pool_id: u32) {
    if !env.storage().persistent().has(&pool_id) {
        env.panic_with_error(ContractError::PoolNotFound);
    }
}

fn pool_claimed(env: &Env, pool_id: u32) -> i128 {
    let storage = env.storage().persistent();
    let total_key = (Symbol::new(env, POOL_CLAIMED_PREFIX), pool_id);
    if let Some(total) = storage.get(&total_key) {
        extend_persistent_ttl(env, &total_key);
        return total;
    }

    // Recover claims recorded before the pool-wide total was introduced.
    let count_key = (Symbol::new(env, APPLICATION_COUNT_PREFIX), pool_id);
    let count: u32 = storage.get(&count_key).unwrap_or(0);
    let application_prefix = Symbol::new(env, APPLICATION_PREFIX);
    let claimed_prefix = Symbol::new(env, CLAIMED_AMOUNT_PREFIX);
    let mut total = 0i128;
    for index in 1..=count {
        let key = (&application_prefix, pool_id, index);
        if let Some((_, student, _)) = storage.get::<_, (u32, Address, String)>(&key) {
            let key = (&claimed_prefix, pool_id, student);
            if let Some(application) = storage.get::<_, Application>(&key) {
                total = total.saturating_add(application.amount_claimed);
            }
        }
    }
    total
}

/// Storage key for the per-student released-milestone bitmap.
fn released_flags_key(env: &Env, pool_id: u32, student: &Address) -> (Symbol, u32, Address) {
    (
        Symbol::new(env, "mile_rel_flags"),
        pool_id,
        student.clone(),
    )
}

/// Storage key for the per-student released-total counter.
fn released_total_key(env: &Env, pool_id: u32, student: &Address) -> (Symbol, u32, Address) {
    (
        Symbol::new(env, "mile_rel_total"),
        pool_id,
        student.clone(),
    )
}

/// Return the total amount released (school-signed) for a student in a pool.
fn released_total(env: &Env, pool_id: u32, student: &Address) -> i128 {
    env.storage()
        .persistent()
        .get::<_, i128>(&released_total_key(env, pool_id, student))
        .unwrap_or(0)
}

/// Return the released flags for a student's milestones.
fn released_flags(env: &Env, pool_id: u32, student: &Address) -> Vec<bool> {
    env.storage()
        .persistent()
        .get::<_, Vec<bool>>(&released_flags_key(env, pool_id, student))
        .unwrap_or(Vec::new(env))
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Set the platform admin address.
    pub fn set_admin(env: Env, admin: Address) {
        admin.require_auth();
        let admin_key = Symbol::new(&env, ADMIN_KEY);
        env.storage().instance().set(&admin_key, &admin);
        extend_instance_ttl(&env);

        // Issue #954: emit admin-set event
        env.events().publish((ADMIN_SET,), admin.clone());
    }

    /// Register a school's on-chain identity mapping.
    ///
    /// Only the root protocol admin (set via [`Contract::set_admin`]) may call
    /// this. The `metadata_hash` is a 32-byte digest of the accredited body's
    /// off-chain metadata and is written to persistent ledger storage keyed by
    /// `school_addr`. Registering an already-registered school overwrites its
    /// metadata hash, allowing efficient in-place updates.
    pub fn register_school(env: Env, school_addr: Address, metadata_hash: BytesN<32>) {
        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let admin: Address = env
            .storage()
            .instance()
            .get::<_, Address>(&admin_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminNotSet));

        // Enforce root protocol admin authorization.
        admin.require_auth();

        let school_key = (Symbol::new(&env, SCHOOL_REG_PREFIX), school_addr.clone());
        env.storage().persistent().set(&school_key, &metadata_hash);

        env.events()
            .publish((SCHOOL_REGISTERED, school_addr), metadata_hash);
    }

    /// Check if a school has been registered.
    pub fn is_school_registered(env: Env, school: Address) -> bool {
        let school_key = (Symbol::new(&env, SCHOOL_REG_PREFIX), school);
        env.storage().persistent().has(&school_key)
    }

    /// Return the metadata hash recorded for a registered school.
    pub fn get_school_metadata(env: Env, school: Address) -> BytesN<32> {
        let school_key = (Symbol::new(&env, SCHOOL_REG_PREFIX), school);
        env.storage()
            .persistent()
            .get::<_, BytesN<32>>(&school_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::SchoolNotRegistered))
    }

    // ─── Pool Management ─────────────────────────────────────────────────────

    /// Create a new donation / sponsorship pool.
    ///
    /// # Authorization
    /// This function does **not** require the `creator` address to authorize the
    /// call (`creator.require_auth()` is never called). Any address can create
    /// a pool "as" any other address. For authorization, use
    /// [`create_pool_for_school`] which validates that the caller is authorized.
    pub fn create_pool(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: i128,
        application_deadline: u64,
    ) -> u32 {
        if title.len() == 0 {
            env.panic_with_error(ContractError::InvalidPoolName);
        }
        if description.len() == 0 {
            panic!("Description cannot be empty");
        }
        if goal == 0 {
            env.panic_with_error(ContractError::InvalidPoolTarget);
        }
        if description.len() as u32 > MAX_DESCRIPTION_LENGTH as u32 {
            panic!("Description exceeds maximum length");
        }
        if goal == 0 {
            env.panic_with_error(ContractError::InvalidPoolTarget);
        }
        if application_deadline == 0 {
            env.panic_with_error(ContractError::InvalidPoolDeadline);
        }

        let pool_count_key = Symbol::new(&env, POOL_COUNT);
        let mut pool_count: u32 = env
            .storage()
            .instance()
            .get::<_, u32>(&pool_count_key)
            .unwrap_or(0);

        let pool_id = pool_count + 1;
        pool_count = pool_id;

        let metadata_key = (Symbol::new(&env, "metadata"), pool_id);
        env.storage()
            .persistent()
            .set(&metadata_key, &(title.clone(), description.clone()));

        let pool = Pool {
            sponsor: creator.clone(),
            goal,
            collected: 0i128,
            is_closed: false,
            state: PoolState::Active,
            application_deadline,
            last_donation_at: 0u64,
        };

        env.storage().persistent().set(&pool_id, &pool);
        extend_persistent_ttl(&env, &pool_id);

        env.storage().instance().set(&pool_count_key, &pool_count);
        extend_instance_ttl(&env);

        // Emit pool creation event
        env.events().publish(
            (POOL_CREATED, pool_id),
            (
                pool.sponsor.clone(),
                goal,
                title.clone(),
                description.clone(),
            ),
        );

        pool_id
    }

    /// Create a new sponsorship pool linked to a registered school.
    pub fn create_pool_for_school(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: i128,
        school: Address,
        application_deadline: u64,
    ) -> u32 {
        creator.require_auth();

        if !Self::is_school_registered(env.clone(), school.clone()) {
            env.panic_with_error(ContractError::SchoolNotRegistered);
        }

        let pool_id = Self::create_pool(
            env.clone(),
            creator,
            title,
            description,
            goal,
            application_deadline,
        );
        let pool_school_key = (Symbol::new(&env, POOL_SCHOOL_PREFIX), pool_id);
        env.storage().persistent().set(&pool_school_key, &school);
        extend_persistent_ttl(&env, &pool_school_key);
        pool_id
    }

    /// Save mutable pool metadata and its multi-signature configuration.
    ///
    /// Updates the pool's metadata (description, URL, and image hash) and configures
    /// the multi-signature requirements for pool operations. Only the pool sponsor
    /// may call this function.
    ///
    /// # Constraints
    /// - `description` must not exceed `MAX_DESCRIPTION_LENGTH` (500 bytes)
    /// - `url` must not exceed `MAX_URL_LENGTH` (256 bytes)
    /// - `image_hash` must not exceed `MAX_IMAGE_HASH_LENGTH` (64 bytes)
    /// - `signers` must not be empty
    /// - `required_signatures` must be greater than zero
    /// - `required_signatures` must not exceed the number of signers
    /// - All signers must be unique (no duplicates allowed)
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `"Description exceeds maximum length"` if description length > 500
    /// - `"URL exceeds maximum length"` if url length > 256
    /// - `"Image hash exceeds maximum length"` if image_hash length > 64
    /// - `"Empty signers list"` if signers vector is empty
    /// - `"Zero required_signatures"` if required_signatures is 0
    /// - `"required_signatures exceeds signer count"` if required_signatures > signers.len()
    /// - `"Mismatched multi-signature parameters"` if any signers are duplicated
    /// - Authorization error if caller is not the pool sponsor
    pub fn save_pool(
        env: Env,
        pool_id: u32,
        description: String,
        url: String,
        image_hash: String,
        required_signatures: u32,
        signers: Vec<Address>,
    ) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));
        extend_persistent_ttl(&env, &pool_id);

        pool.sponsor.require_auth();

        if description.len() as usize > MAX_DESCRIPTION_LENGTH {
            panic!("Description exceeds maximum length");
        }
        if url.len() as usize > MAX_URL_LENGTH {
            panic!("URL exceeds maximum length");
        }
        if image_hash.len() as usize > MAX_IMAGE_HASH_LENGTH {
            panic!("Image hash exceeds maximum length");
        }

        if signers.is_empty() {
            panic!("Empty signers list");
        }
        if required_signatures == 0 {
            panic!("Zero required_signatures");
        }
        if required_signatures > signers.len() {
            panic!("required_signatures exceeds signer count");
        }
        let signer_count = signers.len();
        let mut i = 0u32;
        while i < signer_count {
            let mut j = i + 1;
            while j < signer_count {
                if signers.get(i).unwrap() == signers.get(j).unwrap() {
                    panic!("Mismatched multi-signature parameters");
                }
                j += 1;
            }
            i += 1;
        }

        let metadata_key = (Symbol::new(&env, SAVED_METADATA_PREFIX), pool_id);
        env.storage()
            .persistent()
            .set(&metadata_key, &(description, url, image_hash));
        extend_persistent_ttl(&env, &metadata_key);

        let signers_key = (Symbol::new(&env, "pool_signers"), pool_id);
        env.storage()
            .persistent()
            .set(&signers_key, &(required_signatures, signers));
        extend_persistent_ttl(&env, &signers_key);
    }

    /// Return the multi-signature threshold and signer list saved for a pool.
    pub fn get_pool_signers(env: Env, pool_id: u32) -> (u32, Vec<Address>) {
        require_pool_exists(&env, pool_id);
        let signers_key = (Symbol::new(&env, "pool_signers"), pool_id);
        env.storage()
            .persistent()
            .get(&(signers_key))
            .unwrap_or_else(|| (0u32, Vec::new(&env)))
    }

    /// Configure the token accepted by token-backed donations for a pool.
    pub fn set_pool_token(env: Env, pool_id: u32, token_address: Address) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.sponsor.require_auth();
        let token_key = (Symbol::new(&env, POOL_TOKEN_PREFIX), pool_id);
        env.storage().persistent().set(&token_key, &token_address);
        extend_persistent_ttl(&env, &token_key);
    }

    /// Get the token address configured for a pool via [`Contract::set_pool_token`].
    ///
    /// # Arguments
    /// * `env`     - The contract environment
    /// * `pool_id` - The ID of the pool to query
    ///
    /// # Returns
    /// The `Address` of the token accepted by token-backed donations for the pool.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if no pool with `pool_id` exists
    /// - `ContractError::PoolNotFound` if the pool exists but no token has been
    ///   configured for it
    pub fn get_pool_token(env: Env, pool_id: u32) -> Address {
        if !env.storage().persistent().has(&pool_id) {
            env.panic_with_error(ContractError::PoolNotFound);
        }

        let token_key = (Symbol::new(&env, POOL_TOKEN_PREFIX), pool_id);
        env.storage()
            .persistent()
            .get::<_, Address>(&token_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound))
    }

    /// Get the current lifecycle state of a pool.
    ///
    /// Unlike [`Contract::get_pool`], which omits the state, this getter exposes
    /// the pool's [`PoolState`] (Active, Paused, Completed, Cancelled, Disbursed
    /// or Closed) directly.
    ///
    /// # Arguments
    /// * `env`     - The contract environment
    /// * `pool_id` - The ID of the pool to query
    ///
    /// # Returns
    /// The pool's current [`PoolState`].
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if no pool with `pool_id` exists
    pub fn get_pool_state(env: Env, pool_id: u32) -> PoolState {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.state
    }

    /// Get the total amount of accumulated, unclaimed protocol fees.
    ///
    /// Fees accrue from operations such as `claim_funds` and
    /// `create_pool_with_fee`, and are paid out via
    /// [`Contract::claim_protocol_fees`].
    ///
    /// # Arguments
    /// * `env` - The contract environment
    ///
    /// # Returns
    /// The accumulated fee amount in the token's smallest unit, or `0` if no
    /// fees have accrued yet.
    pub fn get_unclaimed_fees(env: Env) -> i128 {
        let unclaimed_fees_key = Symbol::new(&env, UNCLAIMED_FEES);
        env.storage()
            .persistent()
            .get::<_, i128>(&unclaimed_fees_key)
            .unwrap_or(0)
    }

    /// Get the pending emergency withdrawal request for a pool, if any.
    ///
    /// Requests are created by [`Contract::request_emergency_withdraw`] and
    /// removed by [`Contract::execute_emergency_withdraw`].
    ///
    /// # Arguments
    /// * `env`     - The contract environment
    /// * `pool_id` - The ID of the pool to query
    ///
    /// # Returns
    /// The stored [`EmergencyWithdrawalRequest`] for the pool.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if no pool with `pool_id` exists
    /// - `ContractError::EmergencyWithdrawalNotRequested` if the pool exists but
    ///   has no pending emergency withdrawal request
    pub fn get_emergency_withdrawal(env: Env, pool_id: u32) -> EmergencyWithdrawalRequest {
        if !env.storage().persistent().has(&pool_id) {
            env.panic_with_error(ContractError::PoolNotFound);
        }

        let withdrawal_key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
        env.storage()
            .persistent()
            .get::<_, EmergencyWithdrawalRequest>(&withdrawal_key)
            .unwrap_or_else(|| {
                env.panic_with_error(ContractError::EmergencyWithdrawalNotRequested)
            })
    }

    /// Get the school linked to a pool.
    pub fn get_pool_school(env: Env, pool_id: u32) -> Address {
        require_pool_exists(&env, pool_id);
        let pool_school_key = (Symbol::new(&env, POOL_SCHOOL_PREFIX), pool_id);
        env.storage()
            .persistent()
            .get::<_, Address>(&pool_school_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolHasNoLinkedSchool))
    }

    /// Donate to an existing pool.
    ///
    /// **Note:** This function does **not** require donor authentication (`donor.require_auth()`) and only updates internal accounting. It does not perform a token transfer. Use `donate_with_token()` for token-backed donations that include donor auth.
    pub fn donate(env: Env, pool_id: u32, donor: Address, amount: i128) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        if pool.is_closed {
            env.panic_with_error(ContractError::PoolIsClosed);
        }

        // TODO: Replace with real implementation from issue #XYZ
        // Pool state validation
        if pool.state != PoolState::Active {
            env.panic_with_error(ContractError::InvalidPoolState);
        }

        if env.ledger().timestamp() >= pool.application_deadline {
            env.panic_with_error(ContractError::CampaignExpired);
        }

        let new_collected = pool.collected + amount;
        let next_state = if new_collected >= pool.goal {
            PoolState::Completed
        } else {
            pool.state.clone()
        };
        let updated_pool = Pool {
            collected: new_collected,
            state: next_state,
            last_donation_at: env.ledger().timestamp(),
            ..pool
        };
        env.storage().persistent().set(&pool_id, &updated_pool);
        extend_persistent_ttl(&env, &pool_id);

        // Emit donation event
        env.events().publish(
            (DONATION_MADE, pool_id),
            (donor.clone(), amount, new_collected),
        );
        // Track unique donors
        let donor_key = (pool_id, "donor", &donor);
        if !env.storage().persistent().has(&donor_key) {
            env.storage().persistent().set(&donor_key, &true);
            extend_persistent_ttl(&env, &donor_key);
            let count_key = (pool_id, "d_count");
            let donor_count: u32 = env
                .storage()
                .persistent()
                .get::<_, u32>(&count_key)
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&count_key, &(donor_count + 1));
            extend_persistent_ttl(&env, &count_key);
        } else {
            extend_persistent_ttl(&env, &donor_key);
        }

        // Track individual donor's total contribution
        let contrib_key = (pool_id, "contribution", &donor);
        let current_contrib: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&contrib_key, &(current_contrib + amount));
        extend_persistent_ttl(&env, &contrib_key);
    }

    /// Get pool information as a tuple (id, creator, goal, collected, is_closed).
    pub fn get_pool(env: Env, pool_id: u32) -> (u32, Address, i128, i128, bool, u64) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));
        extend_persistent_ttl(&env, &pool_id);

        (
            pool_id,
            pool.sponsor,
            pool.goal,
            pool.collected,
            pool.is_closed,
            pool.application_deadline,
        )
    }

    /// Get pool metadata as a tuple (title, description).
    /// Returns empty strings if the pool exists but has no metadata.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    pub fn get_pool_metadata(env: Env, pool_id: u32) -> (String, String) {
        require_pool_exists(&env, pool_id);
        let metadata_key = (Symbol::new(&env, POOL_METADATA_PREFIX), pool_id);
        env.storage()
            .persistent()
            .get::<_, (String, String)>(&metadata_key)
            .unwrap_or_else(|| (String::from_str(&env, ""), String::from_str(&env, "")))
    }

    /// Get saved description, URL, and image hash metadata.
    /// Returns empty strings if the pool exists but has no saved metadata.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    pub fn get_saved_pool_metadata(env: Env, pool_id: u32) -> (String, String, String) {
        require_pool_exists(&env, pool_id);
        let metadata_key = (Symbol::new(&env, SAVED_METADATA_PREFIX), pool_id);
        env.storage()
            .persistent()
            .get::<_, (String, String, String)>(&metadata_key)
            .unwrap_or_else(|| {
                (
                    String::from_str(&env, ""),
                    String::from_str(&env, ""),
                    String::from_str(&env, ""),
                )
            })
    }

    // Note: try_get_pool is auto-generated by Soroban SDK from get_pool
    // Commenting out manual implementation to avoid duplicate definition
    // /// Safely retrieve pool information.
    // pub fn try_get_pool(env: Env, pool_id: u32) -> Option<(u32, Address, u128, u128, bool)> {
    //     env.storage()
    //         .persistent()
    //         .get::<_, Pool>(&pool_id)
    //         .map(|pool| {
    //             (
    //                 pool_id,
    //                 pool.sponsor,
    //                 pool.goal,
    //                 pool.collected,
    //                 pool.is_closed,
    //             )
    //         })
    // }

    /// Get the total amount raised for a pool.
    pub fn get_total_raised(env: Env, pool_id: u32) -> i128 {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));
        extend_persistent_ttl(&env, &pool_id);

        pool.collected
    }

    /// Get the campaign balance (total amount raised) for a pool.
    ///
    /// Issue #1269: Returns the total collected donation balance for the specified campaign.
    /// Panics with `ContractError::PoolNotFound` if the campaign does not exist.
    pub fn get_campaign_balance(env: Env, pool_id: u32) -> i128 {
        Self::get_total_raised(env, pool_id)
    }

    /// Get the ledger timestamp of the most recent contribution to a pool.
    /// Returns 0 if the pool has never received a donation.
    pub fn get_last_donation_at(env: Env, pool_id: u32) -> u64 {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));
        extend_persistent_ttl(&env, &pool_id);

        pool.last_donation_at
    }

    /// Close a donation pool.
    pub fn close_pool(env: Env, pool_id: u32) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.sponsor.require_auth();

        if pool.is_closed {
            env.panic_with_error(ContractError::PoolAlreadyClosed);
        }

        if pool.state != PoolState::Disbursed && pool.state != PoolState::Cancelled {
            env.panic_with_error(ContractError::PoolNotDisbursedOrRefunded);
        }

        // Issue #1678: a Cancelled pool must have refunded every donor before
        // it can be closed. Otherwise closing would strand donor funds, since
        // `refund_donation` refuses to operate on a closed pool.
        if pool.state == PoolState::Cancelled && pool.collected > 0 {
            env.panic_with_error(ContractError::PoolHasUnrefundedFunds);
        }

        let updated_pool = Pool {
            is_closed: true,
            state: PoolState::Closed,
            ..pool
        };

        env.storage().persistent().set(&pool_id, &updated_pool);
        extend_persistent_ttl(&env, &pool_id);

        // Emit pool closed event
        env.events().publish(
            (POOL_CLOSED, pool_id),
            (updated_pool.sponsor.clone(), updated_pool.collected),
        );
    }

    /// Return campaign ids in creation order.
    pub fn get_all_campaigns(env: Env) -> Vec<u32> {
        let count = Self::get_pool_count(env.clone());
        let mut campaigns = Vec::new(&env);
        let mut id = 1u32;
        while id <= count {
            if env.storage().persistent().has(&id) {
                campaigns.push_back(id);
            }
            id += 1;
        }
        campaigns
    }

    /// Get the total number of pools.
    pub fn get_pool_count(env: Env) -> u32 {
        let pool_count_key = Symbol::new(&env, POOL_COUNT);
        env.storage()
            .instance()
            .get::<_, u32>(&pool_count_key)
            .unwrap_or(0)
    }

    /// Get the number of unique donors for a pool.
    pub fn get_donor_count(env: Env, pool_id: u32) -> u32 {
        // Verify the pool exists first (issue #1482: an existence check is
        // enough here, a full Pool read is not required).
        if !env.storage().persistent().has(&pool_id) {
            env.panic_with_error(ContractError::PoolNotFound);
        }

        let count_key = (pool_id, "d_count");
        let count = env
            .storage()
            .persistent()
            .get::<_, u32>(&count_key)
            .unwrap_or(0);
        extend_persistent_ttl(&env, &count_key);
        count
    }

    /// Get the total contribution of a specific donor to a specific pool.
    pub fn get_contribution(env: Env, pool_id: u32, donor: Address) -> i128 {
        // Verify the pool exists first (issue #1482: an existence check is
        // enough here, a full Pool read is not required).
        if !env.storage().persistent().has(&pool_id) {
            env.panic_with_error(ContractError::PoolNotFound);
        }

        let contrib_key = (pool_id, "contribution", &donor);
        let contribution = env
            .storage()
            .persistent()
            .get::<_, i128>(&contrib_key)
            .unwrap_or(0);
        extend_persistent_ttl(&env, &contrib_key);
        contribution
    }

    /// Get the number of applications submitted to a pool.
    pub fn get_application_count(env: Env, pool_id: u32) -> u32 {
        require_pool_exists(&env, pool_id);
        let count_key = (Symbol::new(&env, APPLICATION_COUNT_PREFIX), pool_id);
        let count = env
            .storage()
            .persistent()
            .get::<_, u32>(&count_key)
            .unwrap_or(0);
        extend_persistent_ttl(&env, &count_key);
        count
    }

    /// Student applies to a school-linked pool.
    ///
    /// Records a student's application to a pool. The pool must be in Active state
    /// and not closed. A student may only apply once per pool; duplicate applications
    /// are rejected.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::PoolIsClosed` if the pool has been closed
    /// - `ContractError::InvalidPoolState` if the pool is not in Active state
    /// - `ContractError::DuplicateApplication` if the student has already applied to this pool
    pub fn apply_to_pool(env: Env, pool_id: u32, student: Address, application_data: String) {
        student.require_auth();

        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        if pool.is_closed {
            env.panic_with_error(ContractError::PoolIsClosed);
        }

        if pool.state != PoolState::Active {
            env.panic_with_error(ContractError::InvalidPoolState);
        }

        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if env.storage().persistent().has(&applicant_key) {
            env.panic_with_error(ContractError::DuplicateApplication);
        }

        let count_key = (Symbol::new(&env, APPLICATION_COUNT_PREFIX), pool_id);
        let mut app_count: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&count_key)
            .unwrap_or(0);
        app_count += 1;

        let app_key = (Symbol::new(&env, APPLICATION_PREFIX), pool_id, app_count);
        env.storage().persistent().set(
            &app_key,
            &(app_count, student.clone(), application_data.clone()),
        );
        extend_persistent_ttl(&env, &app_key);

        let application_key = (
            Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage().persistent().set(
            &application_key,
            &Application {
                pool_id,
                student: student.clone(),
                application_data,
                status: ApplicationStatus::Pending,
                approved_amount: 0,
                amount_claimed: 0,
            },
        );
        extend_persistent_ttl(&env, &application_key);

        env.storage().persistent().set(&applicant_key, &true);
        extend_persistent_ttl(&env, &applicant_key);
        env.storage().persistent().set(&count_key, &app_count);
        extend_persistent_ttl(&env, &count_key);

        // Emit application/contribution event with privacy flag (default: false for public)
        env.events().publish(
            (APPLICATION_SUBMITTED, pool_id),
            (student.clone(), app_count, false), // false = public application
        );
    }

    /// School approves or rejects a student's application.
    ///
    /// Requires authorization from `school` (`school.require_auth()`), and
    /// `school` must be the school linked to the pool. Sets the application
    /// status to approved or rejected according to `approved` and emits an
    /// `app_aprvd` event.
    ///
    /// The pool must not be closed and must be in an appropriate state for approvals.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::PoolIsClosed` if the pool has been closed
    /// - `ContractError::InvalidPoolState` if the pool is in Cancelled state
    /// - `ContractError::OnlyLinkedSchoolCanApprove` if `school` is not the
    ///   school linked to the pool
    /// - `ContractError::StudentHasNotApplied` if the student never applied
    ///   to the pool
    pub fn approve_application(
        env: Env,
        pool_id: u32,
        school: Address,
        student: Address,
        approved: bool,
    ) {
        school.require_auth();

        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        if pool.is_closed {
            env.panic_with_error(ContractError::PoolIsClosed);
        }

        if pool.state == PoolState::Cancelled {
            env.panic_with_error(ContractError::InvalidPoolState);
        }

        let linked_school = Self::get_pool_school(env.clone(), pool_id);
        if linked_school != school {
            env.panic_with_error(ContractError::OnlyLinkedSchoolCanApprove);
        }

        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if !env.storage().persistent().has(&applicant_key) {
            env.panic_with_error(ContractError::StudentHasNotApplied);
        }

        let status = if approved {
            ApplicationStatus::Approved
        } else {
            ApplicationStatus::Rejected
        };
        Self::set_application_status(env.clone(), school.clone(), pool_id, student.clone(), status);

        // Emit a distinct event for approve vs reject.
        if approved {
            env.events()
                .publish((APP_APPROVED, pool_id), (student.clone(),));
        } else {
            env.events()
                .publish((APP_REJECTED, pool_id), (student.clone(),));
        }
    }

    /// Revoke a previously approved application.
    ///
    /// Only the linked school (or admin) may revoke. Revocation is only
    /// permitted while the student has not yet claimed any funds
    /// (`amount_claimed == 0`); otherwise the transition fails with
    /// `ContractError::InvalidStatusTransition`.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::StudentHasNotApplied` if the student never applied
    /// - `ContractError::InvalidStatusTransition` if the application is not
    ///   `Approved` or the student has already claimed funds
    pub fn revoke_application(env: Env, pool_id: u32, school: Address, student: Address) {
        school.require_auth();

        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if !env.storage().persistent().has(&applicant_key) {
            env.panic_with_error(ContractError::StudentHasNotApplied);
        }

        Self::set_application_status(
            env.clone(),
            school,
            pool_id,
            student.clone(),
            ApplicationStatus::Revoked,
        );

        env.events()
            .publish((APP_REVOKED, pool_id), (student.clone(),));
    }

    /// Set application milestones and enforce sum(amounts) == approved_amount.
    ///
    /// Issue #1661: milestone totals are validated against the student's
    /// `approved_amount` rather than the pool goal, so several students can
    /// share one pool. Milestones start unreleased; the linked school must
    /// call [`Contract::release_milestone`] before the student can claim.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::StudentHasNotApplied` if the student never applied
    /// - `"Application is not approved"` if the application has not been
    ///   approved via `approve_application` (Issue #1345)
    /// - `ContractError::MilestoneTotalMismatch` if the sum of milestone
    ///   amounts does not equal the student's approved amount
    pub fn setup_application_milestones(
        env: Env,
        pool_id: u32,
        student: Address,
        milestones: Vec<Milestone>,
    ) {
        let _pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        // Issue #1662: milestones are configured by the linked school (or the
        // platform admin), matching `approve_application`. The student cannot
        // define their own disbursement schedule.
        let linked_school = Self::get_pool_school(env.clone(), pool_id);
        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let admin: Option<Address> = env.storage().persistent().get::<_, Address>(&admin_key);
        let is_admin = admin.map(|a| a == student).unwrap_or(false);
        if is_admin {
            // Admin path: the admin address must authorize the call.
            student.require_auth();
        } else {
            linked_school.require_auth();
        }

        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if !env.storage().persistent().has(&applicant_key) {
            env.panic_with_error(ContractError::StudentHasNotApplied);
        }

        // Issue #1345: milestones may only be configured for an approved
        // application. Mirrors the same check already enforced by claim_funds.
        let status = Self::get_application_status(env.clone(), pool_id, student.clone());
        if status != ApplicationStatus::Approved {
            panic!("Application is not approved");
        }

        if milestones.is_empty() {
            panic!("Milestones required");
        }

        // Issue #1661: validate against the student's approved amount.
        let approved_amount = Self::get_application(env.clone(), pool_id, student.clone())
            .map(|app| app.approved_amount)
            .unwrap_or(0);
        if approved_amount <= 0 {
            env.panic_with_error(ContractError::MilestoneTotalMismatch);
        }

        let mut sum: i128 = 0;
        for i in 0..milestones.len() {
            let amount = milestones.get(i).unwrap().amount;
            if amount <= 0 {
                env.panic_with_error(ContractError::InvalidAmount);
            }
            sum = sum
                .checked_add(amount)
                .unwrap_or_else(|| env.panic_with_error(ContractError::AmountOverflow));
        }

        if sum != approved_amount {
            env.panic_with_error(ContractError::MilestoneTotalMismatch);
        }

        let milestones_key = (
            Symbol::new(&env, MILESTONES_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage().persistent().set(&milestones_key, &milestones);
        extend_persistent_ttl(&env, &milestones_key);

        // Reset release tracking for the new schedule.
        let mut flags = Vec::new(&env);
        for _ in 0..milestones.len() {
            flags.push_back(false);
        }
        env.storage()
            .persistent()
            .set(&released_flags_key(&env, pool_id, &student), &flags);
        env.storage()
            .persistent()
            .set(&released_total_key(&env, pool_id, &student), &0i128);

        // Issue #954: emit milestones-set event
        env.events().publish(
            (MILESTONES_SET, pool_id),
            (student.clone(), milestones.len()),
        );
    }

    /// Release a single milestone for a student. Only the pool's linked
    /// school (or the platform admin) may release milestones.
    ///
    /// Releasing a milestone increases the student's released total, which
    /// caps how much they may claim via [`Contract::claim_funds`].
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::UnauthorizedAdmin` if the caller is neither the
    ///   linked school nor the platform admin
    /// - `ContractError::StudentHasNotApplied` if the student never applied
    /// - `"Milestone index out of bounds"` if `index` is not a valid milestone
    /// - `"Milestone already released"` if the milestone was already released
    pub fn release_milestone(env: Env, pool_id: u32, school: Address, student: Address, index: u32) {
        school.require_auth();

        let _pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        let linked_school = Self::get_pool_school(env.clone(), pool_id);
        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let is_admin = env
            .storage()
            .persistent()
            .get::<_, Address>(&admin_key)
            .map(|a| a == school)
            .unwrap_or(false);
        if school != linked_school && !is_admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }

        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if !env.storage().persistent().has(&applicant_key) {
            env.panic_with_error(ContractError::StudentHasNotApplied);
        }

        let milestones_key = (
            Symbol::new(&env, MILESTONES_PREFIX),
            pool_id,
            student.clone(),
        );
        let milestones: Vec<Milestone> = env
            .storage()
            .persistent()
            .get::<_, Vec<Milestone>>(&milestones_key)
            .unwrap_or(Vec::new(&env));
        if index >= milestones.len() {
            panic!("Milestone index out of bounds");
        }

        let mut flags = released_flags(&env, pool_id, &student);
        if flags.is_empty() {
            for _ in 0..milestones.len() {
                flags.push_back(false);
            }
        }
        if flags.get(index).unwrap() {
            panic!("Milestone already released");
        }
        flags.set(index, true);
        env.storage()
            .persistent()
            .set(&released_flags_key(&env, pool_id, &student), &flags);

        let amount = milestones.get(index).unwrap().amount;
        let new_total = released_total(&env, pool_id, &student)
            .checked_add(amount)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AmountOverflow));
        env.storage()
            .persistent()
            .set(&released_total_key(&env, pool_id, &student), &new_total);

        env.events().publish(
            (MILESTONE_RELEASED, pool_id),
            (student.clone(), index, amount, new_total),
        );
    }

    /// Release several milestones for a student in one call.
    ///
    /// Each index is released via [`Contract::release_milestone`] semantics;
    /// the caller must be the linked school or the platform admin.
    pub fn release_milestones(
        env: Env,
        pool_id: u32,
        school: Address,
        student: Address,
        indexes: Vec<u32>,
    ) {
        for i in 0..indexes.len() {
            Self::release_milestone(
                env.clone(),
                pool_id,
                school.clone(),
                student.clone(),
                indexes.get(i).unwrap(),
            );
        }
    }

    /// Return the total amount released for a student in a pool.
    pub fn get_released_total(env: Env, pool_id: u32, student: Address) -> i128 {
        require_pool_exists(&env, pool_id);
        released_total(&env, pool_id, &student)
    }

    /// Return the per-milestone released flags for a student in a pool.
    pub fn get_released_flags(env: Env, pool_id: u32, student: Address) -> Vec<bool> {
        require_pool_exists(&env, pool_id);
        released_flags(&env, pool_id, &student)
    }

    /// Get student milestones for a pool.
    pub fn get_milestones(env: Env, pool_id: u32, student: Address) -> Vec<Milestone> {
        require_pool_exists(&env, pool_id);
        let milestones_key = (Symbol::new(&env, MILESTONES_PREFIX), pool_id, student);
        let milestones = env
            .storage()
            .persistent()
            .get::<_, Vec<Milestone>>(&milestones_key)
            .unwrap_or(Vec::new(&env));
        extend_persistent_ttl(&env, &milestones_key);
        milestones
    }

    /// Set application status for a student in a pool.
    ///
    /// The status is stored only inside the `Application` record. The
    /// transition is validated against the documented state machine
    /// (see [`ApplicationStatus`]) and rejected with
    /// `ContractError::InvalidStatusTransition` when not permitted.
    pub fn set_application_status(
        env: Env,
        caller: Address,
        pool_id: u32,
        student: Address,
        status: ApplicationStatus,
    ) {
        caller.require_auth();
        let linked_school = Self::get_pool_school(env.clone(), pool_id);
        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let is_admin = env
            .storage()
            .instance()
            .get::<_, Address>(&admin_key)
            .map(|a| a == caller)
            .unwrap_or(false);

        if caller != linked_school && !is_admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }
        let application_key = (
            Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        let mut application: Application = env
            .storage()
            .persistent()
            .get::<_, Application>(&application_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::ApplicationStatusNotFound));

        if let Err(err) =
            validate_transition(&application.status, &status, application.amount_claimed)
        {
            env.panic_with_error(err);
        }

        application.status = status;
        env.storage()
            .persistent()
            .set(&application_key, &application);
        extend_persistent_ttl(&env, &application_key);
    }

    /// Get application status for a student in a pool.
    ///
    /// Returns `ApplicationStatus::Pending` if the student has not applied.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    pub fn get_application_status(env: Env, pool_id: u32, student: Address) -> ApplicationStatus {
        require_pool_exists(&env, pool_id);
        let application_key = (
            Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage()
            .persistent()
            .get::<_, Application>(&application_key)
            .map(|application| application.status)
            .unwrap_or(ApplicationStatus::Pending)
    }

    /// Returns true if the student has already applied to the given pool.
    pub fn has_applied(env: Env, pool_id: u32, student: Address) -> bool {
        require_pool_exists(&env, pool_id);
        let applicant_key = (Symbol::new(&env, APPLICANT_PREFIX), pool_id, student);
        env.storage().persistent().has(&applicant_key)
    }

    /// Get claimed amount for a student in a pool.
    pub fn get_claimed_amount(env: Env, pool_id: u32, student: Address) -> i128 {
        require_pool_exists(&env, pool_id);
        let claimed_key = (
            Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        // Try to get the Application struct first
        let app: Option<Application> = env.storage().persistent().get(&claimed_key);
        extend_persistent_ttl(&env, &claimed_key);
        match app {
            Some(application) => application.amount_claimed,
            None => 0,
        }
    }

    /// Get the full Application record for a student in a pool.
    /// Returns `None` if the student has not applied to the pool.
    pub fn get_application(env: Env, pool_id: u32, student: Address) -> Option<Application> {
        require_pool_exists(&env, pool_id);
        let app_key = (
            Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        let application = env.storage().persistent().get::<_, Application>(&app_key);
        extend_persistent_ttl(&env, &app_key);
        application
    }

    /// Returns the stored application tuple (app_count, applicant, data) for a given pool and index, if it exists.
    pub fn get_application_by_index(
        env: Env,
        pool_id: u32,
        index: u32,
    ) -> Option<(u32, Address, String)> {
        require_pool_exists(&env, pool_id);
        let app_key = (Symbol::new(&env, APPLICATION_PREFIX), pool_id, index);
        let application = env
            .storage()
            .persistent()
            .get::<_, (u32, Address, String)>(&app_key);
        extend_persistent_ttl(&env, &app_key);
        application
    }

    /// Withdraw surplus funds not locked by active applications.
    ///
    /// Locked funds = sum of (approved_amount - amount_claimed) for every
    /// application whose status is `Approved` or `Pending`.
    /// Surplus = pool.collected - locked_funds.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `"Insolvency: locked funds exceed collected"` if locked > collected
    /// - `"No surplus to withdraw"` if surplus == 0
    pub fn withdraw_unallocated_funds(env: Env, pool_id: u32, token_address: Address) {
        // TODO(#1748): resolve pool token and reject a mismatched token_address
        let mut pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.sponsor.require_auth();

        let count_key = (Symbol::new(&env, APPLICATION_COUNT_PREFIX), pool_id);
        let app_count: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&count_key)
            .unwrap_or(0);

        let mut locked: i128 = 0i128;
        for idx in 1..=app_count {
            let app_key = (Symbol::new(&env, APPLICATION_PREFIX), pool_id, idx);
            let entry: Option<(u32, Address, soroban_sdk::String)> =
                env.storage().persistent().get(&app_key);
            if let Some((_, student, _)) = entry {
                let claim_key = (
                    Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
                    pool_id,
                    student.clone(),
                );
                let application: Application = env
                    .storage()
                    .persistent()
                    .get::<_, Application>(&claim_key)
                    .unwrap_or(Application {
                        pool_id,
                        student: student.clone(),
                        application_data: String::from_str(&env, ""),
                        status: ApplicationStatus::Pending,
                        approved_amount: 0,
                        amount_claimed: 0,
                    });

                if matches!(
                    application.status,
                    ApplicationStatus::Approved | ApplicationStatus::Pending
                ) {
                    let remaining =
                        (application.approved_amount - application.amount_claimed).max(0);
                    locked = locked
                        .checked_add(remaining)
                        .expect("Locked funds overflow");
                }
            }
        }

        let surplus: i128 = pool
            .collected
            .checked_sub(locked)
            .expect("Insolvency: locked funds exceed collected");

        if surplus == 0 {
            panic!("No surplus to withdraw");
        }

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &pool.sponsor, &surplus);

        pool.collected -= surplus;
        env.storage().persistent().set(&pool_id, &pool);
        extend_persistent_ttl(&env, &pool_id);
    }

    /// Claim funds: allows an approved student to receive a partial or full
    /// disbursement from a pool.
    ///
    /// Uses `Application` to persist `amount_claimed` across calls, enabling
    /// streamed / milestone-based withdrawals where the student draws down
    /// their approved allocation incrementally.
    ///
    /// # Arguments
    /// * `env`           - The contract environment
    /// * `student`       - The student address receiving funds (must authorize)
    /// * `pool_id`       - The ID of the pool to claim from
    /// * `claim_amount`  - The amount to claim this call (must be > 0)
    /// * `token_address` - The token used for the transfer
    ///
    /// # Panics
    /// - `ContractError::InvalidClaimAmount` if `claim_amount <= 0`
    /// - `ContractError::ApplicationStatusNotFound` if no status has been set
    /// - `ContractError::ApplicationNotApproved` if status != "Approved"
    /// - `ContractError::PoolIsClosed` if the pool is closed
    /// - `ContractError::InvalidPoolState` if the pool is cancelled or disbursed
    /// - `ContractError::OverdrawAttempt` if the claim exceeds the pool's unclaimed funds
    /// - `ContractError::TokenTransferFailed` if a configured pool token does not match
    pub fn claim_funds(
        env: Env,
        student: Address,
        pool_id: u32,
        claim_amount: i128,
        token_address: Address,
    ) {
        student.require_auth();

        if claim_amount <= 0 {
            env.panic_with_error(ContractError::InvalidClaimAmount);
        }

        // Verify application is approved
        let status = Self::get_application_status(env.clone(), pool_id, student.clone());
        if status != ApplicationStatus::Approved {
            env.panic_with_error(ContractError::ApplicationNotApproved);
        }

        // Load pool to check available collected funds
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        if pool.is_closed || pool.state == PoolState::Closed {
            env.panic_with_error(ContractError::PoolIsClosed);
        }

        if matches!(pool.state, PoolState::Cancelled | PoolState::Disbursed) {
            env.panic_with_error(ContractError::InvalidPoolState);
        }

        let token_key = (Symbol::new(&env, POOL_TOKEN_PREFIX), pool_id);
        if env
            .storage()
            .persistent()
            .get::<_, Address>(&token_key)
            .is_some_and(|expected| expected != token_address)
        {
            env.panic_with_error(ContractError::TokenTransferFailed);
        }

        let collected = pool.collected;

        // Load or initialise the Application record for this student
        let app_key = (
            Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        let mut application: Application = env
            .storage()
            .persistent()
            .get::<_, Application>(&app_key)
            .unwrap_or(Application {
                pool_id,
                student: student.clone(),
                application_data: String::from_str(&env, ""),
                status: ApplicationStatus::Approved,
                approved_amount: collected,
                amount_claimed: 0,
            });

        if application.approved_amount == 0 {
            application.approved_amount = collected;
        }

        let student_total = application
            .amount_claimed
            .checked_add(claim_amount)
            .filter(|total| *total <= collected)
            .unwrap_or_else(|| env.panic_with_error(ContractError::OverdrawAttempt));

        // Issue #1661: claims are additionally gated by the total amount the
        // school has released for this student's milestones.
        let released = released_total(&env, pool_id, &student);
        if student_total > released {
            env.panic_with_error(ContractError::MilestonesNotReleased);
        }

        let pool_total = pool_claimed(&env, pool_id)
            .checked_add(claim_amount)
            .filter(|total| *total <= pool.collected)
            .unwrap_or_else(|| env.panic_with_error(ContractError::OverdrawAttempt));

        let fee = claim_amount / 100;
        let net_transfer = claim_amount - fee;
        let storage = env.storage().persistent();
        let unclaimed_fees_key = Symbol::new(&env, UNCLAIMED_FEES);
        let current_fees: i128 = storage.get(&unclaimed_fees_key).unwrap_or(0);

        application.amount_claimed = student_total;
        storage.set(&app_key, &application);
        extend_persistent_ttl(&env, &app_key);
        let pool_claimed_key = (Symbol::new(&env, POOL_CLAIMED_PREFIX), pool_id);
        storage.set(&pool_claimed_key, &pool_total);
        extend_persistent_ttl(&env, &pool_claimed_key);
        storage.set(&unclaimed_fees_key, &(current_fees + fee));
        extend_persistent_ttl(&env, &unclaimed_fees_key);

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &student, &net_transfer);

        // Issue #954: emit funds-claimed event
        env.events().publish(
            (FUNDS_CLAIMED, pool_id),
            (student.clone(), claim_amount, application.amount_claimed),
        );
    }

    /// Claim accumulated protocol fees on behalf of the protocol/treasury.
    ///
    /// Allows Protocol Admins to retrieve all accumulated fees from operations.
    /// This function separates fee tracking cleanly from active token allocations.
    ///
    /// # Arguments
    /// * `env`           - The contract environment
    /// * `admin`         - The admin address claiming fees (must authorize)
    /// * `token_address` - The token to transfer fees as
    ///
    /// # Panics
    /// - `ContractError::UnauthorizedAdmin` if the caller is not the stored admin address
    /// - `ContractError::NoUnclaimedFees` if there are no accumulated fees to claim
    pub fn claim_protocol_fees(env: Env, admin: Address, token_address: Address) -> i128 {
        admin.require_auth();

        // Verify caller is the protocol admin
        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let stored_admin: Address = env
            .storage()
            .instance()
            .get::<_, Address>(&admin_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminNotSet));
        if stored_admin != admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }

        // Get accumulated unclaimed fees
        let unclaimed_fees_key = Symbol::new(&env, UNCLAIMED_FEES);
        let fees: i128 = env
            .storage()
            .persistent()
            .get::<_, i128>(&unclaimed_fees_key)
            .unwrap_or(0);

        if fees == 0 {
            env.panic_with_error(ContractError::NoUnclaimedFees);
        }

        // Transfer accumulated fees to admin
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &admin, &fees);

        // Reset unclaimed fees to 0
        env.storage().persistent().set(&unclaimed_fees_key, &0i128);
        extend_persistent_ttl(&env, &unclaimed_fees_key);

        // Issue #954: emit fees-claimed event
        env.events().publish((FEES_CLAIMED, admin.clone()), (fees,));

        fees
    }

    // ─── Creation Fee ─────────────────────────────────────────────────────────

    /// Set the pool creation fee (in stroops / smallest token unit).
    ///
    /// Only the stored admin may call this function.
    /// A fee of zero is valid (disables the creation fee).
    ///
    /// Emits exactly one `fee_upd` (`FEE_UPDATED`) event on success.
    ///
    /// # Panics
    /// - `ContractError::AdminNotSet` if no admin has been configured
    /// - `ContractError::UnauthorizedAdmin` if `admin` does not match the stored admin
    /// - `ContractError::InvalidFee` if `fee` is negative
    pub fn set_creation_fee(env: Env, admin: Address, fee: i128) {
        admin.require_auth();

        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let stored_admin: Address = env
            .storage()
            .instance()
            .get::<_, Address>(&admin_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminNotSet));
        if stored_admin != admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }

        if fee < 0 {
            env.panic_with_error(ContractError::InvalidFee);
        }

        let fee_key = Symbol::new(&env, CREATION_FEE_KEY);
        env.storage().instance().set(&fee_key, &fee);
        extend_instance_ttl(&env);

        // Issue #1461: FEE_UPDATED is the canonical topic; emit it once only.
        env.events().publish((FEE_UPDATED,), fee);
    }

    /// Get the current pool creation fee.
    /// Returns `0` if no fee has been set.
    pub fn get_creation_fee(env: Env) -> i128 {
        let fee_key = Symbol::new(&env, CREATION_FEE_KEY);
        env.storage()
            .instance()
            .get::<_, i128>(&fee_key)
            .unwrap_or(0)
    }

    // ─── Refund Deadline ──────────────────────────────────────────────────────

    /// Set the refund deadline (as a ledger sequence number) for a pool.
    ///
    /// Only the pool sponsor may call this.
    /// The deadline must be in the future (greater than the current ledger)
    /// and no more than `MAX_POOL_DEADLINE_LEDGERS` ledgers ahead of it.
    ///
    /// Issue #1664: ledger sequence is the single time base for refunds. The
    /// refund deadline must not precede the pool's funding deadline, so a pool
    /// can never be refundable while donations are still being accepted.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `"Error(Auth, InvalidAction)"` if caller is not the pool sponsor
    /// - `"Deadline must be in the future"` if deadline <= current ledger
    /// - `"Deadline too far in the future"` if the deadline exceeds the maximum horizon
    /// - `ContractError::InvalidRefundDeadline` if the deadline precedes the
    ///   pool's funding deadline
    pub fn set_pool_deadline(env: Env, pool_id: u32, deadline: u32) {
        let _pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        _pool.sponsor.require_auth();

        let current_seq = env.ledger().sequence();
        if deadline <= current_seq {
            panic!("Deadline must be in the future");
        }
        if deadline > current_seq.saturating_add(MAX_POOL_DEADLINE_LEDGERS) {
            panic!("Deadline too far in the future");
        }

        // Issue #1664: the refund window must not open before the funding
        // deadline. `application_deadline` is a timestamp; convert it to the
        // ledger sequence at which it elapses so both share one time base.
        let funding_deadline_ledger = Self::funding_deadline_ledger(&env, &_pool);
        if deadline < funding_deadline_ledger {
            env.panic_with_error(ContractError::InvalidRefundDeadline);
        }

        let deadline_key = (Symbol::new(&env, POOL_DEADLINE_PREFIX), pool_id);
        env.storage().persistent().set(&deadline_key, &deadline);
        extend_persistent_ttl(&env, &deadline_key);

        // Issue #954: emit deadline-set event
        env.events()
            .publish((DEADLINE_SET, pool_id), (_pool.sponsor.clone(), deadline));
    }

    /// Convert a pool's funding deadline (timestamp) to the ledger sequence at
    /// which it elapses, using the current ledger as the reference point.
    fn funding_deadline_ledger(env: &Env, pool: &Pool) -> u32 {
        let now_ts = env.ledger().timestamp();
        let now_seq = env.ledger().sequence();
        if pool.application_deadline <= now_ts {
            return now_seq;
        }
        // 5 seconds per ledger; round up so the funding window is never
        // shortened.
        let secs_remaining = pool.application_deadline - now_ts;
        let ledgers_remaining = ((secs_remaining + 4) / 5) as u32;
        now_seq.saturating_add(ledgers_remaining)
    }

    /// Get the effective refund deadline ledger for a pool.
    ///
    /// Issue #1664: if no explicit refund deadline has been set, the refund
    /// window is derived from the funding deadline plus the grace period, so
    /// every pool has a reachable refund window.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    pub fn get_pool_deadline(env: Env, pool_id: u32) -> u32 {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        let deadline_key = (Symbol::new(&env, POOL_DEADLINE_PREFIX), pool_id);
        if let Some(deadline) = env.storage().persistent().get::<_, u32>(&deadline_key) {
            return deadline;
        }

        Self::funding_deadline_ledger(&env, &pool)
            .saturating_add(REFUND_GRACE_PERIOD_LEDGERS)
    }

    /// Refund a donor's contribution from an expired pool.
    ///
    /// Issue #1664: ledger sequence is the single time base. The effective
    /// refund deadline is the explicit `pool_deadline` if set, otherwise the
    /// pool's funding deadline plus `REFUND_GRACE_PERIOD_LEDGERS`, so every
    /// pool is refundable.
    ///
    /// A refund is only permitted when ALL of the following hold:
    ///   1. The current ledger is strictly after the effective deadline.
    ///   2. The grace period has elapsed
    ///      (`current_ledger >= deadline + REFUND_GRACE_PERIOD_LEDGERS`).
    ///   3. The pool is open and no student funds have been disbursed.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::PoolIsClosed` if the pool is closed
    /// - `ContractError::InvalidPoolState` if the pool is disbursed or a student has claimed funds
    /// - `ContractError::PoolNotExpired` if the deadline has not passed (or grace not elapsed)
    /// - `ContractError::NoContributionToRefund` if the donor has no recorded contribution
    pub fn refund_donation(env: Env, pool_id: u32, donor: Address, token_address: Address) {
        // TODO(#1749): resolve donation token and reject mismatches with TokenTransferFailed
        donor.require_auth();

        let mut pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        let deadline_key = (Symbol::new(&env, POOL_DEADLINE_PREFIX), pool_id);
        let deadline: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&deadline_key)
            .unwrap_or_else(|| {
                Self::funding_deadline_ledger(&env, &pool)
                    .saturating_add(REFUND_GRACE_PERIOD_LEDGERS)
            });

        let current_ledger = env.ledger().sequence();

        // Deadline must have passed AND grace period must have elapsed.
        // Use `checked_add` so a deadline near `u32::MAX` cannot panic the
        // grace-period computation (release builds run with
        // `overflow-checks = true`; see Cargo.toml). On overflow the grace
        // limit is beyond the last representable ledger, so the grace period
        // is treated as elapsed.
        let grace_elapsed = deadline
            .checked_add(REFUND_GRACE_PERIOD_LEDGERS)
            .map_or(true, |grace_limit| current_ledger >= grace_limit);
        if current_ledger <= deadline || !grace_elapsed {
            env.panic_with_error(ContractError::PoolNotExpired);
        }

        if pool.is_closed || pool.state == PoolState::Closed {
            env.panic_with_error(ContractError::PoolIsClosed);
        }

        if pool.state == PoolState::Disbursed || pool_claimed(&env, pool_id) != 0 {
            env.panic_with_error(ContractError::InvalidPoolState);
        }

        let token_key = (Symbol::new(&env, POOL_TOKEN_PREFIX), pool_id);
        if let Some(expected_token) = env.storage().persistent().get::<_, Address>(&token_key) {
            if expected_token != token_address {
                panic!("TokenTransferFailed");
            }
        }

        let contrib_key = (pool_id, "contribution", &donor);
        let contribution: i128 = env
            .storage()
            .persistent()
            .get::<_, i128>(&contrib_key)
            .unwrap_or(0);

        if contribution == 0 {
            env.panic_with_error(ContractError::NoContributionToRefund);
        }

        // Clear the contribution record before transferring (re-entrancy guard)
        env.storage().persistent().set(&contrib_key, &0i128);

        // Reduce pool collected amount
        pool.collected = pool.collected.saturating_sub(contribution);
        env.storage().persistent().set(&pool_id, &pool);

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &donor, &contribution);

        // Issue #954: emit donation-refund event
        env.events()
            .publish((DONATION_REFUND, pool_id), (donor.clone(), contribution));
    }

    /// Donate to a pool using a specific token.
    ///
    /// Transfers tokens from the donor to the contract and updates the pool's
    /// collected amount. The token must match the pool's configured token if one
    /// has been set via [`set_pool_token`].
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::PoolIsClosed` if the pool has been closed
    /// - `ContractError::InvalidPoolState` if the pool is not in Active state
    /// - `ContractError::CampaignExpired` if the current timestamp is at or after the pool's application deadline
    /// - `ContractError::InvalidAmount` if amount is zero or negative
    /// - `ContractError::TokenTransferFailed` if token_address does not match the pool's configured token
    pub fn donate_with_token(
        env: Env,
        pool_id: u32,
        donor: Address,
        token_address: Address,
        amount: i128,
    ) {
        donor.require_auth();

        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        if pool.is_closed {
            env.panic_with_error(ContractError::PoolIsClosed);
        }

        // TODO: Replace with real implementation from issue #XYZ
        // Pool state validation
        if pool.state != PoolState::Active {
            env.panic_with_error(ContractError::InvalidPoolState);
        }

        if env.ledger().timestamp() >= pool.application_deadline {
            env.panic_with_error(ContractError::CampaignExpired);
        }

        if amount <= 0 {
            env.panic_with_error(ContractError::InvalidAmount);
        }

        let token_key = (Symbol::new(&env, POOL_TOKEN_PREFIX), pool_id);
        if let Some(expected_token) = env.storage().persistent().get::<_, Address>(&token_key) {
            if expected_token != token_address {
                env.panic_with_error(ContractError::TokenTransferFailed);
            }
        }

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&donor, &env.current_contract_address(), &amount);

        let new_collected = pool
            .collected
            .checked_add(amount)
            .expect("Collected amount overflow");

        let updated_pool = Pool {
            collected: new_collected,
            last_donation_at: env.ledger().timestamp(),
            ..pool
        };
        env.storage().persistent().set(&pool_id, &updated_pool);

        // Emit contribution event with privacy flag (true = private donation)
        env.events().publish(
            (CONTRIBUTION, pool_id),
            (donor.clone(), amount, new_collected, true), // true = private contribution
        );
        // Track unique donors
        let donor_key = (pool_id, "donor", &donor);
        if !env.storage().persistent().has(&donor_key) {
            env.storage().persistent().set(&donor_key, &true);
            let donor_count: u32 = env
                .storage()
                .persistent()
                .get::<_, u32>(&(pool_id, "d_count"))
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&(pool_id, "d_count"), &(donor_count + 1));
        }

        // Track individual donor's total contribution
        let contrib_key = (pool_id, "contribution", &donor);
        let current_contrib: i128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&contrib_key, &(current_contrib + amount));
    }

    /// Request an emergency withdrawal for a pool.
    ///
    /// Only the currently configured admin may submit this request. The function
    /// stores an `EmergencyWithdrawalRequest` keyed by the pool id, including the
    /// token address, amount, ledger timestamp, and the admin who requested it.
    ///
    /// # Panics
    /// - `ContractError::AdminNotSet` if no admin has been configured
    /// - `ContractError::UnauthorizedAdmin` if the caller is not the stored admin
    /// - `ContractError::PoolNotFound` if the pool does not exist
    /// - `ContractError::PoolIsClosed` if the pool is closed
    /// - `ContractError::InvalidPoolState` if the pool is not `Active`
    /// - `"EmergencyWithdrawalAlreadyRequested"` if a request already exists for the pool
    pub fn request_emergency_withdraw(
        env: Env,
        admin: Address,
        pool_id: u32,
        token_address: Address,
        amount: i128,
    ) {
        admin.require_auth();

        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get::<_, Address>(&admin_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminNotSet));
        if stored_admin != admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }

        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        if pool.is_closed {
            env.panic_with_error(ContractError::PoolIsClosed);
        }

        if pool.state != PoolState::Active {
            env.panic_with_error(ContractError::InvalidPoolState);
        }

        let withdrawal_key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
        if env.storage().persistent().has(&withdrawal_key) {
            panic!("EmergencyWithdrawalAlreadyRequested");
        }

        let request = EmergencyWithdrawalRequest {
            pool_id,
            token_address,
            amount,
            request_timestamp: env.ledger().timestamp(),
            requested_by: admin,
        };
        env.storage().persistent().set(&withdrawal_key, &request);
        env.events().publish(
            (EMERGENCY_WITHDRAWAL_REQUESTED, pool_id),
            (
                request.token_address.clone(),
                request.amount,
                request.requested_by.clone(),
                request.request_timestamp,
            ),
        );
    }

    /// Execute a pending emergency withdrawal after the grace period elapses.
    ///
    /// Any caller may invoke this — not only the original requester — once
    /// `GRACE_PERIOD_SECS` (24 hours) has passed since the request was stored.
    /// Tokens are transferred to `requested_by` and the stored request is removed.
    ///
    /// # Panics
    /// - `ContractError::EmergencyWithdrawalNotRequested` if no request exists for `pool_id`
    /// - `ContractError::GracePeriodNotElapsed` if fewer than `GRACE_PERIOD_SECS` have passed
    pub fn execute_emergency_withdraw(env: Env, pool_id: u32) {
        let withdrawal_key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
        let request: EmergencyWithdrawalRequest = env
            .storage()
            .persistent()
            .get::<_, EmergencyWithdrawalRequest>(&withdrawal_key)
            .unwrap_or_else(|| {
                env.panic_with_error(ContractError::EmergencyWithdrawalNotRequested)
            });

        let current_timestamp = env.ledger().timestamp();
        let time_elapsed = current_timestamp.saturating_sub(request.request_timestamp);

        if time_elapsed < GRACE_PERIOD_SECS {
            env.panic_with_error(ContractError::GracePeriodNotElapsed);
        }

        env.storage().persistent().remove(&withdrawal_key);

        let token_client = token::Client::new(&env, &request.token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &request.requested_by,
            &request.amount,
        );

        env.events().publish(
            (EMERGENCY_WITHDRAWAL_EXECUTED, pool_id),
            (request.token_address.clone(), request.amount, request.requested_by.clone()),
        );
    }

    // Mock function to set pool state for testing
    pub fn set_pool_state(env: Env, pool_id: u32, state: PoolState) {
        let mut pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        let old_state = pool.state.clone();
        pool.state = state.clone();
        env.storage().persistent().set(&pool_id, &pool);

        // Issue #954: emit pool-state-set event
        env.events()
            .publish((POOL_STATE_SET, pool_id), (old_state, state));
    }

    /// Return the funding goal for a campaign (pool).
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` (error #1) if no campaign with `campaign_id` exists.
    pub fn get_campaign_goal(env: Env, campaign_id: u32) -> i128 {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&campaign_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.goal
    }

    /// Create a new donation / sponsorship pool with creation fee deduction.
    ///
    /// If creation fee > 0, fee is transferred from `creator` to the contract and
    /// a `fee_paid` event is emitted.
    /// If creation fee == 0, creation proceeds without token transfer or balance checks.
    pub fn create_pool_with_fee(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: i128,
        application_deadline: u64,
        fee_token: Address,
    ) -> u32 {
        creator.require_auth();

        let fee = Self::get_creation_fee(env.clone());
        if fee > 0 {
            let token_client = token::Client::new(&env, &fee_token);
            token_client.transfer(&creator, &env.current_contract_address(), &fee);

            // Accumulate unclaimed fees
            let unclaimed_key = Symbol::new(&env, UNCLAIMED_FEES);
            let current_unclaimed: i128 = env
                .storage()
                .persistent()
                .get::<_, i128>(&unclaimed_key)
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&unclaimed_key, &(current_unclaimed + fee));

            // Emit fee payment event
            env.events().publish((FEE_PAID,), (creator.clone(), fee));
        }

        Self::create_pool(env, creator, title, description, goal, application_deadline)
    }

    /// Alias for `create_pool_with_fee`.
    pub fn create_campaign_with_fee(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: i128,
        application_deadline: u64,
        fee_token: Address,
    ) -> u32 {
        Self::create_pool_with_fee(
            env,
            creator,
            title,
            description,
            goal,
            application_deadline,
            fee_token,
        )
    }

    /// Configure the global crowdfunding token.
    ///
    /// # Authorization
    /// Only the currently configured admin may call this.
    ///
    /// # Panics
    /// - `ContractError::AdminNotSet` if no admin has been configured
    /// - `ContractError::UnauthorizedAdmin` if `admin` does not match the stored admin
    /// - `"InvalidTokenAddress"` if token address is invalid (e.g. contract's own address)
    pub fn set_crowdfunding_token(env: Env, admin: Address, token: Address) {
        admin.require_auth();

        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get::<_, Address>(&admin_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminNotSet));
        if stored_admin != admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }

        if token == env.current_contract_address() {
            env.panic_with_error(ContractError::InvalidTokenAddress);
        }

        let token_key = Symbol::new(&env, CROWDFUNDING_TOKEN_KEY);
        env.storage().persistent().set(&token_key, &token);

        // Emit event
        env.events().publish((CROWDFUNDING_TOKEN_SET,), token);
    }

    /// Get the currently configured global crowdfunding token.
    ///
    /// # Panics
    /// - `"Crowdfunding token not set"` if `set_crowdfunding_token` has never
    ///   succeeded. This is a plain `.expect()` panic, unlike the typed
    ///   `ContractError::AdminNotSet` used by `set_crowdfunding_token` for its
    ///   own admin lookup.
    ///
    /// Follow-up (issue #1483): replace the `.expect()` with a typed
    /// `ContractError` variant, consistent with `ContractError::AdminNotSet`
    /// and `ContractError::SchoolNotRegistered`, and document the new code in
    /// `nevo_contract/ERRORS.md`. The panic text is locked in by
    /// `test_issue_1483_crowdfunding_token_unset` so that change has to be
    /// deliberate.
    pub fn get_crowdfunding_token(env: Env) -> Address {
        let token_key = Symbol::new(&env, CROWDFUNDING_TOKEN_KEY);
        env.storage()
            .persistent()
            .get::<_, Address>(&token_key)
            .expect("Crowdfunding token not set")
    }

    /// Return the number of ledgers a donor must wait after a pool's deadline
    /// before `refund_donation()` will succeed.
    ///
    /// This corresponds to the compile-time constant `REFUND_GRACE_PERIOD_LEDGERS`
    /// (currently 17 280 ledgers, ≈ 24 hours at a 5-second ledger cadence).
    /// Off-chain integrators should call this getter rather than hard-coding the
    /// value so they remain correct if the constant is ever updated.
    pub fn get_refund_grace_period_ledgers(_env: Env) -> u32 {
        REFUND_GRACE_PERIOD_LEDGERS
    }

    /// Return the number of seconds an admin must wait after
    /// `request_emergency_withdraw()` before `execute_emergency_withdraw()`
    /// will succeed.
    ///
    /// This corresponds to the compile-time constant `GRACE_PERIOD_SECS`
    /// (currently 86 400 seconds, i.e. 24 hours). Off-chain integrators
    /// should call this getter rather than hard-coding the value so they
    /// remain correct if the constant is ever updated.
    pub fn get_emergency_grace_period_secs(_env: Env) -> u64 {
        GRACE_PERIOD_SECS
    }

    /// Return the maximum allowed length for pool descriptions.
    ///
    /// This corresponds to the compile-time constant `MAX_DESCRIPTION_LENGTH`
    /// (currently 500 bytes). Off-chain integrators should call this getter
    /// rather than hard-coding the value so they remain correct if the constant
    /// is ever updated.
    pub fn get_max_description_length(_env: Env) -> u32 {
        MAX_DESCRIPTION_LENGTH as u32
    }

    // ─── Upgrade / Versioning (Issue #1657) ──────────────────────────────────

    /// Return the current contract release version.
    ///
    /// The value is bumped on every release so off-chain clients can detect
    /// which contract build is deployed.
    pub fn version(_env: Env) -> u32 {
        VERSION
    }

    /// Upgrade the contract's WASM code to `new_wasm_hash`.
    ///
    /// Only the platform admin (set via [`Contract::set_admin`]) may call
    /// this. The new code is installed with
    /// `env.deployer().update_current_contract_wasm`, and a
    /// `ContractUpgraded` event carrying the new hash is emitted.
    ///
    /// # Panics
    /// - `ContractError::AdminNotSet` if no admin has been configured
    /// - `ContractError::UnauthorizedAdmin` if `admin` is not the stored admin
    pub fn upgrade(env: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        admin.require_auth();

        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get::<_, Address>(&admin_key)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminNotSet));
        if stored_admin != admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }

        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());

        env.events()
            .publish((CONTRACT_UPGRADED,), new_wasm_hash);
    }
}

mod test;
mod test_campaign_lifecycle;
mod test_contract_initialization;
mod test_issue_1287_pool_multisig;
mod test_issue_1319_memory;
mod test_issue_1480_claim_fee;
mod test_issue_1478_campaign_enumeration;
mod test_issue_1477_pool_school_relationship;
mod test_issue_1467_create_pool_validation_panics;
mod test_issue_1469_milestone_setup_panics;
mod test_issue_1470_get_pool_omits_state;
mod test_issue_1472_application_index_bounds;
mod test_issue_1473_get_application_claimed_amount_consistency;
mod test_issue_1482_pool_getter_missing_pool;
mod test_issue_1483_crowdfunding_token_unset;
mod test_issue_1484_create_campaign_with_fee;
mod test_issue_1485_campaign_balance_alias;
mod test_issue_1487_emergency_withdraw_amount_validation;
mod test_get_max_description_length;
mod test_issues;
mod test_auth_bypass;
mod test_numeric_overflow;
mod test_timestamp_edge_cases;
mod test_token_transfer_errors;
mod test_get_total_raised;
mod test_is_school_registered;
mod test_get_school_metadata;
mod test_create_pool_for_school;
mod test_pool_creation;
mod test_pool_retrieval;
mod test_register_school;
mod test_issue_1455;
mod test_withdraw;
mod test_issue_1460;
mod test_issue_1459;
mod test_issue_1355_pool_state_completeness;
mod test_issue_1368_repeat_donor_aggregation;
mod test_issue_1358_donate_close_claim_ordering;
mod test_issue_1357_emergency_withdrawal_request_lifecycle;
mod test_issue_1371_emergency_withdraw_duplicate_requests;
mod test_issue_1352_set_pool_state_transitions;
mod test_issue_1351_close_pool_pending_claims;
mod test_issue_1353_grace_period_boundaries;
mod test_issue_1377_pool_deadline;
mod test_issue_1462_crowdfunding_token_event;
mod test_issue_1331_withdraw_unallocated_funds;
mod test_issue_1365_event_emission_ordering;

#[cfg(test)]
mod test_pool_disbursement_safety;
#[cfg(test)]
mod test_issue_1750_missing_pool_getters;
mod test_issue_1354_contract_error_variants;
mod test_issue_1356_milestone_struct_validation;
mod test_issue_1359_apply_approve_claim_flow;
mod test_issue_1369_duplicate_school_metadata_hash;
