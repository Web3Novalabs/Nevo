use soroban_sdk::{Address, Env, String};

use crate::types::{DataKey, SchoolRegistry};

/// Trait for managing the on-chain school/validator registry.
pub trait ValidatorRegistry {
    /// Register a school address with its verification metadata.
    /// Overwrites any existing entry for the same address.
    fn register_school(
        env: Env,
        school: Address,
        name: String,
        country: String,
        accreditation_id: String,
    );

    /// Returns `true` if `school` is present in the registry.
    fn is_registered(env: Env, school: Address) -> bool;

    /// Retrieve the registry entry for `school`, or `None` if not found.
    fn get_school(env: Env, school: Address) -> Option<SchoolRegistry>;
}

// ── storage helpers ───────────────────────────────────────────────────────────

/// Persist a `SchoolRegistry` entry keyed by `school`.
pub fn set_school(env: &Env, school: &Address, entry: &SchoolRegistry) {
    env.storage()
        .persistent()
        .set(&DataKey::SchoolRegistry(school.clone()), entry);
}

/// Returns `true` if `school` has a registry entry.
pub fn has_school(env: &Env, school: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::SchoolRegistry(school.clone()))
}

/// Retrieve the registry entry for `school`.
pub fn get_school(env: &Env, school: &Address) -> Option<SchoolRegistry> {
    env.storage()
        .persistent()
        .get(&DataKey::SchoolRegistry(school.clone()))
}
