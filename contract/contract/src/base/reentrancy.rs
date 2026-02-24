use crate::base::{errors::CrowdfundingError, types::StorageKey};
use soroban_sdk::Env;

// ── per-pool lock ────────────────────────────────────────────────────────────

pub fn acquire_pool_lock(env: &Env, pool_id: u64) -> Result<(), CrowdfundingError> {
    let key = StorageKey::ReentrancyLock(pool_id);
    if env
        .storage()
        .instance()
        .get::<StorageKey, bool>(&key)
        .unwrap_or(false)
    {
        return Err(CrowdfundingError::Unauthorized);
    }
    env.storage().instance().set(&key, &true);
    Ok(())
}

pub fn release_pool_lock(env: &Env, pool_id: u64) {
    env.storage()
        .instance()
        .remove(&StorageKey::ReentrancyLock(pool_id));
}

// ── global emergency-withdrawal lock ────────────────────────────────────────

pub fn acquire_emergency_lock(env: &Env) -> Result<(), CrowdfundingError> {
    let key = StorageKey::EmergencyWithdrawalLock;
    if env
        .storage()
        .instance()
        .get::<StorageKey, bool>(&key)
        .unwrap_or(false)
    {
        return Err(CrowdfundingError::Unauthorized);
    }
    env.storage().instance().set(&key, &true);
    Ok(())
}

pub fn release_emergency_lock(env: &Env) {
    env.storage()
        .instance()
        .remove(&StorageKey::EmergencyWithdrawalLock);
}

// ── public entry-point called by the trait impl ──────────────────────────────

/// Acquires the per-pool reentrancy lock.
///
/// Callers must always pair this with `release_pool_lock` — even on error paths:
///
/// ```rust
/// reentrancy_lock_logic(&env, pool_id)?;     // 1. acquire
/// let result = do_withdrawal_logic(...);      // 2. effect
/// release_pool_lock(&env, pool_id);          // 3. release (unconditional)
/// result
/// ```
pub fn reentrancy_lock_logic(env: &Env, pool_id: u64) -> Result<(), CrowdfundingError> {
    acquire_pool_lock(env, pool_id)
}
