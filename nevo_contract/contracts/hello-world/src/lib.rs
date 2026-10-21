#![cfg_attr(not(test), no_std)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, BytesN,
    Env, String, Symbol, Vec,
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

// TODO: Replace with real implementation from issue #XYZ
// Emergency withdrawal storage keys
const EMERGENCY_WITHDRAWAL_PREFIX: &str = "emergency_withdraw";
const GRACE_PERIOD_SECS: u64 = 86400; // 24 hours

// Application and claim tracking constants
const APPLICATION_STATUS_PREFIX: &str = "app_status";
const CLAIMED_AMOUNT_PREFIX: &str = "claimed_amount";
const APPLICATION_STATUS_APPROVED: &str = "Approved";
const APPLICATION_STATUS_REJECTED: &str = "Rejected";

// Protocol fees accumulator - tracks unclaimed fees collected from operations
const UNCLAIMED_FEES: &str = "unclaimed_fees";

// Creation fee key - stores the fee charged when creating a new pool
const CREATION_FEE_KEY: &str = "creation_fee";

// Refund deadline constants
// Donors may request a refund only after the pool deadline has passed AND
// the grace period (REFUND_GRACE_PERIOD_LEDGERS) has elapsed.
const POOL_DEADLINE_PREFIX: &str = "pool_deadline";
const REFUND_GRACE_PERIOD_LEDGERS: u32 = 17_280; // ~24 hours at 5s/ledger

// Pool metadata validation constraints
const MAX_DESCRIPTION_LENGTH: usize = 500;
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
const MILESTONES_SET: Symbol = symbol_short!("mile_set");
const FUNDS_CLAIMED: Symbol = symbol_short!("fund_clmd");
const FEES_CLAIMED: Symbol = symbol_short!("fees_clmd");
const DONATION_REFUND: Symbol = symbol_short!("don_refnd");
const DEADLINE_SET: Symbol = symbol_short!("ddln_set");
const POOL_STATE_SET: Symbol = symbol_short!("pool_stat");
const SCHOOL_REG: Symbol = symbol_short!("schl_reg");
const ADMIN_SET: Symbol = symbol_short!("admin_set");
// Issue #954: shared constant replacing inline Symbol::new(&env, "creation_fee_updated")
const FEE_UPDATED: Symbol = symbol_short!("fee_upd");

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

/// Tracks a student's approved funding and how much has been streamed so far.
///
/// `amount_claimed` starts at zero and increments with each partial withdrawal,
/// allowing the contract to enforce the invariant:
///   amount_claimed + new_claim <= approved_amount
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Application {
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
/// - `donate()`: Validates pool is `Active` (rejects if `Closed`)
/// - `close_pool()`: Requires pool to be `Disbursed` or `Cancelled` before closing
///
/// # Validation Rules
///
/// - `donate()` only accepts donations if state is `Active`
/// - `close_pool()` only allows closing from `Disbursed` or `Cancelled` states
/// - Other state transitions are not yet implemented (see TODO comments)
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
    pub sponsor: Address,
    pub goal: u128,
    pub collected: u128,
    pub is_closed: bool,
    pub state: PoolState,
    pub application_deadline: u64,
}

/// Milestone for streaming disbursements
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub amount: u128,
}

