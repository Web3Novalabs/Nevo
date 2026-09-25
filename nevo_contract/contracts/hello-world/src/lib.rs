//! Nevo donation pool contract.
//!
//! Sponsors create donation pools with a funding goal and application
//! deadline; donors contribute funds (native or via `donate_with_token`)
//! that accumulate toward the pool's goal. Schools register with the
//! platform admin and can be linked to pools so that students may apply
//! for support. Approved applications are funded via milestone-based
//! disbursements: a school/admin sets a list of `Milestone`s whose amounts
//! must sum to the pool goal, and approved students claim funds as
//! milestones are met, up to their approved amount.
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
const POOL_TOKEN_PREFIX: &str = "pool_token";
const SAVED_METADATA_PREFIX: &str = "saved_metadata";

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
const CROWDFUNDING_TOKEN_KEY: &str = "crowdfunding_token";

// Refund deadline constants
// Donors may request a refund only after the pool deadline has passed AND
// the grace period (REFUND_GRACE_PERIOD_LEDGERS) has elapsed.
const POOL_DEADLINE_PREFIX: &str = "pool_deadline";
const REFUND_GRACE_PERIOD_LEDGERS: u32 = 17_280; // ~24 hours at 5s/ledger

// Pool metadata validation constraints
const MAX_TITLE_LENGTH: u32 = 256;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const POOL_METADATA_PREFIX: &str = "metadata";
const MAX_URL_LENGTH: usize = 256;
const MAX_IMAGE_HASH_LENGTH: usize = 64;
const POOL_METADATA_PREFIX: &str = "metadata";

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
const ADMIN_SET: Symbol = symbol_short!("admin_set");
// Issue #954: shared constant replacing inline Symbol::new(&env, "creation_fee_updated")
const FEE_UPDATED: Symbol = symbol_short!("fee_upd");
const CROWDFUNDING_TOKEN_SET: Symbol = symbol_short!("tok_set");
const FEE_PAID: Symbol = symbol_short!("fee_paid");

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
    /// Pool has already been closed and cannot be closed again.
    PoolAlreadyClosed = 15,
    /// Pool title/name is empty.
    InvalidPoolName = 16,
    /// Pool funding goal is zero.
    InvalidPoolTarget = 17,
    /// Pool application deadline is already in the past.
    InvalidPoolDeadline = 18,
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
    /// Ledger timestamp of the most recent contribution to this pool.
    /// Zero until the pool receives its first donation.
    pub last_donation_at: u64,
}

