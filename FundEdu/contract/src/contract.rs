use soroban_sdk::{contract, contractimpl, contracterror, Address, Env, String};

use crate::{
    errors::ContractError,
    storage::{get_application, get_pool, next_pool_id, set_application, set_pool},
    types::{ApplicationStatus, ScholarshipPool},
};

/// Errors returned by FundEduContract entry points.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FundEduError {
    /// Pool name must not be empty.
    InvalidSponsor = 1,
    /// Pool is not in an active state.
    PoolNotActive = 2,
    /// A duplicate application was submitted.
    DuplicateApplication = 3,
    /// Caller is not the authorised validator.
    UnauthorizedValidator = 4,
    /// Application is not in Pending state.
    NotPending = 5,
    /// Application has already been rejected.
    AlreadyRejected = 6,
    /// `target_amount` must be strictly greater than zero.
    InvalidFunding = 7,
    /// No pool exists for the given `pool_id`.
    PoolNotFound = 8,
}

#[contract]
pub struct FundEduContract;

#[contractimpl]
impl FundEduContract {
    /// Create a new scholarship pool.
    ///
    /// # Arguments
    /// * `sponsor`       – Address funding the pool; must sign the transaction.
    /// * `name`          – Human-readable pool name (must not be empty).
    /// * `target_amount` – Total funding goal in token smallest units (must be > 0).
    /// * `token_address` – SAC token used for contributions.
    ///
    /// # Returns
    /// The unique `pool_id` assigned to the new pool (starts at 0, increments by 1).
    ///
    /// # Errors
    /// * [`FundEduError::InvalidSponsor`]  – `name` is empty.
    /// * [`FundEduError::InvalidFunding`]  – `target_amount` ≤ 0.
    pub fn create_pool(
        env: Env,
        sponsor: Address,
        name: String,
        target_amount: i128,
        token_address: Address,
    ) -> Result<u64, FundEduError> {
        // ── Validate inputs before touching auth or storage ──────────────────
        if name.is_empty() {
            return Err(FundEduError::InvalidSponsor);
        }

        if target_amount <= 0 {
            return Err(FundEduError::InvalidFunding);
        }

        // ── Require sponsor authorisation ────────────────────────────────────
        sponsor.require_auth();

        // ── Allocate ID, build struct, persist ───────────────────────────────
        let pool_id = next_pool_id(&env);

        let pool = ScholarshipPool {
            name,
            sponsor,
            target_amount,
            token_address,
            is_active: true,
        };

        set_pool(&env, pool_id, &pool);

        Ok(pool_id)
    }

    /// Retrieve a scholarship pool by its id. Returns `None` if not found.
    pub fn get_pool(env: Env, pool_id: u64) -> Option<ScholarshipPool> {
        get_pool(&env, pool_id)
    }

    /// Claim awarded scholarship funds.
    /// Follows Check-Effects-Interactions (CEI) pattern.
    pub fn claim_funds(
        env: Env,
        pool_id: u64,
        student: Address,
        amount: i128,
    ) -> Result<(), ContractError> {
        student.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        // 1. Checks
        let pool = get_pool(&env, pool_id).ok_or(ContractError::PoolNotFound)?;
        if !pool.is_active {
            return Err(ContractError::PoolNotActive);
        }

        let mut app = get_application(&env, pool_id, student.clone())
            .ok_or(ContractError::ApplicationNotFound)?;

        if app.status != ApplicationStatus::Approved {
            return Err(ContractError::NotApproved);
        }

        if app.amount_claimed + amount > app.total_granted {
            return Err(ContractError::ExceedsGrant);
        }

        // 2. Effects (Update state BEFORE interaction)
        app.amount_claimed += amount;
        set_application(&env, pool_id, student.clone(), &app);

        // 3. Interactions (External call after state update)
        let token_client = token::Client::new(&env, &pool.token_address);
        token_client.transfer(&env.current_contract_address(), &student, &amount);

        Ok(())
    }
}