// TODO: Replace with real implementation from issue #XYZ
// Emergency withdrawal request structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyWithdrawalRequest {
    pub pool_id: u32,
    pub token_address: Address,
    pub amount: i128,
    pub request_timestamp: u64,
    pub requested_by: Address,
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    /// Set the platform admin address.
    pub fn set_admin(env: Env, admin: Address) {
        admin.require_auth();
        let admin_key = Symbol::new(&env, ADMIN_KEY);
        env.storage().persistent().set(&admin_key, &admin);

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
            .persistent()
            .get::<_, Address>(&admin_key)
            .expect("Admin not set");

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
            .expect("School not registered")
    }

    // ─── Pool Management ─────────────────────────────────────────────────────

    /// Create a new donation / sponsorship pool.
    pub fn create_pool(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: u128,
        application_deadline: u64,
    ) -> u32 {
        if description.len() as u32 > MAX_DESCRIPTION_LENGTH as u32 {
            panic!("Description exceeds maximum length");
        }

        let pool_count_key = Symbol::new(&env, POOL_COUNT);
        let mut pool_count: u32 = env
            .storage()
            .persistent()
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
            collected: 0u128,
            is_closed: false,
            state: PoolState::Active,
            application_deadline,
        };

        env.storage().persistent().set(&pool_id, &pool);

        env.storage().persistent().set(&pool_count_key, &pool_count);

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
        goal: u128,
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
        pool_id
    }

    /// Get the school linked to a pool.
    pub fn get_pool_school(env: Env, pool_id: u32) -> Address {
        let pool_school_key = (Symbol::new(&env, POOL_SCHOOL_PREFIX), pool_id);
        env.storage()
            .persistent()
            .get::<_, Address>(&pool_school_key)
            .expect("Pool school not set")
    }

    /// Donate to an existing pool.
    pub fn donate(env: Env, pool_id: u32, donor: Address, amount: u128) {
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

        let new_collected = pool.collected + amount;
        let updated_pool = Pool {
            sponsor: pool.sponsor,
            goal: pool.goal,
            collected: new_collected,
            is_closed: pool.is_closed,
            state: pool.state,
            application_deadline: pool.application_deadline,
        };
        env.storage().persistent().set(&pool_id, &updated_pool);

        let donor_index: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&(pool_id, "d_count"))
            .unwrap_or(0);
        let _ = donor;
        env.storage()
            .persistent()
            .set(&(pool_id, "d_count"), &(donor_index + 1));

        // Emit donation event
        env.events().publish(
            (DONATION_MADE, pool_id),
            (donor.clone(), amount, new_collected),
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
        let current_contrib: u128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&contrib_key, &(current_contrib + amount));
    }

    /// Get pool information as a tuple (id, creator, goal, collected, is_closed).
    pub fn get_pool(env: Env, pool_id: u32) -> (u32, Address, u128, u128, bool, u64) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

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
    /// Returns empty strings if the pool or metadata does not exist.
    pub fn get_pool_metadata(env: Env, pool_id: u32) -> (String, String) {
        let metadata_key = (Symbol::new(&env, "metadata"), pool_id);
        env.storage()
            .persistent()
            .get::<_, (String, String)>(&metadata_key)
            .unwrap_or_else(|| (String::from_str(&env, ""), String::from_str(&env, "")))
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
    pub fn get_total_raised(env: Env, pool_id: u32) -> u128 {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.collected
    }

    /// Close a donation pool.
    pub fn close_pool(env: Env, pool_id: u32) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.sponsor.require_auth();

        if pool.state != PoolState::Disbursed && pool.state != PoolState::Cancelled {
            env.panic_with_error(ContractError::PoolNotDisbursedOrRefunded);
        }

        let updated_pool = Pool {
            sponsor: pool.sponsor,
            goal: pool.goal,
            collected: pool.collected,
            is_closed: true,
            state: pool.state,
            application_deadline: pool.application_deadline,
        };

        env.storage().persistent().set(&pool_id, &updated_pool);

        // Emit pool closed event
        env.events().publish(
            (POOL_CLOSED, pool_id),
            (updated_pool.sponsor.clone(), updated_pool.collected),
        );
    }

    /// Get the total number of pools.
    pub fn get_pool_count(env: Env) -> u32 {
        let pool_count_key = Symbol::new(&env, POOL_COUNT);
        env.storage()
            .persistent()
            .get::<_, u32>(&pool_count_key)
            .unwrap_or(0)
    }

    /// Get the number of unique donors for a pool.
    pub fn get_donor_count(env: Env, pool_id: u32) -> u32 {
        // Verify the pool exists first
        let _pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        env.storage()
            .persistent()
            .get::<_, u32>(&(pool_id, "d_count"))
            .unwrap_or(0)
    }

    /// Get the total contribution of a specific donor to a specific pool.
    pub fn get_contribution(env: Env, pool_id: u32, donor: Address) -> u128 {
        // Verify the pool exists first
        let _pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        env.storage()
            .persistent()
            .get::<_, u128>(&(pool_id, "contribution", &donor))
            .unwrap_or(0)
    }

    /// Student applies to a school-linked pool.
    pub fn apply_to_pool(env: Env, pool_id: u32, student: Address, application_data: String) {
        student.require_auth();

        let _: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

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
        env.storage()
            .persistent()
            .set(&app_key, &(app_count, student.clone(), application_data));

        env.storage().persistent().set(&applicant_key, &true);
        env.storage().persistent().set(&count_key, &app_count);

        let pending = String::from_str(&env, "Pending");
        Self::set_application_status(env.clone(), pool_id, student.clone(), pending);

        // Emit application/contribution event with privacy flag (default: false for public)
        env.events().publish(
            (APPLICATION_SUBMITTED, pool_id),
            (student.clone(), app_count, false), // false = public application
        );
    }

    /// School approves or rejects a student's application.
    pub fn approve_application(
        env: Env,
        pool_id: u32,
        school: Address,
        student: Address,
        approved: bool,
    ) {
        school.require_auth();

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
            String::from_str(&env, APPLICATION_STATUS_APPROVED)
        } else {
            String::from_str(&env, APPLICATION_STATUS_REJECTED)
        };
        Self::set_application_status(env.clone(), pool_id, student.clone(), status);

        // Issue #954: emit application-approved event
        env.events()
            .publish((APP_APPROVED, pool_id), (student.clone(), approved));
    }

    /// Set application milestones and enforce sum(amounts) == pool goal.
    pub fn setup_application_milestones(
        env: Env,
        pool_id: u32,
        student: Address,
        milestones: Vec<Milestone>,
    ) {
        student.require_auth();

        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        if milestones.is_empty() {
            panic!("Milestones required");
        }

        let mut sum: u128 = 0;
        for i in 0..milestones.len() {
            sum = sum
                .checked_add(milestones.get(i).unwrap().amount)
                .expect("Milestone amount overflow");
        }

        if sum != pool.goal {
            panic!("Milestone total must equal pool goal");
        }

        let milestones_key = (Symbol::new(&env, MILESTONES_PREFIX), pool_id, student.clone());
        env.storage().persistent().set(&milestones_key, &milestones);

        // Issue #954: emit milestones-set event
        env.events()
            .publish((MILESTONES_SET, pool_id), (student.clone(), milestones.len()));
    }

    /// Get student milestones for a pool.
    pub fn get_milestones(env: Env, pool_id: u32, student: Address) -> Vec<Milestone> {
        let milestones_key = (Symbol::new(&env, MILESTONES_PREFIX), pool_id, student);
        env.storage()
            .persistent()
            .get::<_, Vec<Milestone>>(&milestones_key)
            .unwrap_or(Vec::new(&env))
    }

    /// Set application status for a student in a pool.
    pub fn set_application_status(env: Env, pool_id: u32, student: Address, status: String) {
        let status_key = (
            Symbol::new(&env, APPLICATION_STATUS_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage().persistent().set(&status_key, &status);
    }

    /// Get application status for a student in a pool.
    pub fn get_application_status(env: Env, pool_id: u32, student: Address) -> String {
        let status_key = (
            Symbol::new(&env, APPLICATION_STATUS_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage()
            .persistent()
            .get::<_, String>(&status_key)
            .unwrap_or(String::from_str(&env, ""))
    }

    /// Get claimed amount for a student in a pool.
    pub fn get_claimed_amount(env: Env, pool_id: u32, student: Address) -> i128 {
        let claimed_key = (
            Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        // Try to get the Application struct first
        let app: Option<Application> = env.storage().persistent().get(&claimed_key);
        match app {
            Some(application) => application.amount_claimed,
            None => 0,
        }
    }

    /// Get the full Application record for a student in a pool.
    /// Returns `None` if the student has not yet made any claim.
    pub fn get_application(env: Env, pool_id: u32, student: Address) -> Option<Application> {
        let app_key = (
            Symbol::new(&env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage().persistent().get::<_, Application>(&app_key)
    }

    /// Withdraw surplus funds not locked by active applications.
    ///
    /// Locked funds = sum of (approved_amount - amount_claimed) for every
    /// application whose status is "Approved" or "Pending".
    /// Surplus = pool.collected - locked_funds.
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `"Insolvency: locked funds exceed collected"` if locked > collected
    /// - `"No surplus to withdraw"` if surplus == 0
    pub fn withdraw_unallocated_funds(env: Env, pool_id: u32, token_address: Address) {
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

        let approved_str = String::from_str(&env, APPLICATION_STATUS_APPROVED);
        let pending_str = String::from_str(&env, "Pending");

        let mut locked: u128 = 0u128;
        for idx in 1..=app_count {
            let app_key = (Symbol::new(&env, APPLICATION_PREFIX), pool_id, idx);
            let entry: Option<(u32, Address, soroban_sdk::String)> =
                env.storage().persistent().get(&app_key);
            if let Some((_, student, _)) = entry {
                let status_key = (
                    Symbol::new(&env, APPLICATION_STATUS_PREFIX),
                    pool_id,
                    student.clone(),
                );
                let status: String = env
                    .storage()
                    .persistent()
                    .get::<_, String>(&status_key)
                    .unwrap_or(String::from_str(&env, ""));

                if status == approved_str || status == pending_str {
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
                            approved_amount: 0,
                            amount_claimed: 0,
                        });
                    let remaining =
                        (application.approved_amount - application.amount_claimed).max(0) as u128;
                    locked = locked
                        .checked_add(remaining)
                        .expect("Locked funds overflow");
                }
            }
        }

        let surplus: u128 = pool
            .collected
            .checked_sub(locked)
            .expect("Insolvency: locked funds exceed collected");

        if surplus == 0 {
            panic!("No surplus to withdraw");
        }

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &pool.sponsor,
            &(surplus as i128),
        );

        pool.collected -= surplus;
        env.storage().persistent().set(&pool_id, &pool);
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
    /// - `"Claim amount must be positive"` if `claim_amount <= 0`
    /// - `"Application status not found"` if no status has been set
    /// - `"Application is not approved"` if status != "Approved"
    /// - `"Overdraw attempt"` if `amount_claimed + claim_amount > collected`
    pub fn claim_funds(
        env: Env,
        student: Address,
        pool_id: u32,
        claim_amount: i128,
        token_address: Address,
    ) {
        student.require_auth();

        if claim_amount <= 0 {
            panic!("Claim amount must be positive");
        }

        // Verify application is approved
        let status_key = (
            Symbol::new(&env, APPLICATION_STATUS_PREFIX),
            pool_id,
            student.clone(),
        );
        let status: String = env
            .storage()
            .persistent()
            .get::<_, String>(&status_key)
            .unwrap_or_else(|| panic!("Application status not found"));

        if status != String::from_str(&env, APPLICATION_STATUS_APPROVED) {
            panic!("Application is not approved");
        }

        // Load pool to check available collected funds
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        let collected = pool.collected as i128;

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
                approved_amount: collected,
                amount_claimed: 0,
            });

        // Enforce the partial-payment invariant
        if application.amount_claimed + claim_amount > collected {
            panic!("Overdraw attempt");
        }

        // Accumulate protocol fees (1% of claim amount)
        // Fee tracking is isolated from student allocations
        let fee = claim_amount / 100;
        let net_transfer = claim_amount - fee;

        // Disburse tokens to the student
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &student, &net_transfer);
        let unclaimed_fees_key = Symbol::new(&env, UNCLAIMED_FEES);
        let mut current_fees: i128 = env
            .storage()
            .persistent()
            .get::<_, i128>(&unclaimed_fees_key)
            .unwrap_or(0);
        current_fees += fee;
        env.storage()
            .persistent()
            .set(&unclaimed_fees_key, &current_fees);

        // Persist the updated running total
        application.amount_claimed += claim_amount;
        env.storage().persistent().set(&app_key, &application);

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
            .persistent()
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

        // Issue #954: emit fees-claimed event
        env.events()
            .publish((FEES_CLAIMED, admin.clone()), (fees,));

        fees
    }

    // ─── Creation Fee ─────────────────────────────────────────────────────────

    /// Set the pool creation fee (in stroops / smallest token unit).
    ///
    /// Only the stored admin may call this function.
    /// A fee of zero is valid (disables the creation fee).
    /// A negative fee panics with `"InvalidFee"`.
    ///
    /// Emits a `creation_fee_updated` event on success.
    ///
    /// # Panics
    /// - `"Admin not set"` if no admin has been configured
    /// - `"Unauthorized admin"` if `admin` does not match the stored admin
    /// - `"InvalidFee"` if `fee` is negative
    /// A negative fee panics with `ContractError::InvalidFee`.
    ///
    /// Emits a `FEE_UPDATED` event on success.
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
            .persistent()
            .get::<_, Address>(&admin_key)
            .expect("Admin not set");
        if stored_admin != admin {
            panic!("Unauthorized admin");
        }

        if fee < 0 {
            panic!("InvalidFee");
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminNotSet));
        if stored_admin != admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }

        if fee < 0 {
            env.panic_with_error(ContractError::InvalidFee);
        }

        let fee_key = Symbol::new(&env, CREATION_FEE_KEY);
        env.storage().persistent().set(&fee_key, &fee);

        // Emit event: topics = ["creation_fee_updated"], data = new fee value
        env.events().publish(
            (Symbol::new(&env, "creation_fee_updated"),),
            fee,
        );
        // Issue #954: use shared FEE_UPDATED constant instead of inline Symbol::new
        env.events().publish((FEE_UPDATED,), fee);
    }

    /// Get the current pool creation fee.
    /// Returns `0` if no fee has been set.
    pub fn get_creation_fee(env: Env) -> i128 {
        let fee_key = Symbol::new(&env, CREATION_FEE_KEY);
        env.storage()
            .persistent()
            .get::<_, i128>(&fee_key)
            .unwrap_or(0)
    }

    // ─── Refund Deadline ──────────────────────────────────────────────────────

    /// Set the refund deadline (as a ledger sequence number) for a pool.
    ///
    /// Only the pool sponsor may call this.
    /// The deadline must be in the future (greater than the current ledger).
    ///
    /// # Panics
    /// - `"Pool not found"` if pool_id is invalid
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `"Error(Auth, InvalidAction)"` if caller is not the pool sponsor
    /// - `"Deadline must be in the future"` if deadline <= current ledger
    pub fn set_pool_deadline(env: Env, pool_id: u32, deadline: u32) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .expect("Pool not found");
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.sponsor.require_auth();

        if deadline <= env.ledger().sequence() {
            panic!("Deadline must be in the future");
        }

        let deadline_key = (Symbol::new(&env, POOL_DEADLINE_PREFIX), pool_id);
        env.storage().persistent().set(&deadline_key, &deadline);

        // Issue #954: emit deadline-set event
        env.events()
            .publish((DEADLINE_SET, pool_id), (pool.sponsor.clone(), deadline));
    }

    /// Get the refund deadline ledger for a pool.
    /// Returns `0` if no deadline has been set.
    pub fn get_pool_deadline(env: Env, pool_id: u32) -> u32 {
        let deadline_key = (Symbol::new(&env, POOL_DEADLINE_PREFIX), pool_id);
        env.storage()
            .persistent()
            .get::<_, u32>(&deadline_key)
            .unwrap_or(0)
    }

    /// Refund a donor's contribution from an expired pool.
    ///
    /// A refund is only permitted when ALL of the following hold:
    ///   1. The pool has a deadline set (non-zero).
    ///   2. The current ledger is strictly after the deadline
    ///      (`current_ledger > deadline`).
    ///   3. The grace period has elapsed
    ///      (`current_ledger >= deadline + REFUND_GRACE_PERIOD_LEDGERS`).
    ///
    /// # Panics
    /// - `"Pool not found"` if pool_id is invalid
    /// - `"PoolNotExpired"` if the deadline has not passed yet
    /// - `"PoolNotExpired"` if the pool is exactly at the deadline (no grace)
    /// - `"PoolNotExpired"` if inside the grace period
    /// - `"No contribution to refund"` if the donor has no recorded contribution
    pub fn refund_donation(
        env: Env,
        pool_id: u32,
        donor: Address,
        token_address: Address,
    ) {
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::PoolNotExpired` if the deadline has not passed (or grace not elapsed)
    /// - `ContractError::NoContributionToRefund` if the donor has no recorded contribution
    pub fn refund_donation(env: Env, pool_id: u32, donor: Address, token_address: Address) {
        donor.require_auth();

        let mut pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .expect("Pool not found");
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        let deadline_key = (Symbol::new(&env, POOL_DEADLINE_PREFIX), pool_id);
        let deadline: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&deadline_key)
            .unwrap_or(0);

        let current_ledger = env.ledger().sequence();

        // Deadline must have passed AND grace period must have elapsed
        if deadline == 0
            || current_ledger <= deadline
            || current_ledger < deadline + REFUND_GRACE_PERIOD_LEDGERS
        {
            panic!("PoolNotExpired");
            env.panic_with_error(ContractError::PoolNotExpired);
        }

        let contrib_key = (pool_id, "contribution", &donor);
        let contribution: u128 = env
            .storage()
            .persistent()
            .get::<_, u128>(&contrib_key)
            .unwrap_or(0);

        if contribution == 0 {
            panic!("No contribution to refund");
            env.panic_with_error(ContractError::NoContributionToRefund);
        }

        // Clear the contribution record before transferring (re-entrancy guard)
        env.storage().persistent().set(&contrib_key, &0u128);

        // Reduce pool collected amount
        pool.collected = pool.collected.saturating_sub(contribution);
        env.storage().persistent().set(&pool_id, &pool);

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &donor,
            &(contribution as i128),
        );

        // Issue #954: emit donation-refund event
        env.events()
            .publish((DONATION_REFUND, pool_id), (donor.clone(), contribution));
    }

    /// Donate to a pool using a specific token.
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

        if amount <= 0 {
            panic!("InvalidAmount");
        }

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&donor, &env.current_contract_address(), &amount);

        let new_collected = pool
            .collected
            .checked_add(amount as u128)
            .expect("Collected amount overflow");

        let updated_pool = Pool {
            sponsor: pool.sponsor,
            goal: pool.goal,
            collected: new_collected,
            is_closed: pool.is_closed,
            state: pool.state,
            application_deadline: pool.application_deadline,
        };
        env.storage().persistent().set(&pool_id, &updated_pool);

        let donor_index: u32 = env
            .storage()
            .persistent()
            .get::<_, u32>(&(pool_id, "d_count"))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&(pool_id, "d_count"), &(donor_index + 1));

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
        let current_contrib: u128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&contrib_key, &(current_contrib + (amount as u128)));
    }

    // TODO: Replace with real implementation from issue #XYZ
    // Mock emergency withdrawal request function
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
            panic!("Error(Auth, InvalidAction)");
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
    }

    // TODO: Replace with real implementation from issue #XYZ
    // Mock emergency withdrawal execution function
    pub fn execute_emergency_withdraw(env: Env, pool_id: u32) {
        let withdrawal_key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool_id);
        let request: EmergencyWithdrawalRequest = env
            .storage()
            .persistent()
            .get::<_, EmergencyWithdrawalRequest>(&withdrawal_key)
            .expect("Emergency withdrawal not requested");

        let current_timestamp = env.ledger().timestamp();
        let time_elapsed = current_timestamp.saturating_sub(request.request_timestamp);

        if time_elapsed < GRACE_PERIOD_SECS {
            panic!("Grace period not elapsed");
        }

        let token_client = token::Client::new(&env, &request.token_address);
        token_client.transfer(
            &env.current_contract_address(),
            &request.requested_by,
            &request.amount,
        );

        env.storage().persistent().remove(&withdrawal_key);
    }

    // TODO: Replace with real implementation from issue #XYZ
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
}

mod test;
mod test_issues;
mod test_register_school;
mod test_issue_1455;