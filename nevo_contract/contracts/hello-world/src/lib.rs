#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, String, Symbol, Vec, symbol_short};

// Storage key constants
const POOL_COUNT: &str = "pool_count";
const POOL_PREFIX: &str = "p";
const CREATOR_SUFFIX: &str = "_creator";
const GOAL_SUFFIX: &str = "_goal";
const COLLECTED_SUFFIX: &str = "_collected";
const CLOSED_SUFFIX: &str = "_closed";
const APPLICATION_COUNT_PREFIX: &str = "a_count_";
const APPLICATION_PREFIX: &str = "a_";
const APPLICANT_PREFIX: &str = "ap_";
const MILESTONES_PREFIX: &str = "milestones";
const ADMIN_KEY: &str = "admin";
const SCHOOL_REG_PREFIX: &str = "school_reg";
const POOL_SCHOOL_PREFIX: &str = "pool_school";

// Application and claim tracking constants
const APPLICATION_STATUS_PREFIX: &str = "app_status";
const CLAIMED_AMOUNT_PREFIX: &str = "claimed_amount";
const APPLICATION_STATUS_APPROVED: &str = "Approved";
const APPLICATION_STATUS_REJECTED: &str = "Rejected";

// Protocol fees accumulator - tracks unclaimed fees collected from operations
const UNCLAIMED_FEES: &str = "unclaimed_fees";

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

/// Pool information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pool {
    pub sponsor: Address,
    pub goal: u128,
    pub collected: u128,
    pub is_closed: bool,
}

