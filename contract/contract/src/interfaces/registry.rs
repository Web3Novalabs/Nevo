use soroban_sdk::{Address, Env};

use crate::base::types::{SchoolRegistry, StorageKey};

/// Returns `true` if `validator` is present in the on-chain school registry.
pub fn is_validator_registered(env: &Env, validator: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&StorageKey::SchoolRegistry(validator.clone()))
}

/// Persist a `SchoolRegistry` entry keyed by `school`.
pub fn register_school(env: &Env, school: &Address, entry: &SchoolRegistry) {
    env.storage()
        .persistent()
        .set(&StorageKey::SchoolRegistry(school.clone()), entry);
}

/// Retrieve the registry entry for `school`.
pub fn get_school(env: &Env, school: &Address) -> Option<SchoolRegistry> {
    env.storage()
        .persistent()
        .get(&StorageKey::SchoolRegistry(school.clone()))
}