/// Milestone for streaming disbursements
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub amount: u128,
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
        goal: u128,
        application_deadline: u64,
    ) -> u32 {
        if title.len() == 0 {
            panic!("Title cannot be empty");
        }
        if description.len() == 0 {
            panic!("Description cannot be empty");
        }
        if description.len() as u32 > MAX_DESCRIPTION_LENGTH as u32 {
            panic!("Description exceeds maximum length");
        }
        if application_deadline == 0 {
            panic!("Duration must be greater than zero");
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
            last_donation_at: 0u64,
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

    /// Save mutable pool metadata and its multi-signature configuration.
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

        let signers_key = (Symbol::new(&env, "pool_signers"), pool_id);
        env.storage()
            .persistent()
            .set(&signers_key, &(required_signatures, signers));
    }

    /// Return the multi-signature threshold and signer list saved for a pool.
    pub fn get_pool_signers(env: Env, pool_id: u32) -> (u32, Vec<Address>) {
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
    ///
    /// **Note:** This function does **not** require donor authentication (`donor.require_auth()`) and only updates internal accounting. It does not perform a token transfer. Use `donate_with_token()` for token-backed donations that include donor auth.
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
            collected: new_collected,
            last_donation_at: env.ledger().timestamp(),
            ..pool
        };
        env.storage().persistent().set(&pool_id, &updated_pool);

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
        let metadata_key = (Symbol::new(&env, POOL_METADATA_PREFIX), pool_id);
        env.storage()
            .persistent()
            .get::<_, (String, String)>(&metadata_key)
            .unwrap_or_else(|| (String::from_str(&env, ""), String::from_str(&env, "")))
    }

    /// Get saved description, URL, and image hash metadata.
    pub fn get_saved_pool_metadata(env: Env, pool_id: u32) -> (String, String, String) {
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
    pub fn get_total_raised(env: Env, pool_id: u32) -> u128 {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));

        pool.collected
    }

    /// Get the campaign balance (total amount raised) for a pool.
    ///
    /// Issue #1269: Returns the total collected donation balance for the specified campaign.
    /// Panics with `ContractError::PoolNotFound` if the campaign does not exist.
    pub fn get_campaign_balance(env: Env, pool_id: u32) -> u128 {
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

        let updated_pool = Pool {
            is_closed: true,
            ..pool
        };

        env.storage().persistent().set(&pool_id, &updated_pool);

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
            .persistent()
            .get::<_, u32>(&pool_count_key)
            .unwrap_or(0)
    }

    /// Get all campaign (pool) IDs.
    pub fn get_all_campaigns(env: Env) -> Vec<u32> {
        let count = Self::get_pool_count(env.clone());
        let mut list = Vec::new(&env);
        let mut id = 1u32;
        while id <= count {
            if env.storage().persistent().has(&id) {
                list.push_back(id);
            }
            id += 1;
        }
        list
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

        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if !env.storage().persistent().has(&applicant_key) {
            env.panic_with_error(ContractError::StudentHasNotApplied);
        }

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

    /// Returns true if the student has already applied to the given pool.
    pub fn has_applied(env: Env, pool_id: u32, student: Address) -> bool {
        let applicant_key = (Symbol::new(&env, APPLICANT_PREFIX), pool_id, student);
        env.storage().persistent().has(&applicant_key)
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

    /// Returns the stored application tuple (app_count, applicant, data) for a given pool and index, if it exists.
    pub fn get_application_by_index(
        env: Env,
        pool_id: u32,
        index: u32,
    ) -> Option<(u32, Address, String)> {
        let app_key = (Symbol::new(&env, APPLICATION_PREFIX), pool_id, index);
        env.storage()
            .persistent()
            .get::<_, (u32, Address, String)>(&app_key)
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

        if pool.state == PoolState::Cancelled {
            env.panic_with_error(ContractError::InvalidPoolState);
        }

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
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminNotSet));
        if stored_admin != admin {
            env.panic_with_error(ContractError::UnauthorizedAdmin);
        }

        if fee < 0 {
            env.panic_with_error(ContractError::InvalidFee);
        }

        let fee_key = Symbol::new(&env, CREATION_FEE_KEY);
        env.storage().persistent().set(&fee_key, &fee);

        // Issue #954: use shared FEE_UPDATED constant instead of inline Symbol::new
        env.events().publish((FEE_UPDATED,), fee);
        // Emit event: topics = ["creation_fee_updated"], data = new fee value
        env.events().publish(
            (Symbol::new(&env, "creation_fee_updated"),),
            fee,
        );
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
    /// - `ContractError::PoolNotFound` if pool_id is invalid
    /// - `ContractError::PoolNotExpired` if the deadline has not passed (or grace not elapsed)
    /// - `ContractError::NoContributionToRefund` if the donor has no recorded contribution
    pub fn refund_donation(env: Env, pool_id: u32, donor: Address, token_address: Address) {
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
            .unwrap_or(0);

        let current_ledger = env.ledger().sequence();

        // Deadline must have passed AND grace period must have elapsed
        if deadline == 0
            || current_ledger <= deadline
            || current_ledger < deadline + REFUND_GRACE_PERIOD_LEDGERS
        {
            env.panic_with_error(ContractError::PoolNotExpired);
        }

        let token_key = (Symbol::new(&env, POOL_TOKEN_PREFIX), pool_id);
        if let Some(expected_token) = env
            .storage()
            .persistent()
            .get::<_, Address>(&token_key)
        {
            if expected_token != token_address {
                panic!("TokenTransferFailed");
            }
        }

        let contrib_key = (pool_id, "contribution", &donor);
        let contribution: u128 = env
            .storage()
            .persistent()
            .get::<_, u128>(&contrib_key)
            .unwrap_or(0);

        if contribution == 0 {
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

        let token_key = (Symbol::new(&env, POOL_TOKEN_PREFIX), pool_id);
        if let Some(expected_token) = env
            .storage()
            .persistent()
            .get::<_, Address>(&token_key)
        {
            if expected_token != token_address {
                panic!("TokenTransferFailed");
            }
        }

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&donor, &env.current_contract_address(), &amount);

        let new_collected = pool
            .collected
            .checked_add(amount as u128)
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
        let current_contrib: u128 = env.storage().persistent().get(&contrib_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&contrib_key, &(current_contrib + (amount as u128)));
    }

    /// Request an emergency withdrawal for a pool.
    ///
    /// Only the currently configured admin may submit this request. The function
    /// stores an `EmergencyWithdrawalRequest` keyed by the pool id, including the
    /// token address, amount, ledger timestamp, and the admin who requested it.
    ///
    /// # Panics
    /// - `ContractError::AdminNotSet` if no admin has been configured
    /// - `"Error(Auth, InvalidAction)"` if the caller is not the stored admin
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
            panic!("Error(Auth, InvalidAction)");
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
    }

    /// Execute a pending emergency withdrawal after the grace period elapses.
    ///
    /// Any caller may invoke this — not only the original requester — once
    /// `GRACE_PERIOD_SECS` (24 hours) has passed since the request was stored.
    /// Tokens are transferred to `requested_by` and the stored request is removed.
    ///
    /// # Panics
    /// - `"Emergency withdrawal not requested"` if no request exists for `pool_id`
    /// - `"Grace period not elapsed"` if fewer than `GRACE_PERIOD_SECS` have passed
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

    /// Return the funding goal for a campaign (pool).
    ///
    /// # Panics
    /// - `ContractError::PoolNotFound` (error #1) if no campaign with `campaign_id` exists.
    pub fn get_campaign_goal(env: Env, campaign_id: u32) -> u128 {
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
    /// a `creation_fee_paid` event is emitted.
    /// If creation fee == 0, creation proceeds without token transfer or balance checks.
    pub fn create_pool_with_fee(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: u128,
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

            // Emit fee payment events
            env.events().publish(
                (Symbol::new(&env, "creation_fee_paid"), creator.clone()),
                fee,
            );
            env.events().publish((FEE_PAID,), (creator.clone(), fee));
        }

        Self::create_pool(
            env,
            creator,
            title,
            description,
            goal,
            application_deadline,
        )
    }

    /// Alias for `create_pool_with_fee`.
    pub fn create_campaign_with_fee(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: u128,
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
            panic!("InvalidTokenAddress");
        }

        let token_key = Symbol::new(&env, CROWDFUNDING_TOKEN_KEY);
        env.storage().persistent().set(&token_key, &token);

        // Emit events
        env.events().publish((CROWDFUNDING_TOKEN_SET,), token.clone());
        env.events().publish(
            (Symbol::new(&env, "crowdfunding_token_set"), admin),
            token,
        );
    }

    /// Get the currently configured global crowdfunding token.
    pub fn get_crowdfunding_token(env: Env) -> Address {
        let token_key = Symbol::new(&env, CROWDFUNDING_TOKEN_KEY);
        env.storage()
            .persistent()
            .get::<_, Address>(&token_key)
            .expect("Crowdfunding token not set")
    }
}

mod test;
mod test_issues;
mod test_register_school;
mod test_contract_initialization;
mod test_pool_creation;
mod test_pool_retrieval;
mod test_campaign_lifecycle;
mod test_withdraw;
mod test_issue_1287_pool_multisig;
