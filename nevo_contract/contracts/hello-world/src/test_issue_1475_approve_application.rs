#[cfg(test)]
mod tests {
    use crate::{Contract, ContractClient, ContractError, PoolState};
    use soroban_sdk::{testutils::*, Address, Env, String};

    fn setup_pool_with_school_and_application(
        env: &Env,
        client: &ContractClient,
    ) -> (u32, Address, Address, Address) {
        let admin = Address::generate(env);
        let sponsor = Address::generate(env);
        let school = Address::generate(env);
        let student = Address::generate(env);

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

        // Student applies to pool
        let application_data = String::from_str(env, "My application");
        student.require_auth();
        client.apply_to_pool(&pool_id, &student, &application_data);

        (pool_id, school, student, sponsor)
    }

    #[test]
    fn test_approve_application_valid_approval() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, school, student, _sponsor) = setup_pool_with_school_and_application(&env, &client);

        school.require_auth();
        // Should succeed
        client.approve_application(&pool_id, &school, &student, &true);

        // Verify application status is Approved
        let status = client.get_application_status(&pool_id, &student);
        assert_eq!(status, String::from_str(&env, "Approved"));
    }

    #[test]
    fn test_approve_application_valid_rejection() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, school, student, _sponsor) = setup_pool_with_school_and_application(&env, &client);

        school.require_auth();
        // Should succeed
        client.approve_application(&pool_id, &school, &student, &false);

        // Verify application status is Rejected
        let status = client.get_application_status(&pool_id, &student);
        assert_eq!(status, String::from_str(&env, "Rejected"));
    }

    #[test]
    fn test_approve_application_on_closed_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, school, student, sponsor) = setup_pool_with_school_and_application(&env, &client);

        // Set pool state to Disbursed and close it
        client.set_pool_state(&pool_id, &PoolState::Disbursed);
        sponsor.require_auth();
        client.close_pool(&pool_id);

        school.require_auth();
        // Should panic with PoolIsClosed
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_application(&pool_id, &school, &student, &true);
        }));

        assert!(result.is_err(), "Expected panic when approving on closed pool");
    }

    #[test]
    fn test_approve_application_on_cancelled_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, school, student, _sponsor) = setup_pool_with_school_and_application(&env, &client);

        // Set pool state to Cancelled
        client.set_pool_state(&pool_id, &PoolState::Cancelled);

        school.require_auth();
        // Should panic with InvalidPoolState
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_application(&pool_id, &school, &student, &true);
        }));

        assert!(result.is_err(), "Expected panic when approving on cancelled pool");
    }

    #[test]
    fn test_approve_application_wrong_school() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _school, student, _sponsor) = setup_pool_with_school_and_application(&env, &client);

        let wrong_school = Address::generate(&env);
        wrong_school.require_auth();
        // Should panic with OnlyLinkedSchoolCanApprove
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_application(&pool_id, &wrong_school, &student, &true);
        }));

        assert!(result.is_err(), "Expected panic when non-linked school tries to approve");
    }

    #[test]
    fn test_approve_application_student_not_applied() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, school, _student, _sponsor) = setup_pool_with_school_and_application(&env, &client);

        let student_who_did_not_apply = Address::generate(&env);

        school.require_auth();
        // Should panic with StudentHasNotApplied
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_application(&pool_id, &school, &student_who_did_not_apply, &true);
        }));

        assert!(result.is_err(), "Expected panic when student has not applied");
    }

    #[test]
    fn test_approve_application_nonexistent_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let school = Address::generate(&env);
        let student = Address::generate(&env);

        school.require_auth();
        // Should panic with PoolNotFound
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.approve_application(&999u32, &school, &student, &true);
        }));

        assert!(result.is_err(), "Expected panic for nonexistent pool");
    }

    #[test]
    fn test_approve_application_on_paused_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, school, student, _sponsor) = setup_pool_with_school_and_application(&env, &client);

        // Set pool state to Paused (still in non-Cancelled state, so approval should succeed)
        client.set_pool_state(&pool_id, &PoolState::Paused);

        school.require_auth();
        // Should succeed - Paused is not Cancelled
        client.approve_application(&pool_id, &school, &student, &true);

        let status = client.get_application_status(&pool_id, &student);
        assert_eq!(status, String::from_str(&env, "Approved"));
    }

    #[test]
    fn test_approve_application_on_completed_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, school, student, _sponsor) = setup_pool_with_school_and_application(&env, &client);

        // Set pool state to Completed (still in non-Cancelled state, so approval should succeed)
        client.set_pool_state(&pool_id, &PoolState::Completed);

        school.require_auth();
        // Should succeed - Completed is not Cancelled
        client.approve_application(&pool_id, &school, &student, &true);

        let status = client.get_application_status(&pool_id, &student);
        assert_eq!(status, String::from_str(&env, "Approved"));
    }

    #[test]
    fn test_approve_application_on_disbursed_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, school, student, _sponsor) = setup_pool_with_school_and_application(&env, &client);

        // Set pool state to Disbursed (still in non-Cancelled state, so approval should succeed)
        client.set_pool_state(&pool_id, &PoolState::Disbursed);

        school.require_auth();
        // Should succeed - Disbursed is not Cancelled
        client.approve_application(&pool_id, &school, &student, &true);

        let status = client.get_application_status(&pool_id, &student);
        assert_eq!(status, String::from_str(&env, "Approved"));
    }

    #[test]
    fn test_approve_application_multiple_students() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let sponsor = Address::generate(&env);
        let school = Address::generate(&env);

        client.set_admin(&admin);

        let metadata_hash = soroban_sdk::BytesN::<32>::from_array(
            env,
            &[0u8; 32],
        );
        admin.require_auth();
        client.register_school(&school, &metadata_hash);

        let title = String::from_str(&env, "Test Pool");
        let description = String::from_str(&env, "Test Description");
        let goal = 3000u128;
        let deadline = 200000u64;

        let pool_id = client.create_pool_for_school(
            &sponsor,
            &title,
            &description,
            &goal,
            &school,
            &deadline,
        );

        // Multiple students apply
        let student1 = Address::generate(&env);
        let student2 = Address::generate(&env);
        let student3 = Address::generate(&env);

        let application_data = String::from_str(&env, "My application");

        student1.require_auth();
        client.apply_to_pool(&pool_id, &student1, &application_data.clone());

        student2.require_auth();
        client.apply_to_pool(&pool_id, &student2, &application_data.clone());

        student3.require_auth();
        client.apply_to_pool(&pool_id, &student3, &application_data.clone());

        // School approves all students
        school.require_auth();
        client.approve_application(&pool_id, &school, &student1, &true);
        client.approve_application(&pool_id, &school, &student2, &true);
        client.approve_application(&pool_id, &school, &student3, &false); // Reject one

        // Verify statuses
        let status1 = client.get_application_status(&pool_id, &student1);
        let status2 = client.get_application_status(&pool_id, &student2);
        let status3 = client.get_application_status(&pool_id, &student3);

        assert_eq!(status1, String::from_str(&env, "Approved"));
        assert_eq!(status2, String::from_str(&env, "Approved"));
        assert_eq!(status3, String::from_str(&env, "Rejected"));
    }
}
