#![no_std]

pub mod errors;
pub mod registry_interface;
pub mod types;

use errors::FundEduError;
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

/// On-chain status of a student application.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
}

/// Storage key scoped to a (validator, applicant) pair.
#[contracttype]
pub enum ApplicationKey {
    Application(Address, Address),
}

#[contract]
pub struct FundEduContract;

#[contractimpl]
impl FundEduContract {
    /// Approve a pending application.
    ///
    /// Requires `validator` auth. Returns `NotPending` if the application is
    /// not currently `Pending`.
    pub fn approve_application(
        env: Env,
        validator: Address,
        applicant: Address,
    ) -> Result<(), FundEduError> {
        validator.require_auth();

        let key = ApplicationKey::Application(validator.clone(), applicant.clone());
        let status: ApplicationStatus = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(ApplicationStatus::Pending);

        if status != ApplicationStatus::Pending {
            return Err(FundEduError::NotPending);
        }

        env.storage()
            .persistent()
            .set(&key, &ApplicationStatus::Approved);
        Ok(())
    }

    /// Reject a pending application.
    ///
    /// Requires `validator` auth. Returns `NotPending` if the application is
    /// not currently `Pending`.
    pub fn reject_application(
        env: Env,
        validator: Address,
        applicant: Address,
    ) -> Result<(), FundEduError> {
        validator.require_auth();

        let key = ApplicationKey::Application(validator.clone(), applicant.clone());
        let status: ApplicationStatus = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(ApplicationStatus::Pending);

        if status != ApplicationStatus::Pending {
            return Err(FundEduError::NotPending);
        }

        env.storage()
            .persistent()
            .set(&key, &ApplicationStatus::Rejected);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup() -> (Env, FundEduContractClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(FundEduContract, ());
        let client = FundEduContractClient::new(&env, &contract_id);
        let validator = Address::generate(&env);
        let applicant = Address::generate(&env);
        (env, client, validator, applicant)
    }

    // ── approve_application ──────────────────────────────────────────────────

    #[test]
    fn test_approve_pending_succeeds() {
        let (_env, client, validator, applicant) = setup();
        client.approve_application(&validator, &applicant);
    }

    #[test]
    fn test_approve_already_approved_returns_not_pending() {
        let (_env, client, validator, applicant) = setup();
        client.approve_application(&validator, &applicant);
        let err = client
            .try_approve_application(&validator, &applicant)
            .unwrap_err()
            .unwrap();
        assert_eq!(err, FundEduError::NotPending);
    }

    #[test]
    fn test_approve_already_rejected_returns_not_pending() {
        let (_env, client, validator, applicant) = setup();
        client.reject_application(&validator, &applicant);
        let err = client
            .try_approve_application(&validator, &applicant)
            .unwrap_err()
            .unwrap();
        assert_eq!(err, FundEduError::NotPending);
    }

    // ── reject_application ───────────────────────────────────────────────────

    #[test]
    fn test_reject_pending_succeeds() {
        let (_env, client, validator, applicant) = setup();
        client.reject_application(&validator, &applicant);
    }

    #[test]
    fn test_reject_already_approved_returns_not_pending() {
        let (_env, client, validator, applicant) = setup();
        client.approve_application(&validator, &applicant);
        let err = client
            .try_reject_application(&validator, &applicant)
            .unwrap_err()
            .unwrap();
        assert_eq!(err, FundEduError::NotPending);
    }

    #[test]
    fn test_reject_already_rejected_returns_not_pending() {
        let (_env, client, validator, applicant) = setup();
        client.reject_application(&validator, &applicant);
        let err = client
            .try_reject_application(&validator, &applicant)
            .unwrap_err()
            .unwrap();
        assert_eq!(err, FundEduError::NotPending);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn test_reject_unauthorized_panics() {
        let env = Env::default();
        // No mock_all_auths — auth will fail
        let contract_id = env.register(FundEduContract, ());
        let client = FundEduContractClient::new(&env, &contract_id);
        let validator = Address::generate(&env);
        let applicant = Address::generate(&env);
        client.reject_application(&validator, &applicant);
    }

    #[test]
    #[should_panic(expected = "Error(Auth, InvalidAction)")]
    fn test_approve_unauthorized_panics() {
        let env = Env::default();
        let contract_id = env.register(FundEduContract, ());
        let client = FundEduContractClient::new(&env, &contract_id);
        let validator = Address::generate(&env);
        let applicant = Address::generate(&env);
        client.approve_application(&validator, &applicant);
    }
}
