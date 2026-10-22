#[cfg(test)]
mod tests {
    use crate::{Contract, ContractClient, ContractError, PoolState};
    use soroban_sdk::{testutils::*, Address, Env, String};

    fn setup_pool_with_school(env: &Env, client: &ContractClient) -> (u32, Address, Address) {
        let admin = Address::generate(env);
        let sponsor = Address::generate(env);
        let school = Address::generate(env);

        client.set_admin(&admin);

        // Register school
        let metadata_hash = soroban_sdk::BytesN::<32>::from_array(
            env,
            &[0u8; 32],
        );
        admin.require_auth();
        client.register_school(&school, &metadata_hash);

        // Create pool for school
        let title = String::from_str(env, "Test Pool");
        let description = String::from_str(env, "Test Description");
        let goal = 1000u128;
        let deadline = 200000u64;

        let pool_id = client.create_pool_for_school(
            &sponsor,
            &title,
            &description,
            &goal,
            &school,
            &deadline,
        );

        (pool_id, school, sponsor)
    }

    #[test]
    fn test_apply_to_pool_valid_application() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, _sponsor) = setup_pool_with_school(&env, &client);
        let student = Address::generate(&env);

        let application_data = String::from_str(&env, "My application");

        student.require_auth();
        // Should succeed
        client.apply_to_pool(&pool_id, &student, &application_data);

        // Verify student has applied
        let has_applied = client.has_applied(&pool_id, &student);
        assert!(has_applied, "Student should have applied to the pool");
    }

    #[test]
    fn test_apply_to_pool_duplicate_application() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, _sponsor) = setup_pool_with_school(&env, &client);
        let student = Address::generate(&env);

        let application_data = String::from_str(&env, "My application");

        student.require_auth();
        // First application should succeed
        client.apply_to_pool(&pool_id, &student, &application_data.clone());

        student.require_auth();
        // Second application should panic with DuplicateApplication
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.apply_to_pool(&pool_id, &student, &application_data);
        }));

        assert!(result.is_err(), "Expected panic for duplicate application");
    }

    #[test]
    fn test_apply_to_pool_on_closed_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, sponsor) = setup_pool_with_school(&env, &client);

        // Set pool state to Disbursed and close it
        client.set_pool_state(&pool_id, &PoolState::Disbursed);
        sponsor.require_auth();
        client.close_pool(&pool_id);

        let student = Address::generate(&env);
        let application_data = String::from_str(&env, "My application");

        student.require_auth();
        // Should panic with PoolIsClosed
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.apply_to_pool(&pool_id, &student, &application_data);
        }));

        assert!(result.is_err(), "Expected panic when applying to closed pool");
    }

    #[test]
    fn test_apply_to_pool_on_cancelled_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, _sponsor) = setup_pool_with_school(&env, &client);

        // Set pool state to Cancelled
        client.set_pool_state(&pool_id, &PoolState::Cancelled);

        let student = Address::generate(&env);
        let application_data = String::from_str(&env, "My application");

        student.require_auth();
        // Should panic with InvalidPoolState
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.apply_to_pool(&pool_id, &student, &application_data);
        }));

        assert!(result.is_err(), "Expected panic when applying to cancelled pool");
    }

    #[test]
    fn test_apply_to_pool_on_paused_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, _sponsor) = setup_pool_with_school(&env, &client);

        // Set pool state to Paused
        client.set_pool_state(&pool_id, &PoolState::Paused);

        let student = Address::generate(&env);
        let application_data = String::from_str(&env, "My application");

        student.require_auth();
        // Should panic with InvalidPoolState
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.apply_to_pool(&pool_id, &student, &application_data);
        }));

        assert!(result.is_err(), "Expected panic when applying to paused pool");
    }

    #[test]
    fn test_apply_to_pool_on_completed_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, _sponsor) = setup_pool_with_school(&env, &client);

        // Set pool state to Completed
        client.set_pool_state(&pool_id, &PoolState::Completed);

        let student = Address::generate(&env);
        let application_data = String::from_str(&env, "My application");

        student.require_auth();
        // Should panic with InvalidPoolState
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.apply_to_pool(&pool_id, &student, &application_data);
        }));

        assert!(result.is_err(), "Expected panic when applying to completed pool");
    }

    #[test]
    fn test_apply_to_pool_on_disbursed_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, _sponsor) = setup_pool_with_school(&env, &client);

        // Set pool state to Disbursed
        client.set_pool_state(&pool_id, &PoolState::Disbursed);

        let student = Address::generate(&env);
        let application_data = String::from_str(&env, "My application");

        student.require_auth();
        // Should panic with InvalidPoolState
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.apply_to_pool(&pool_id, &student, &application_data);
        }));

        assert!(result.is_err(), "Expected panic when applying to disbursed pool");
    }

    #[test]
    fn test_apply_to_pool_nonexistent_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let student = Address::generate(&env);
        let application_data = String::from_str(&env, "My application");

        student.require_auth();
        // Should panic with PoolNotFound
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.apply_to_pool(&999u32, &student, &application_data);
        }));

        assert!(result.is_err(), "Expected panic for nonexistent pool");
    }

    #[test]
    fn test_apply_to_pool_multiple_students() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, _sponsor) = setup_pool_with_school(&env, &client);

        let student1 = Address::generate(&env);
        let student2 = Address::generate(&env);
        let student3 = Address::generate(&env);

        let application_data = String::from_str(&env, "My application");

        // Multiple different students can apply
        student1.require_auth();
        client.apply_to_pool(&pool_id, &student1, &application_data.clone());

        student2.require_auth();
        client.apply_to_pool(&pool_id, &student2, &application_data.clone());

        student3.require_auth();
        client.apply_to_pool(&pool_id, &student3, &application_data.clone());

        // Verify all students have applied
        assert!(client.has_applied(&pool_id, &student1));
        assert!(client.has_applied(&pool_id, &student2));
        assert!(client.has_applied(&pool_id, &student3));
    }

    #[test]
    fn test_get_application_count() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, _sponsor) = setup_pool_with_school(&env, &client);

        let student1 = Address::generate(&env);
        let student2 = Address::generate(&env);

        let application_data = String::from_str(&env, "My application");

        // Two students apply to the pool
        student1.require_auth();
        client.apply_to_pool(&pool_id, &student1, &application_data.clone());

        student2.require_auth();
        client.apply_to_pool(&pool_id, &student2, &application_data.clone());

        // Assert the getter returns 2
        let count = client.get_application_count(&pool_id);
        assert_eq!(count, 2, "Application count should be 2");
    }
}
