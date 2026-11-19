#[cfg(test)]
mod tests {
    use crate::{Contract, ContractClient, MAX_DESCRIPTION_LENGTH, MAX_URL_LENGTH, MAX_IMAGE_HASH_LENGTH};
    use soroban_sdk::{testutils::*, Address, Env, String, Vec};

    fn setup_pool(env: &Env, client: &ContractClient) -> (u32, Address) {
        let admin = Address::generate(env);
        let sponsor = Address::generate(env);

        client.set_admin(&admin);

        let title = String::from_str(env, "Test Pool");
        let description = String::from_str(env, "Test Description");
        let goal = 1000u128;
        let deadline = 200000u64;

        let pool_id = client.create_pool(&sponsor, &title, &description, &goal, &deadline);
        (pool_id, sponsor)
    }

    fn create_signers(env: &Env, count: u32) -> Vec<Address> {
        let mut signers = Vec::new(env);
        for _ in 0..count {
            signers.push_back(Address::generate(env));
        }
        signers
    }

    #[test]
    fn test_save_pool_description_at_max_length() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        // Create description exactly at MAX_DESCRIPTION_LENGTH
        let description = String::from_str(&env, &"a".repeat(MAX_DESCRIPTION_LENGTH as usize));
        let url = String::from_str(&env, "http://example.com");
        let image_hash = String::from_str(&env, "abc123");

        sponsor.require_auth();
        // Should succeed
        client.save_pool(&pool_id, &description, &url, &image_hash, &1, &signers);

