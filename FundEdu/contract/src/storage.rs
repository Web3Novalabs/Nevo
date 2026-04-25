use soroban_sdk::{Address, Env};

use crate::types::{Application, DataKey, ScholarshipPool};

const POOL_BUMP_AMOUNT: u32 = 100;
const POOL_LIFETIME_THRESHOLD: u32 = 50;

const APP_BUMP_AMOUNT: u32 = 100;
const APP_LIFETIME_THRESHOLD: u32 = 50;

pub fn get_pool(env: &Env, pool_id: u64) -> Option<ScholarshipPool> {
    let key = DataKey::Pool(pool_id);
    if let Some(pool) = env
        .storage()
        .persistent()
        .get::<DataKey, ScholarshipPool>(&key)
    {
        env.storage()
            .persistent()
            .extend_ttl(&key, POOL_LIFETIME_THRESHOLD, POOL_BUMP_AMOUNT);
        Some(pool)
    } else {
        None
    }
}

pub fn set_pool(env: &Env, pool_id: u64, pool: &ScholarshipPool) {
    let key = DataKey::Pool(pool_id);
    env.storage().persistent().set(&key, pool);
    env.storage()
        .persistent()
        .extend_ttl(&key, POOL_LIFETIME_THRESHOLD, POOL_BUMP_AMOUNT);
}

pub fn next_pool_id(env: &Env) -> u64 {
    let id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NextPoolId)
        .unwrap_or(0);
    env.storage()
        .instance()
        .set(&DataKey::NextPoolId, &(id + 1));
    id
}

pub fn get_application(env: &Env, pool_id: u64, student: Address) -> Option<Application> {
    let key = DataKey::Application(pool_id, student);
    if let Some(app) = env.storage().persistent().get::<DataKey, Application>(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, APP_LIFETIME_THRESHOLD, APP_BUMP_AMOUNT);
        Some(app)
    } else {
        None
    }
}

pub fn set_application(env: &Env, pool_id: u64, student: Address, application: &Application) {
    let key = DataKey::Application(pool_id, student);
    env.storage().persistent().set(&key, application);
    env.storage()
        .persistent()
        .extend_ttl(&key, APP_LIFETIME_THRESHOLD, APP_BUMP_AMOUNT);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ApplicationStatus;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_application_storage() {
        let env = Env::default();
        let contract_id = env.register(crate::contract::FundEduContract, ());

        env.as_contract(&contract_id, || {
            let student = Address::generate(&env);
            let pool_id = 1;

            let app = Application {
                student: student.clone(),
                requested_amount: 1000,
                total_granted: 1000,
                amount_claimed: 0,
                status: ApplicationStatus::Pending,
                milestone_index: 0,
            };

            set_application(&env, pool_id, student.clone(), &app);
            let retrieved = get_application(&env, pool_id, student).unwrap();

            assert_eq!(retrieved.student, app.student);
            assert_eq!(retrieved.requested_amount, app.requested_amount);
            assert_eq!(retrieved.status, app.status);
            assert_eq!(retrieved.milestone_index, app.milestone_index);
        });
    }

    #[test]
    fn test_get_non_existent_application() {
        let env = Env::default();
        let contract_id = env.register(crate::contract::FundEduContract, ());

        env.as_contract(&contract_id, || {
            let student = Address::generate(&env);
            let pool_id = 1;

            assert!(get_application(&env, pool_id, student).is_none());
        });
    }
}