/// Milestone for streaming disbursements
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Milestone {
    pub amount: u128,
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
    }

    /// Register a school by admin authorization.
    pub fn register_school(env: Env, admin: Address, school: Address) {
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

        let school_key = (Symbol::new(&env, SCHOOL_REG_PREFIX), school);
        env.storage().persistent().set(&school_key, &true);
    }

    /// Check if a school has been registered.
    pub fn is_school_registered(env: Env, school: Address) -> bool {
        let school_key = (Symbol::new(&env, SCHOOL_REG_PREFIX), school);
        env.storage()
            .persistent()
            .get::<_, bool>(&school_key)
            .unwrap_or(false)
    }

    // ─── Pool Management ─────────────────────────────────────────────────────

    /// Create a new donation / sponsorship pool.
    pub fn create_pool(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        goal: u128,
    ) -> u32 {
        if description.len() > MAX_DESCRIPTION_LENGTH {
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

        // Legacy compatibility: keep old symbolic key constants reachable.
        let _ = (
            POOL_PREFIX,
            CREATOR_SUFFIX,
            GOAL_SUFFIX,
            COLLECTED_SUFFIX,
            CLOSED_SUFFIX,
        );

        let metadata_key = (Symbol::new(&env, "metadata"), pool_id);
        env.storage().persistent().set(&metadata_key, &(title, description));

        let pool = Pool {
            sponsor: creator.clone(),
            goal,
            collected: 0u128,
            is_closed: false,
        };

        env.storage().persistent().set(&pool_id, &pool);

        env.storage().persistent().set(&pool_count_key, &pool_count);

        // Emit pool creation event
        env.events().publish(
            (POOL_CREATED, pool_id),
            (creator.clone(), goal, title, description)
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
    ) -> u32 {
        creator.require_auth();

        if !Self::is_school_registered(env.clone(), school.clone()) {
            panic!("School is not registered");
        }

        let pool_id = Self::create_pool(env.clone(), creator, title, description, goal);
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
            .expect("Pool not found");

        if pool.is_closed {
            panic!("Pool is closed");
        }

        let new_collected = pool.collected + amount;
        let updated_pool = Pool {
            sponsor: pool.sponsor,
            goal: pool.goal,
            collected: new_collected,
            is_closed: pool.is_closed,
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
            (donor.clone(), amount, new_collected)
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
        env.storage().persistent().set(&contrib_key, &(current_contrib + amount));
    }

    /// Get pool information as a tuple (id, creator, goal, collected, is_closed).
    pub fn get_pool(env: Env, pool_id: u32) -> (u32, Address, u128, u128, bool) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .expect("Pool not found");

        (
            pool_id,
            pool.sponsor,
            pool.goal,
            pool.collected,
            pool.is_closed,
        )
    }

    /// Get pool metadata as a tuple (title, description).
    /// Returns empty strings if the pool or metadata does not exist.
    pub fn get_pool_metadata(env: Env, pool_id: u32) -> (String, String) {
        let metadata_key = (Symbol::new(&env, "metadata"), pool_id);
        env.storage()
            .persistent()
            .get::<_, (String, String)>(&metadata_key)
            .unwrap_or_else(|| {
                (
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
            .expect("Pool not found");

        pool.collected
    }

    /// Close a donation pool.
    pub fn close_pool(env: Env, pool_id: u32) {
        let pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .expect("Pool not found");

        pool.sponsor.require_auth();

        let updated_pool = Pool {
            sponsor: pool.sponsor,
            goal: pool.goal,
            collected: pool.collected,
            is_closed: true,
        };

        env.storage().persistent().set(&pool_id, &updated_pool);

        // Emit pool closed event
        env.events().publish(
            (POOL_CLOSED, pool_id),
            (pool.sponsor.clone(), pool.collected)
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
            .expect("Pool not found");
        
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
            .expect("Pool not found");
        
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
            .expect("Pool not found");

        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if env.storage().persistent().has(&applicant_key) {
            panic!("Duplicate application");
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
            (student.clone(), app_count, false) // false = public application
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
            panic!("Only linked school can approve");
        }

        let applicant_key = (
            Symbol::new(&env, APPLICANT_PREFIX),
            pool_id,
            student.clone(),
        );
        if !env.storage().persistent().has(&applicant_key) {
            panic!("Student has not applied");
        }

        let status = if approved {
            String::from_str(&env, APPLICATION_STATUS_APPROVED)
        } else {
            String::from_str(&env, APPLICATION_STATUS_REJECTED)
        };
        Self::set_application_status(env, pool_id, student, status);
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
            .expect("Pool not found");

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

        let milestones_key = (Symbol::new(&env, MILESTONES_PREFIX), pool_id, student);
        env.storage().persistent().set(&milestones_key, &milestones);
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
    /// - `"Pool not found"` if pool_id is invalid
    /// - `"Insolvency: locked funds exceed collected"` if locked > collected
    /// - `"No surplus to withdraw"` if surplus == 0
    pub fn withdraw_unallocated_funds(env: Env, pool_id: u32, token_address: Address) {
        let mut pool: Pool = env
            .storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .expect("Pool not found");

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
                    let claim_key = (CLAIMED_AMOUNT_PREFIX, pool_id, student.clone());
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
            .expect("Pool not found");

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
    /// - `"Unauthorized admin"` if the caller is not the stored admin address
    /// - `"No unclaimed fees"` if there are no accumulated fees to claim
    pub fn claim_protocol_fees(env: Env, admin: Address, token_address: Address) -> i128 {
        admin.require_auth();

        // Verify caller is the protocol admin
        let admin_key = Symbol::new(&env, ADMIN_KEY);
        let stored_admin: Address = env
            .storage()
            .persistent()
            .get::<_, Address>(&admin_key)
            .expect("Admin not set");
        if stored_admin != admin {
            panic!("Unauthorized admin");
        }

        // Get accumulated unclaimed fees
        let unclaimed_fees_key = Symbol::new(&env, UNCLAIMED_FEES);
        let fees: i128 = env
            .storage()
            .persistent()
            .get::<_, i128>(&unclaimed_fees_key)
            .unwrap_or(0);

        if fees == 0 {
            panic!("No unclaimed fees");
        }

        // Transfer accumulated fees to admin
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&env.current_contract_address(), &admin, &fees);

        // Reset unclaimed fees to 0
        env.storage().persistent().set(&unclaimed_fees_key, &0i128);

        fees
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
            .expect("Pool not found");

        if pool.is_closed {
            panic!("Pool is closed");
        }

        if amount <= 0 {
            panic!("Amount must be positive");
        }

        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&donor, &env.current_contract_address(), &amount);

        let new_collected = pool.collected.checked_add(amount as u128)
            .expect("Collected amount overflow");

        let updated_pool = Pool {
            sponsor: pool.sponsor,
            goal: pool.goal,
            collected: new_collected,
            is_closed: pool.is_closed,
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
            (donor.clone(), amount, new_collected, true) // true = private contribution
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
        env.storage().persistent().set(&contrib_key, &(current_contrib + (amount as u128)));
    }
}

mod test;