        // Verify metadata was saved
        let (saved_desc, _, _) = client.get_saved_pool_metadata(&pool_id);
        assert_eq!(saved_desc.len(), MAX_DESCRIPTION_LENGTH as usize);
    }

    #[test]
    fn test_save_pool_description_exceeds_max_length() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        // Create description exceeding MAX_DESCRIPTION_LENGTH by 1
        let description = String::from_str(&env, &"a".repeat(MAX_DESCRIPTION_LENGTH as usize + 1));
        let url = String::from_str(&env, "http://example.com");
        let image_hash = String::from_str(&env, "abc123");

        sponsor.require_auth();
        // Should panic with "Description exceeds maximum length"
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.save_pool(&pool_id, &description, &url, &image_hash, &1, &signers);
        }));

        assert!(result.is_err(), "Expected panic for description exceeding max length");
    }

    #[test]
    fn test_save_pool_url_at_max_length() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        let description = String::from_str(&env, "Valid description");
        // Create URL exactly at MAX_URL_LENGTH
        let url = String::from_str(&env, &"a".repeat(MAX_URL_LENGTH as usize));
        let image_hash = String::from_str(&env, "abc123");

        sponsor.require_auth();
        // Should succeed
        client.save_pool(&pool_id, &description, &url, &image_hash, &1, &signers);

        // Verify metadata was saved
        let (_, saved_url, _) = client.get_saved_pool_metadata(&pool_id);
        assert_eq!(saved_url.len(), MAX_URL_LENGTH as usize);
    }

    #[test]
    fn test_save_pool_url_exceeds_max_length() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        let description = String::from_str(&env, "Valid description");
        // Create URL exceeding MAX_URL_LENGTH by 1
        let url = String::from_str(&env, &"a".repeat(MAX_URL_LENGTH as usize + 1));
        let image_hash = String::from_str(&env, "abc123");

        sponsor.require_auth();
        // Should panic with "URL exceeds maximum length"
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.save_pool(&pool_id, &description, &url, &image_hash, &1, &signers);
        }));

        assert!(result.is_err(), "Expected panic for URL exceeding max length");
    }

    #[test]
    fn test_save_pool_image_hash_at_max_length() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        let description = String::from_str(&env, "Valid description");
        let url = String::from_str(&env, "http://example.com");
        // Create image_hash exactly at MAX_IMAGE_HASH_LENGTH
        let image_hash = String::from_str(&env, &"f".repeat(MAX_IMAGE_HASH_LENGTH as usize));

        sponsor.require_auth();
        // Should succeed
        client.save_pool(&pool_id, &description, &url, &image_hash, &1, &signers);

        // Verify metadata was saved
        let (_, _, saved_hash) = client.get_saved_pool_metadata(&pool_id);
        assert_eq!(saved_hash.len(), MAX_IMAGE_HASH_LENGTH as usize);
    }

    #[test]
    fn test_save_pool_image_hash_exceeds_max_length() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        let description = String::from_str(&env, "Valid description");
        let url = String::from_str(&env, "http://example.com");
        // Create image_hash exceeding MAX_IMAGE_HASH_LENGTH by 1
        let image_hash = String::from_str(&env, &"f".repeat(MAX_IMAGE_HASH_LENGTH as usize + 1));

        sponsor.require_auth();
        // Should panic with "Image hash exceeds maximum length"
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.save_pool(&pool_id, &description, &url, &image_hash, &1, &signers);
        }));

        assert!(result.is_err(), "Expected panic for image hash exceeding max length");
    }

    #[test]
    fn test_save_pool_empty_signers_list() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = Vec::new(&env); // Empty signers list

        let description = String::from_str(&env, "Valid description");
        let url = String::from_str(&env, "http://example.com");
        let image_hash = String::from_str(&env, "abc123");

        sponsor.require_auth();
        // Should panic with "Empty signers list"
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.save_pool(&pool_id, &description, &url, &image_hash, &1, &signers);
        }));

        assert!(result.is_err(), "Expected panic for empty signers list");
    }

    #[test]
    fn test_save_pool_zero_required_signatures() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        let description = String::from_str(&env, "Valid description");
        let url = String::from_str(&env, "http://example.com");
        let image_hash = String::from_str(&env, "abc123");

        sponsor.require_auth();
        // Should panic with "Zero required_signatures"
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.save_pool(&pool_id, &description, &url, &image_hash, &0, &signers);
        }));

        assert!(result.is_err(), "Expected panic for zero required_signatures");
    }

    #[test]
    fn test_save_pool_required_signatures_exceeds_signer_count() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        let description = String::from_str(&env, "Valid description");
        let url = String::from_str(&env, "http://example.com");
        let image_hash = String::from_str(&env, "abc123");

        sponsor.require_auth();
        // Should panic with "required_signatures exceeds signer count"
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.save_pool(&pool_id, &description, &url, &image_hash, &3, &signers); // 3 > 2 signers
        }));

        assert!(result.is_err(), "Expected panic for required_signatures exceeding signer count");
    }

    #[test]
    fn test_save_pool_duplicate_signers() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);

        // Create signers with a duplicate
        let signer1 = Address::generate(&env);
        let signer2 = Address::generate(&env);
        let mut signers = Vec::new(&env);
        signers.push_back(signer1.clone());
        signers.push_back(signer1.clone()); // Duplicate
        signers.push_back(signer2);

        let description = String::from_str(&env, "Valid description");
        let url = String::from_str(&env, "http://example.com");
        let image_hash = String::from_str(&env, "abc123");

        sponsor.require_auth();
        // Should panic with "Mismatched multi-signature parameters"
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.save_pool(&pool_id, &description, &url, &image_hash, &2, &signers);
        }));

        assert!(result.is_err(), "Expected panic for duplicate signers");
    }

    #[test]
    fn test_save_pool_requires_sponsor_auth() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, _sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 2);

        let description = String::from_str(&env, "Valid description");
        let url = String::from_str(&env, "http://example.com");
        let image_hash = String::from_str(&env, "abc123");

        let unauthorized_caller = Address::generate(&env);
        unauthorized_caller.require_auth();

        // Should fail authorization check (not sponsor)
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.save_pool(&pool_id, &description, &url, &image_hash, &1, &signers);
        }));

        assert!(result.is_err(), "Expected authorization failure for non-sponsor");
    }

    #[test]
    fn test_save_pool_valid() {
        let env = Env::new();
        let contract_id = env.register_contract(None, Contract);
        let client = ContractClient::new(&env, &contract_id);

        let (pool_id, sponsor) = setup_pool(&env, &client);
        let signers = create_signers(&env, 3);

        let description = String::from_str(&env, "Valid description");
        let url = String::from_str(&env, "http://example.com");
        let image_hash = String::from_str(&env, "abc123def456");

        sponsor.require_auth();
        // Should succeed
        client.save_pool(&pool_id, &description, &url, &image_hash, &2, &signers);

        // Verify metadata was saved
        let (saved_desc, saved_url, saved_hash) = client.get_saved_pool_metadata(&pool_id);
        assert_eq!(saved_desc, description);
        assert_eq!(saved_url, url);
        assert_eq!(saved_hash, image_hash);

        // Verify signers were saved
        let (required_sigs, saved_signers) = client.get_pool_signers(&pool_id);
        assert_eq!(required_sigs, 2);
        assert_eq!(saved_signers.len(), 3);
    }
}
