#[cfg(test)]
mod tests {
    use crate::{Contract, ContractClient, ContractError, Pool, PoolState};
    use soroban_sdk::{testutils::*, token, Address, Env, String};

    fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
        let token_admin = Address::generate(env);
        let contract_id = env.register_stellar_asset_contract(token_admin.clone());
        let token_client = token::Client::new(env, &contract_id);
        token_client.mint(&token_admin, recipient, &amount);
        contract_id
    }

    fn setup_pool_with_token(env: &Env, client: &ContractClient) -> (u32, Address) {
        // Create admin and pool sponsor
        let admin = Address::generate(env);
        let sponsor = Address::generate(env);

        // Initialize admin
        client.set_admin(&admin);

        // Create a pool
        let title = String::from_str(env, "Test Pool");
        let description = String::from_str(env, "Test Description");
        let goal = 1000u128;
        let deadline = 200000u64;

        let pool_id = client.create_pool(&sponsor, &title, &description, &goal, &deadline);

        // Create and configure token
        let token = create_token(env, 10000, &sponsor);
        client.set_pool_token(&pool_id, &token);

        (pool_id, token)
    }

    #[test]
    fn test_donate_with_token_invalid_amount_zero() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, token) = setup_pool_with_token(&env, &client);
        let donor = Address::generate(&env);

        // Create token for donor
        create_token(&env, 1000, &donor);

        // Attempt to donate with amount 0 - should panic with InvalidAmount
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.donate_with_token(&pool_id, &donor, &token, &0);
        }));

        assert!(result.is_err(), "Expected panic with amount = 0");
    }

    #[test]
    fn test_donate_with_token_invalid_amount_negative() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, token) = setup_pool_with_token(&env, &client);
        let donor = Address::generate(&env);

        // Create token for donor
        create_token(&env, 1000, &donor);

        // Attempt to donate with negative amount - should panic with InvalidAmount
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.donate_with_token(&pool_id, &donor, &token, &-100);
        }));

        assert!(result.is_err(), "Expected panic with negative amount");
    }

    #[test]
    fn test_donate_with_token_mismatched_token_address() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _token) = setup_pool_with_token(&env, &client);
        let donor = Address::generate(&env);

        // Create two different tokens
        let configured_token = create_token(&env, 1000, &donor);
        let wrong_token = create_token(&env, 1000, &donor);

        // Pool is configured with configured_token via set_pool_token
        client.set_pool_token(&pool_id, &configured_token);

        // Attempt to donate with a different token - should panic with TokenTransferFailed
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.donate_with_token(&pool_id, &donor, &wrong_token, &100);
        }));

        assert!(
            result.is_err(),
            "Expected panic when token_address does not match configured token"
        );
    }

    #[test]
    fn test_donate_with_token_valid_donation() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, token) = setup_pool_with_token(&env, &client);
        let donor = Address::generate(&env);

        // Create token for donor
        create_token(&env, 10000, &donor);

        // Successful donation with positive amount
        donor.require_auth();
        client.donate_with_token(&pool_id, &donor, &token, &500);

        // Verify pool collected amount increased
        let (_, _, _, collected, _, _) = client.get_pool(&pool_id);
        assert_eq!(collected, 500u128, "Pool collected amount should be 500");
    }

    #[test]
    fn test_donate_with_token_on_closed_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let sponsor = Address::generate(&env);
        client.set_admin(&admin);

        let title = String::from_str(&env, "Test Pool");
        let description = String::from_str(&env, "Test Description");
        let goal = 1000u128;
        let deadline = 200000u64;

        let pool_id = client.create_pool(&sponsor, &title, &description, &goal, &deadline);
        let token = create_token(&env, 10000, &sponsor);
        client.set_pool_token(&pool_id, &token);

        // Set pool state to Disbursed and close it
        client.set_pool_state(&pool_id, &PoolState::Disbursed);
        sponsor.require_auth();
        client.close_pool(&pool_id);

        let donor = Address::generate(&env);
        create_token(&env, 1000, &donor);

        // Attempt to donate to closed pool - should panic with PoolIsClosed
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.donate_with_token(&pool_id, &donor, &token, &100);
        }));

        assert!(result.is_err(), "Expected panic when donating to closed pool");
    }

    #[test]
    fn test_donate_with_token_on_inactive_pool() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let sponsor = Address::generate(&env);
        client.set_admin(&admin);

        let title = String::from_str(&env, "Test Pool");
        let description = String::from_str(&env, "Test Description");
        let goal = 1000u128;
        let deadline = 200000u64;

        let pool_id = client.create_pool(&sponsor, &title, &description, &goal, &deadline);
        let token = create_token(&env, 10000, &sponsor);
        client.set_pool_token(&pool_id, &token);

        // Set pool state to Cancelled
        client.set_pool_state(&pool_id, &PoolState::Cancelled);

        let donor = Address::generate(&env);
        create_token(&env, 1000, &donor);

        // Attempt to donate to cancelled pool - should panic with InvalidPoolState
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.donate_with_token(&pool_id, &donor, &token, &100);
        }));

        assert!(result.is_err(), "Expected panic when donating to inactive pool");
    }
}
