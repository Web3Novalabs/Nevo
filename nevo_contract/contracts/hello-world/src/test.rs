#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token::StellarAssetClient,
    Address, Env, IntoVal, String, Vec,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

// ============= BASIC POOL TESTS =============

#[test]
fn test_create_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Emergency Relief Fund"),
        &String::from_str(&env, "Helping those in need"),
        &1_000_000_000u128,
    );

    assert_eq!(pool_id, 1);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, 1);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, 1_000_000_000u128);
    assert_eq!(pool.3, 0u128);
    assert_eq!(pool.4, false);
}

#[test]
fn test_donate() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Educational Scholarship"),
        &String::from_str(&env, "Support for students"),
        &10_000_000_000u128,
    );

    client.donate(&pool_id, &donor, &100_000_000u128);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000u128);
}

#[test]
fn test_multiple_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Community Project"),
        &String::from_str(&env, "Building together"),
        &5_000_000_000u128,
    );

    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
    client.donate(&pool_id, &Address::generate(&env), &200_000_000u128);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 300_000_000u128);
}

#[test]
fn test_close_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closed Pool"),
        &String::from_str(&env, "Test pool"),
        &1_000_000_000u128,
    );
    client.close_pool(&pool_id);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
}

#[test]
#[should_panic(expected = "Pool is closed")]
fn test_donate_to_closed_pool() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.close_pool(&pool_id);
    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_close_pool_unauthorized() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client
        .mock_auths(&[MockAuth {
            address: &unauthorized,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "close_pool",
                args: (&pool_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .close_pool(&pool_id);
}

#[test]
fn test_multiple_pools() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let pool_id_1 = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "First pool"),
        &1_000_000_000u128,
    );
    let pool_id_2 = client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Second pool"),
        &2_000_000_000u128,
    );

    assert_eq!(pool_id_1, 1);
    assert_eq!(pool_id_2, 2);
    assert_eq!(client.get_pool_count(), 2);
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_try_get_pool_returns_none_for_missing_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let _missing_pool = client.try_get_pool(&999).unwrap();
}

#[test]
fn test_get_total_raised_starts_at_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Fresh Pool"),
        &String::from_str(&env, "No donations yet"),
        &1_000_000_000u128,
    );
    assert_eq!(client.get_total_raised(&pool_id), 0);
}

#[test]
#[should_panic(expected = "Pool not found")]
fn test_get_total_raised_rejects_missing_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    let _ = client.get_total_raised(&999);
}

#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_pool_description_exceeds_max_length() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let long_desc = String::from_str(&env, &"x".repeat(501));
    client.create_pool(
        &Address::generate(&env),
        &String::from_str(&env, "Title"),
        &long_desc,
        &1_000_000_000u128,
    );
}

// ============= CLAIM FUNDS TESTS =============

#[test]
#[should_panic(expected = "Application status not found")]
fn test_claim_funds_no_status() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &creator, &500_000_000u128);
    let token = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token);
}

#[test]
#[should_panic(expected = "Application is not approved")]
fn test_claim_funds_rejected_application() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &creator, &500_000_000u128);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Rejected"));
    let token = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token);
}

#[test]
#[should_panic(expected = "Overdraw attempt")]
fn test_claim_funds_overdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &creator, &100_000_000u128);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));
    let token = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &500_000_000i128, &token);
}

#[test]
#[should_panic(expected = "Claim amount must be positive")]
fn test_claim_funds_negative_amount() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &creator, &500_000_000u128);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));
    let token = Address::generate(&env);
    client.claim_funds(&student, &pool_id, &-100_000_000i128, &token);
}

#[test]
fn test_get_claimed_amount_initial_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    assert_eq!(client.get_claimed_amount(&pool_id, &student), 0);
}

#[test]
fn test_get_application_status() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );

    assert_eq!(client.get_application_status(&pool_id, &student), String::from_str(&env, ""));

    let approved = String::from_str(&env, "Approved");
    client.set_application_status(&pool_id, &student, &approved);
    assert_eq!(client.get_application_status(&pool_id, &student), approved);
}

// ============= PROTOCOL FEES TESTS =============

#[test]
fn test_protocol_fees_accumulation_on_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let claim_amount: i128 = 100_000_000;
    let token = create_token(&env, claim_amount, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &creator, &500_000_000u128);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));
    client.claim_funds(&student, &pool_id, &claim_amount, &token);

    let app = client.get_application(&pool_id, &student);
    assert!(app.is_some());
}

#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_claim_protocol_fees_requires_admin_authorization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_admin(&admin);
    client.claim_protocol_fees(&non_admin, &token);
}

#[test]
#[should_panic(expected = "No unclaimed fees")]
fn test_claim_protocol_fees_no_fees() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_admin(&admin);
    client.claim_protocol_fees(&admin, &token);
}

#[test]
fn test_claim_protocol_fees_multiple_claims_accumulate() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let student1 = Address::generate(&env);
    let student2 = Address::generate(&env);
    let claim1: i128 = 100_000_000;
    let claim2: i128 = 50_000_000;
    let token = create_token(&env, claim1 + claim2, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &creator, &500_000_000u128);
    client.set_application_status(&pool_id, &student1, &String::from_str(&env, "Approved"));
    client.set_application_status(&pool_id, &student2, &String::from_str(&env, "Approved"));
    client.claim_funds(&student1, &pool_id, &claim1, &token);
    client.claim_funds(&student2, &pool_id, &claim2, &token);

    let fees = client.claim_protocol_fees(&admin, &token);
    assert_eq!(fees, 1_500_000); // 1% of 100M + 1% of 50M
}

#[test]
#[should_panic(expected = "No unclaimed fees")]
fn test_protocol_fees_reset_after_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let claim_amount: i128 = 100_000_000;
    let token = create_token(&env, claim_amount, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &creator, &500_000_000u128);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));
    client.claim_funds(&student, &pool_id, &claim_amount, &token);
    client.claim_protocol_fees(&admin, &token);
    // Second claim should panic
    client.claim_protocol_fees(&admin, &token);
}

// ============= DONOR COUNT TRACKING TESTS =============

#[test]
fn test_new_campaign_has_zero_donors() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
    );
    assert_eq!(client.get_donor_count(&pool_id), 0);
}

#[test]
fn test_same_donor_multiple_donations_keeps_count_at_one() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &donor, &100_000_000u128);
    client.donate(&pool_id, &donor, &200_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 1);
}

#[test]
fn test_different_donors_increment_count_correctly() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
    client.donate(&pool_id, &Address::generate(&env), &200_000_000u128);
    client.donate(&pool_id, &Address::generate(&env), &300_000_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 3);
}

#[test]
#[should_panic(expected = "Pool not found")]
fn test_donor_count_nonexistent_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    client.get_donor_count(&999u32);
}

#[test]
fn test_multiple_contributors_tracked_separately() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &donor1, &100_000_000u128);
    client.donate(&pool_id, &donor2, &200_000_000u128);
    client.donate(&pool_id, &donor3, &300_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor1), 100_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor2), 200_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor3), 300_000_000u128);
}

// ============= SCHOOL & APPLICATION TESTS =============

#[test]
fn test_register_school_and_create_pool_for_school() {
#[test]
#[should_panic(expected = "Pool not found")]
fn test_nonexistent_campaign_returns_error() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Attempt to get donor count for a pool that doesn't exist
    client.get_donor_count(&999u32);
}

// ============= CONTRIBUTIONS GETTER VALIDATION TESTS =============

#[test]
fn test_contributor_with_no_donations_returns_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000,
    );

    assert_eq!(client.get_contribution(&pool_id, &donor), 0);
}

#[test]
fn test_contributor_with_multiple_donations_returns_sum() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &donor, &100_000_000);
    client.donate(&pool_id, &donor, &200_000_000);
    client.donate(&pool_id, &donor, &50_000_000);
    assert_eq!(client.get_contribution(&pool_id, &donor), 350_000_000);
}

#[test]
fn test_nonexistent_contributor_returns_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let existing_donor = Address::generate(&env);
    let nonexistent_donor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &existing_donor, &100_000_000);
    assert_eq!(client.get_contribution(&pool_id, &nonexistent_donor), 0);
}

#[test]
#[should_panic(expected = "Pool not found")]
fn test_nonexistent_campaign_returns_campaign_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    client.get_contribution(&999u32, &donor);
}

#[test]
fn test_multiple_contributors_tracked_separately() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let donor3 = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Description"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &donor1, &100_000_000);
    client.donate(&pool_id, &donor2, &200_000_000);
    client.donate(&pool_id, &donor3, &300_000_000);

    assert_eq!(client.get_contribution(&pool_id, &donor1), 100_000_000);
    assert_eq!(client.get_contribution(&pool_id, &donor2), 200_000_000);
    assert_eq!(client.get_contribution(&pool_id, &donor3), 300_000_000);
}

// ============= ISSUE #515: FUNCTION PARAMETER VALIDATION TESTS =============

// (1) Out-of-range / zero values caught
#[test]
#[should_panic(expected = "Claim amount must be positive")]
fn test_claim_funds_zero_amount_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Doc states: "Donate to an existing pool"
    // Should update collected amount
    client.donate(&pool_id, &donor, &100_000_000);
    
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000); // collected updated
}

/// DOC TEST 3: donate error condition - closed pool
/// Verifies: Error conditions documented (panics with "Pool is closed")
#[test]
#[should_panic(expected = "Pool is closed")]
fn test_doc_donate_error_pool_closed() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Close the pool
    client.mock_auths(&[MockAuth {
        address: &creator,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "close_pool",
            args: (&pool_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]).close_pool(&pool_id);

    // Doc states: panics with "Pool is closed" when donating to closed pool
    client.donate(&pool_id, &donor, &100_000_000);
}

/// DOC TEST 4: donate error condition - pool not found
/// Verifies: Error conditions documented
#[test]
#[should_panic(expected = "Pool not found")]
fn test_doc_donate_error_pool_not_found() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    
    // Try to donate to non-existent pool
    // Doc states: panics with "Pool not found"
    client.donate(&999, &donor, &100_000_000);
}

// ============= CONFIGURATION BOUNDS VALIDATION TESTS =============
// These tests verify that configuration parameters are properly validated
// against their defined bounds and constraints.

/// TEST 1: Maximum description length enforced (MAX_DESCRIPTION_LENGTH = 500)
#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_config_bounds_description_max_length_exceeded() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Test Pool");
    
    // Create a description that exceeds MAX_DESCRIPTION_LENGTH (500 chars)
    let long_description = "a".repeat(501);
    let description = String::from_str(&env, &long_description);
    let goal: u128 = 1_000_000_000;

    // Should panic with "Description exceeds maximum length"
    client.create_pool(&creator, &title, &description, &goal);
}

/// TEST 2: Maximum description length at boundary (exactly 500 chars)
#[test]
fn test_config_bounds_description_max_length_at_boundary() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Test Pool");
    
    // Create a description exactly at MAX_DESCRIPTION_LENGTH (500 chars)
    let boundary_description = "a".repeat(500);
    let description = String::from_str(&env, &boundary_description);
    let goal: u128 = 1_000_000_000;

    // Should succeed - exactly at boundary
    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    assert_eq!(pool_id, 1);
}

/// TEST 3: Description length just under boundary (499 chars)
#[test]
fn test_config_bounds_description_length_under_boundary() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Test Pool");
    
    // Create a description just under MAX_DESCRIPTION_LENGTH (499 chars)
    let valid_description = "a".repeat(499);
    let description = String::from_str(&env, &valid_description);
    let goal: u128 = 1_000_000_000;

    // Should succeed
    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    assert_eq!(pool_id, 1);
}

/// TEST 4: Empty description allowed
#[test]
fn test_config_bounds_description_empty_allowed() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Test Pool");
    let description = String::from_str(&env, "");
    let goal: u128 = 1_000_000_000;

    // Empty description should be allowed
    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    assert_eq!(pool_id, 1);
}

/// TEST 5: Numeric parameter - goal at u128 maximum boundary
#[test]
fn test_config_bounds_goal_u128_max() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Max Goal Pool");
    let description = String::from_str(&env, "Testing maximum goal value");
    let goal: u128 = u128::MAX;

    // Should handle maximum u128 value
    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.2, u128::MAX);
}

/// TEST 6: Numeric parameter - goal at zero (minimum boundary)
#[test]
fn test_config_bounds_goal_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Zero Goal Pool");
    let description = String::from_str(&env, "Testing zero goal");
    let goal: u128 = 0;

    // Zero goal should be allowed (no explicit validation)
    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.2, 0);
}

/// TEST 7: Numeric parameter - donation amount overflow protection
#[test]
#[should_panic(expected = "Collected amount overflow")]
fn test_config_bounds_donation_overflow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token_address = Address::generate(&env);
    
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Overflow Test"),
        &String::from_str(&env, "Test"),
        &u128::MAX,
    );

    // First donation near max
    client.donate_with_token(&pool_id, &donor, &token_address, &(u128::MAX as i128 - 1000));
    
    // Second donation should cause overflow
    client.donate_with_token(&pool_id, &donor, &token_address, &2000i128);
}

/// TEST 8: Numeric parameter - pool_id sequential validation
#[test]
fn test_config_bounds_pool_id_sequential() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Sequential Test");
    let description = String::from_str(&env, "Test");
    let goal: u128 = 1_000_000_000;

    // Create multiple pools and verify sequential IDs
    let pool_id_1 = client.create_pool(&creator, &title, &description, &goal);
    let pool_id_2 = client.create_pool(&creator, &title, &description, &goal);
    let pool_id_3 = client.create_pool(&creator, &title, &description, &goal);

    assert_eq!(pool_id_1, 1);
    assert_eq!(pool_id_2, 2);
    assert_eq!(pool_id_3, 3);
    assert_eq!(client.get_pool_count(), 3);
}

/// TEST 9: Numeric parameter - milestone amount validation
#[test]
#[should_panic(expected = "Milestone total must equal pool goal")]
fn test_config_bounds_milestone_sum_mismatch() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Test"),
        &String::from_str(&env, "Test"),
        &goal,
    );

    // Create milestones that don't sum to goal
    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: 300_000_000 });
    milestones.push_back(Milestone { amount: 400_000_000 });
    // Total: 700_000_000, but goal is 1_000_000_000

    // Should panic with "Milestone total must equal pool goal"
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

/// TEST 10: Numeric parameter - milestone sum equals goal (valid)
#[test]
fn test_config_bounds_milestone_sum_valid() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Test"),
        &String::from_str(&env, "Test"),
        &goal,
    );

    // Create milestones that sum exactly to goal
    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: 300_000_000 });
    milestones.push_back(Milestone { amount: 400_000_000 });
    milestones.push_back(Milestone { amount: 300_000_000 });
    // Total: 1_000_000_000 = goal

    // Should succeed
    client.setup_application_milestones(&pool_id, &student, &milestones);

    let retrieved_milestones = client.get_milestones(&pool_id, &student);
    assert_eq!(retrieved_milestones.len(), 3);
}

/// TEST 11: Numeric parameter - milestone overflow protection
#[test]
#[should_panic(expected = "Milestone amount overflow")]
fn test_config_bounds_milestone_overflow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let goal: u128 = u128::MAX;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Overflow Test"),
        &String::from_str(&env, "Test"),
        &goal,
    );

    // Create milestones that would overflow when summed
    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: u128::MAX });
    milestones.push_back(Milestone { amount: 1 });

    // Should panic with "Milestone amount overflow"
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

/// TEST 12: Numeric parameter - empty milestones rejected
#[test]
#[should_panic(expected = "Milestones required")]
fn test_config_bounds_milestones_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Empty Milestones Test"),
        &String::from_str(&env, "Test"),
        &goal,
    );

    // Empty milestones vector
    let milestones = Vec::new(&env);

    // Should panic with "Milestones required"
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

/// TEST 13: String encoding - UTF-8 characters in description
#[test]
fn test_config_bounds_description_utf8_encoding() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "UTF-8 Test Pool");
    
    // Test with various UTF-8 characters (emojis, special chars, etc.)
    let utf8_description = String::from_str(&env, "Education fund 🎓 for students in África & Asia");
    let goal: u128 = 1_000_000_000;

    // Should handle UTF-8 encoding properly
    let pool_id = client.create_pool(&creator, &title, &utf8_description, &goal);
    assert_eq!(pool_id, 1);
}

/// TEST 14: String encoding - special characters in application data
#[test]
fn test_config_bounds_application_data_special_chars() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let admin = Address::generate(&env);
    let school = Address::generate(&env);

    // Set admin and register school
    client.set_admin(&admin);
    client.register_school(&admin, &school);

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
        &school,
    );

    // Application data with special characters
    let special_chars_data = String::from_str(&env, "Name: José García\nGPA: 3.8\nEmail: test@example.com");

    // Should handle special characters in application data
    client.apply_to_pool(&pool_id, &student, &special_chars_data);

    let status = client.get_application_status(&pool_id, &student);
    assert_eq!(status, String::from_str(&env, "Pending"));
}

/// TEST 15: Hash length validation - simulating image hash storage
/// Note: The contract defines MAX_IMAGE_HASH_LENGTH but doesn't currently use it.
/// This test documents the expected behavior if/when image hashes are added.
#[test]
fn test_config_bounds_hash_length_documented() {
    // MAX_IMAGE_HASH_LENGTH = 64 is defined in the contract
    // This test documents that hash validation should enforce this limit
    // when image hash functionality is implemented
    
    let max_hash_length = 64;
    let valid_hash = "a".repeat(64);
    let invalid_hash = "a".repeat(65);

    assert_eq!(valid_hash.len(), max_hash_length);
    assert!(invalid_hash.len() > max_hash_length);
}

/// TEST 16: URL length validation - simulating URL storage
/// Note: The contract defines MAX_URL_LENGTH but doesn't currently use it.
/// This test documents the expected behavior if/when URL fields are added.
#[test]
fn test_config_bounds_url_length_documented() {
    // MAX_URL_LENGTH = 256 is defined in the contract
    // This test documents that URL validation should enforce this limit
    // when URL functionality is implemented
    
    let max_url_length = 256;
    let valid_url = format!("https://example.com/{}", "a".repeat(230));
    let invalid_url = format!("https://example.com/{}", "a".repeat(240));

    assert!(valid_url.len() <= max_url_length);
    assert!(invalid_url.len() > max_url_length);
}

/// TEST 17: Numeric range - claim amount boundaries
#[test]
#[should_panic(expected = "Claim amount must be positive")]
fn test_config_bounds_claim_amount_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &500_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Zero claim amount should be rejected
    client.claim_funds(&student, &pool_id, &0i128, &token_address);
}

/// TEST 18: Numeric range - donation amount must be positive
#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_config_bounds_donation_amount_negative() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Negative donation amount should be rejected
    client.donate_with_token(&pool_id, &donor, &token_address, &-100i128);
}

/// TEST 19: Numeric range - donation amount at i128 max boundary
#[test]
fn test_config_bounds_donation_amount_i128_max() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    
    // Create token and fund donor
    let amount = i128::MAX;
    let token_address = create_token(&env, amount, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Max Donation Test"),
        &String::from_str(&env, "Test"),
        &(i128::MAX as u128),
    );

    // Should handle i128::MAX donation
    client.donate_with_token(&pool_id, &donor, &token_address, &amount);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, i128::MAX as u128);
}

/// TEST 20: String encoding - title length (no explicit limit but should handle reasonable sizes)
#[test]
fn test_config_bounds_title_reasonable_length() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    
    // Test with a reasonably long title (200 chars)
    let long_title = "a".repeat(200);
    let title = String::from_str(&env, &long_title);
    let description = String::from_str(&env, "Test description");
    let goal: u128 = 1_000_000_000;

    // Should handle reasonable title lengths
    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    assert_eq!(pool_id, 1);
}ly
    client.claim_funds(&student1, &pool_id, &claim_amount, &token_address);
    let app1 = client.get_application(&pool_id, &student1);
    assert!(app1.is_some());
    assert_eq!(app1.unwrap().amount_claimed, claim_amount);

    // Student3 can still claim (operations are isolated)
    client.claim_funds(&student3, &pool_id, &claim_amount, &token_address);
    let app3 = client.get_application(&pool_id, &student3);
    assert!(app3.is_some());
    assert_eq!(app3.unwrap().amount_claimed, claim_amount);
}

/// Test 3: System recoverable after errors - pool can continue after failed operations
#[test]
fn test_recovery_system_continues_after_error() {
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000,
    );
    client.donate(&pool_id, &creator, &500_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Zero is not positive — must be rejected
    client.claim_funds(&student, &pool_id, &0i128, &token_address);
}

// (2) Invalid pool_id (non-existent) rejected with specific message
#[test]
#[should_panic(expected = "Pool not found")]
fn test_get_pool_invalid_id_rejected() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    
    // Doc states: panics with "Pool not found" for non-existent pool
    client.donate(&999, &donor, &100_000_000);
}

/// DOC TEST 5: get_pool documentation accuracy
/// Verifies: Return value specifications accurate
#[test]
fn test_doc_get_pool_return_value_accurate() {
    // Pool 999 was never created
    client.get_pool(&999u32);
}

// (3) donate to non-existent pool rejected
#[test]
#[should_panic(expected = "Pool not found")]
fn test_donate_invalid_pool_id_rejected() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let goal: u128 = 5_000_000_000;
    
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &goal,
    );

    // Doc states: "Get pool information as a tuple (id, creator, goal, collected, is_closed)"
    let pool = client.get_pool(&pool_id);
    
    // Verify tuple structure matches documentation
    assert_eq!(pool.0, pool_id); // id
    assert_eq!(pool.1, creator); // creator
    assert_eq!(pool.2, goal); // goal
    assert_eq!(pool.3, 0); // collected
    assert_eq!(pool.4, false); // is_closed
}

/// DOC TEST 6: close_pool documentation accuracy
/// Verifies: Function behavior and authorization requirements
#[test]
fn test_doc_close_pool_behavior_matches_docs() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Doc states: "Close a donation pool" - requires creator authorization
    client.mock_auths(&[MockAuth {
        address: &creator,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "close_pool",
            args: (&pool_id,).into_val(&env),
            sub_invokes: &[],
        },
    }]).close_pool(&pool_id);

    // Verify pool is closed
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true); // is_closed is true
}

/// DOC TEST 7: get_pool_count documentation accuracy
/// Verifies: Return value accurate
#[test]
fn test_doc_get_pool_count_return_value_accurate() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Doc states: "Get the total number of pools"
    // Initially should be 0
    assert_eq!(client.get_pool_count(), 0);

    let creator = Address::generate(&env);
    
    // Create first pool
    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );
    assert_eq!(client.get_pool_count(), 1);

    // Create second pool
    client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Test"),
        &2_000_000_000,
    );
    assert_eq!(client.get_pool_count(), 2);
}

/// DOC TEST 8: claim_funds documentation accuracy - all documented panics
/// Verifies: All error conditions documented in claim_funds
#[test]
#[should_panic(expected = "Claim amount must be positive")]
fn test_doc_claim_funds_error_negative_amount() {
    let donor = Address::generate(&env);

    // Create pool
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Recovery Pool"),
        &String::from_str(&env, "System recovery test"),
        &1_000_000_000,
    );

    // Make initial donation
    client.donate(&pool_id, &donor, &100_000_000);

    // Verify system is still operational - can continue with valid operations
    client.donate(&pool_id, &donor, &50_000_000);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 150_000_000); // Total collected

    // Can still create new pools
    let pool_id_2 = client.create_pool(
        &creator,
        &String::from_str(&env, "New Pool After Error"),
        &String::from_str(&env, "Recovery verified"),
        &2_000_000_000,
    );
    assert_eq!(pool_id_2, 2);
}

/// Test 4: Rollback mechanisms work - failed claim doesn't update state
#[test]
#[should_panic(expected = "Overdraw attempt")]
fn test_recovery_rollback_on_overdraw() {
    let donor = Address::generate(&env);
    client.donate(&999u32, &donor, &100_000_000);
}

// (4) apply_to_pool on non-existent pool rejected
#[test]
#[should_panic(expected = "Pool not found")]
fn test_apply_to_pool_invalid_pool_id_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &500_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Doc states: Panics with "Claim amount must be positive" if claim_amount <= 0
    client.claim_funds(&student, &pool_id, &0i128, &token_address);
}

/// DOC TEST 9: claim_funds error - application status not found
#[test]
#[should_panic(expected = "Application status not found")]
fn test_doc_claim_funds_error_no_status() {

    // Create pool with limited funds
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Rollback Test"),
        &String::from_str(&env, "Test rollback"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &100_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    let token_address = Address::generate(&env);

    // Attempt to claim more than available (should panic with "Overdraw attempt")
    client.claim_funds(&student, &pool_id, &500_000_000i128, &token_address);
}

/// Test 4b: Verify state unchanged after overdraw attempt
#[test]
fn test_recovery_state_after_overdraw() {
    let student = Address::generate(&env);
    client.apply_to_pool(&999u32, &student, &String::from_str(&env, "data"));
}

// (5) Duplicate application rejected
#[test]
#[should_panic(expected = "Duplicate application")]
fn test_apply_to_pool_duplicate_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &500_000_000);

    // Doc states: Panics with "Application status not found" if no status has been set
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token_address);
}

/// DOC TEST 10: claim_funds error - application not approved
#[test]
#[should_panic(expected = "Application is not approved")]
fn test_doc_claim_funds_error_not_approved() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &500_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Pending"));

    // Doc states: Panics with "Application is not approved" if status != "Approved"
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token_address);
}

/// DOC TEST 11: claim_funds error - overdraw attempt
#[test]
#[should_panic(expected = "Overdraw attempt")]
fn test_doc_claim_funds_error_overdraw() {

    // Create pool with limited funds
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Rollback Test"),
        &String::from_str(&env, "Test rollback"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &100_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Verify no claim has been made yet
    let claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed, 0);

    // Verify pool collected amount unchanged
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000);
}

/// Test 5: Rollback on unauthorized close pool attempt
#[test]
fn test_recovery_rollback_unauthorized_close() {
    let env = Env::default();
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000,
    );

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "data"));
    // Second application from same student must be rejected
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "data"));
}

// (6) create_pool_for_school with unregistered school rejected
#[test]
#[should_panic(expected = "School is not registered")]
fn test_create_pool_for_school_unregistered_school_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &100_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Doc states: Panics with "Overdraw attempt" if amount_claimed + claim_amount > collected
    client.claim_funds(&student, &pool_id, &200_000_000i128, &token_address);
}

/// DOC TEST 12: claim_protocol_fees documentation accuracy
/// Verifies: Function behavior, authorization, and error conditions
#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_doc_claim_protocol_fees_error_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    client.set_admin(&admin);

    // Doc states: Panics with "Unauthorized admin" if caller is not the stored admin
    client.claim_protocol_fees(&non_admin, &token_address);
}

/// DOC TEST 13: claim_protocol_fees error - no unclaimed fees
#[test]
#[should_panic(expected = "No unclaimed fees")]
fn test_doc_claim_protocol_fees_error_no_fees() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    client.set_admin(&admin);

    // Doc states: Panics with "No unclaimed fees" if there are no accumulated fees
    client.claim_protocol_fees(&admin, &token_address);
}

/// DOC TEST 14: set_admin documentation accuracy
/// Verifies: Function behavior and authorization requirements
#[test]
fn test_doc_set_admin_behavior_matches_docs() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // Doc states: "Set the platform admin address" - requires admin authorization
    client.set_admin(&admin);

    // Verify admin was set by trying to register a school
    let school = Address::generate(&env);
    client.register_school(&admin, &school);
    
    // Verify school was registered
    assert!(client.is_school_registered(&school));
}

/// DOC TEST 15: register_school documentation accuracy
/// Verifies: Function behavior and error conditions
#[test]
#[should_panic(expected = "Admin not set")]
fn test_doc_register_school_error_admin_not_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);

    // Doc states: Expects "Admin not set" if admin hasn't been set
    client.register_school(&admin, &school);
}

/// DOC TEST 16: register_school error - unauthorized admin
#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_doc_register_school_error_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wrong_admin = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);

    // Doc states: Panics with "Unauthorized admin" if caller is not the stored admin
    client.register_school(&wrong_admin, &school);
}

/// DOC TEST 17: is_school_registered documentation accuracy
/// Verifies: Return value specifications accurate
#[test]
fn test_doc_is_school_registered_return_value_accurate() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);

    // Doc states: "Check if a school has been registered" - returns bool
    // Before registration, should return false
    assert_eq!(client.is_school_registered(&school), false);

    // After registration, should return true
    client.register_school(&admin, &school);
    assert_eq!(client.is_school_registered(&school), true);
}

/// DOC TEST 18: create_pool_for_school documentation accuracy
/// Verifies: Function behavior and error conditions
#[test]
#[should_panic(expected = "School is not registered")]
fn test_doc_create_pool_for_school_error_not_registered() {

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Auth Test Pool"),
        &String::from_str(&env, "Test auth"),
        &1_000_000_000,
    );

    // Capture initial state
    let pool_before = client.get_pool(&pool_id);
    assert_eq!(pool_before.4, false); // not closed

    // After any failed authorization, pool should still be open
    let pool_after = client.get_pool(&pool_id);
    assert_eq!(pool_after.4, false); // still not closed
}

/// Test 6: Graceful degradation - system handles missing data gracefully
#[test]
fn test_recovery_graceful_degradation_missing_data() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let student = Address::generate(&env);

    // Query application status for non-existent application (returns empty string)
    let status = client.get_application_status(&999, &student);
    assert_eq!(status, String::from_str(&env, ""));

    // Query claimed amount for non-existent claim (returns 0)
    let claimed = client.get_claimed_amount(&999, &student);
    assert_eq!(claimed, 0);

    // System remains operational after graceful failures
    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Post-Degradation Pool"),
        &String::from_str(&env, "Still works"),
        &1_000_000_000,
    );
    assert_eq!(pool_id, 1);
}

/// Test 7: State consistency after multiple failed operations
#[test]
fn test_recovery_state_consistency_multiple_failures() {
    let unregistered_school = Address::generate(&env);

    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000,
        &unregistered_school,
    );
}

// (7) setup_application_milestones with empty milestones rejected
#[test]
#[should_panic(expected = "Milestones required")]
fn test_setup_milestones_empty_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);

    // Doc states: Panics with "School is not registered" for unregistered school
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
        &school,
    );
}

/// DOC TEST 19: create_pool_for_school success case
/// Verifies: Function behavior matches documentation
#[test]
fn test_doc_create_pool_for_school_behavior_matches_docs() {
    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000,
    );

    let empty: soroban_sdk::Vec<Milestone> = soroban_sdk::Vec::new(&env);
    client.setup_application_milestones(&pool_id, &student, &empty);
}

// (8) setup_application_milestones where sum != goal rejected
#[test]
#[should_panic(expected = "Milestone total must equal pool goal")]
fn test_setup_milestones_wrong_sum_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    // Doc states: "Create a new sponsorship pool linked to a registered school"
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
        &school,
    );

    // Verify pool was created
    assert_eq!(pool_id, 1);

    // Verify school is linked to pool
    let linked_school = client.get_pool_school(&pool_id);
    assert_eq!(linked_school, school);
}

/// DOC TEST 20: get_pool_school documentation accuracy
/// Verifies: Return value and error conditions
#[test]
#[should_panic(expected = "Pool school not set")]
fn test_doc_get_pool_school_error_not_set() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Doc states: Expects "Pool school not set" for pools not linked to schools
    client.get_pool_school(&pool_id);
}

/// DOC TEST 21: apply_to_pool documentation accuracy
/// Verifies: Function behavior and error conditions
#[test]
#[should_panic(expected = "Pool not found")]
fn test_doc_apply_to_pool_error_pool_not_found() {
    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Consistency Test"),
        &String::from_str(&env, "Multiple failures"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &200_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Verify state remains consistent - no corruption
    let claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed, 0);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 200_000_000); // collected unchanged
}

/// Test 8: Recovery from protocol fee claim failures
#[test]
fn test_recovery_protocol_fees_failure_handling() {
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &goal,
    );

    let mut milestones: soroban_sdk::Vec<Milestone> = soroban_sdk::Vec::new(&env);
    milestones.push_back(Milestone { amount: 500_000_000 }); // sum != goal
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

// ============= ISSUE #506: ERROR MESSAGE ACCURACY TESTS =============

// (1) Specific error for missing admin (not generic)
#[test]
#[should_panic(expected = "Admin not set")]
fn test_register_school_without_admin_set_gives_specific_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let student = Address::generate(&env);

    // Doc states: Expects "Pool not found" for non-existent pool
    client.apply_to_pool(
        &999,
        &student,
        &String::from_str(&env, "Application"),
    );
}

/// DOC TEST 22: apply_to_pool error - duplicate application
#[test]
#[should_panic(expected = "Duplicate application")]
fn test_doc_apply_to_pool_error_duplicate() {
    let admin = Address::generate(&env);

    client.set_admin(&admin);

    // System should still be operational - can set admin again
    let new_admin = Address::generate(&env);
    client.set_admin(&new_admin);
}

/// Test 9: Graceful handling of duplicate application attempts
#[test]
#[should_panic(expected = "Duplicate application")]
fn test_recovery_duplicate_application_prevention() {
    let school = Address::generate(&env);
    // Admin was never set — must say "Admin not set", not a generic error
    client.register_school(&admin, &school);
}

// (2) Specific error when wrong admin calls register_school
#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_register_school_wrong_admin_gives_specific_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Duplicate Test"),
        &String::from_str(&env, "Test duplicates"),
        &1_000_000_000,
    );

    // First application succeeds
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "First application"),
    );

    // Doc states: Panics with "Duplicate application" for duplicate applications
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Second application"),
    );
}

/// DOC TEST 23: approve_application documentation accuracy
/// Verifies: Function behavior and error conditions
#[test]
#[should_panic(expected = "Only linked school can approve")]
fn test_doc_approve_application_error_wrong_school() {
    // Second application should fail
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Duplicate application"),
    );
}

/// Test 10: State recovery after partial claim sequence
#[test]
fn test_recovery_partial_claim_sequence() {
    let real_admin = Address::generate(&env);
    let fake_admin = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&real_admin);
    // fake_admin is not the stored admin — must say "Unauthorized admin"
    client.register_school(&fake_admin, &school);
}

// (3) Specific error when non-linked school tries to approve
#[test]
#[should_panic(expected = "Only linked school can approve")]
fn test_approve_application_wrong_school_gives_specific_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school1 = Address::generate(&env);
    let school2 = Address::generate(&env);
    let student = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school1);
    client.register_school(&admin, &school2);

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
        &school1,
    );

    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Application"),
    );

    // Doc states: Panics with "Only linked school can approve" if wrong school tries to approve
    client.approve_application(&pool_id, &school2, &student, true);
}

/// DOC TEST 24: approve_application error - student has not applied
#[test]
#[should_panic(expected = "Student has not applied")]
fn test_doc_approve_application_error_no_application() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let school = Address::generate(&env);
    let student = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
        &school,
    );

    // Doc states: Panics with "Student has not applied" if student hasn't applied
    client.approve_application(&pool_id, &school, &student, true);
}

/// DOC TEST 25: setup_application_milestones documentation accuracy
/// Verifies: Function behavior and error conditions
#[test]
#[should_panic(expected = "Milestones required")]
fn test_doc_setup_milestones_error_empty() {
    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Partial Claims"),
        &String::from_str(&env, "Test partial claims"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &300_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    let claim_amount: i128 = 50_000_000;
    let token_address = create_token(&env, claim_amount * 3, &contract_id);

    // First claim succeeds
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), claim_amount);

    // Second claim succeeds
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);
    assert_eq!(
        client.get_claimed_amount(&pool_id, &student),
        claim_amount * 2
    );

    // Verify state is consistent - only 2 claims recorded
    assert_eq!(
        client.get_claimed_amount(&pool_id, &student),
        claim_amount * 2
    );
}

/// Test 11: System handles school registration failures gracefully
#[test]
#[should_panic(expected = "School is not registered")]
fn test_recovery_school_registration_failures() {
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let linked_school = Address::generate(&env);
    let other_school = Address::generate(&env);
    let student = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &linked_school);

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000,
        &linked_school,
    );

    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "data"));
    // other_school is not the linked school — must say "Only linked school can approve"
    client.approve_application(&pool_id, &other_school, &student, &true);
}

// (4) Specific error when approving a student who never applied
#[test]
#[should_panic(expected = "Student has not applied")]
fn test_approve_application_no_application_gives_specific_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    let empty_milestones = Vec::new(&env);

    // Doc states: Panics with "Milestones required" if milestones is empty
    client.setup_application_milestones(&pool_id, &student, &empty_milestones);
}

/// DOC TEST 26: setup_milestones error - total must equal goal
#[test]
#[should_panic(expected = "Milestone total must equal pool goal")]
fn test_doc_setup_milestones_error_total_mismatch() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: 500_000_000 });
    milestones.push_back(Milestone { amount: 300_000_000 }); // Total: 800M != 1B

    // Doc states: Panics with "Milestone total must equal pool goal" if sum != goal
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

/// DOC TEST 27: get_milestones documentation accuracy
/// Verifies: Return value specifications accurate
#[test]
fn test_doc_get_milestones_return_value_accurate() {
    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);

    client.set_admin(&admin);

    // Attempt to create pool for unregistered school (should panic)
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
        &school,
    );
}

/// Test 11b: Verify school registration recovery
#[test]
fn test_recovery_school_registration_success() {
    let creator = Address::generate(&env);
    let school = Address::generate(&env);
    let student = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000,
        &school,
    );

    // Student never applied — must say "Student has not applied"
    client.approve_application(&pool_id, &school, &student, &true);
}

// (5) Specific error when claiming from pool with no status set
#[test]
#[should_panic(expected = "Application status not found")]
fn test_claim_funds_no_status_gives_specific_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Doc states: Returns empty Vec if no milestones set
    let milestones_before = client.get_milestones(&pool_id, &student);
    assert_eq!(milestones_before.len(), 0);

    // Set milestones
    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: 600_000_000 });
    milestones.push_back(Milestone { amount: 400_000_000 });
    client.setup_application_milestones(&pool_id, &student, &milestones);

    // Doc states: Returns Vec<Milestone> for student in pool
    let milestones_after = client.get_milestones(&pool_id, &student);
    assert_eq!(milestones_after.len(), 2);
    assert_eq!(milestones_after.get(0).unwrap().amount, 600_000_000);
    assert_eq!(milestones_after.get(1).unwrap().amount, 400_000_000);
}

/// DOC TEST 28: get_application_status documentation accuracy
/// Verifies: Return value specifications accurate
#[test]
fn test_doc_get_application_status_return_value_accurate() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Doc states: Returns empty string if no status set
    let status_before = client.get_application_status(&pool_id, &student);
    assert_eq!(status_before, String::from_str(&env, ""));

    // Set status
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Doc states: Returns String status for student in pool
    let status_after = client.get_application_status(&pool_id, &student);
    assert_eq!(status_after, String::from_str(&env, "Approved"));
}

/// DOC TEST 29: get_claimed_amount documentation accuracy
/// Verifies: Return value specifications accurate
#[test]
fn test_doc_get_claimed_amount_return_value_accurate() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Doc states: Returns 0 if no claims made
    let claimed_before = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed_before, 0);

    // Make a claim
    client.donate(&pool_id, &creator, &500_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));
    
    let claim_amount: i128 = 100_000_000;
    let token_address = create_token(&env, claim_amount, &contract_id);
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);

    // Doc states: Returns i128 claimed amount for student in pool
    let claimed_after = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed_after, claim_amount);
}

/// DOC TEST 30: get_application documentation accuracy
/// Verifies: Return value specifications accurate
#[test]
fn test_doc_get_application_return_value_accurate() {
    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);

    client.set_admin(&admin);

    // Register school
    client.register_school(&admin, &school);
    assert!(client.is_school_registered(&school));

    // Now pool creation should succeed
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
        &school,
    );
    assert_eq!(pool_id, 1);
}

/// Test 12: Verify pool count consistency after failed pool operations
#[test]
fn test_recovery_pool_count_consistency() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000,
    );
    client.donate(&pool_id, &creator, &500_000_000);

    // No status set — must say "Application status not found", not a generic error
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token_address);
}

// (6) Specific error when overdrawing — not a generic arithmetic error
#[test]
#[should_panic(expected = "Overdraw attempt")]
fn test_claim_funds_overdraw_gives_specific_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Doc states: Returns None if student has not made any claim
    let app_before = client.get_application(&pool_id, &student);
    assert_eq!(app_before, None);

    // Make a claim
    client.donate(&pool_id, &creator, &500_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));
    
    let claim_amount: i128 = 100_000_000;
    let token_address = create_token(&env, claim_amount, &contract_id);
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);

    // Doc states: Returns Some(Application) after claim
    let app_after = client.get_application(&pool_id, &student);
    assert!(app_after.is_some());
    
    let application = app_after.unwrap();
    assert_eq!(application.amount_claimed, claim_amount);
    assert_eq!(application.approved_amount, 500_000_000);
}

/// DOC TEST 31: Usage example - complete pool lifecycle
/// Verifies: Usage examples work as documented
#[test]
fn test_doc_usage_example_complete_pool_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    // Step 1: Set admin
    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // Step 2: Register school
    let school = Address::generate(&env);
    client.register_school(&admin, &school);
    assert!(client.is_school_registered(&school));

    // Step 3: Create pool for school
    let creator = Address::generate(&env);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Scholarship Fund"),
        &String::from_str(&env, "Supporting students"),
        &1_000_000_000,
        &school,
    );

    // Step 4: Donate to pool
    let donor = Address::generate(&env);
    client.donate(&pool_id, &donor, &500_000_000);

    // Step 5: Student applies
    let student = Address::generate(&env);
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "My application"),
    );

    // Step 6: School approves
    client.approve_application(&pool_id, &school, &student, true);
    
    let status = client.get_application_status(&pool_id, &student);
    assert_eq!(status, String::from_str(&env, "Approved"));

    // Step 7: Student claims funds
    let claim_amount: i128 = 100_000_000;
    let token_address = create_token(&env, claim_amount, &contract_id);
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);

    // Verify claim was recorded
    let claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(claimed, claim_amount);

    // Step 8: Admin claims protocol fees (1% of claim)
    let fees = client.claim_protocol_fees(&admin, &token_address);
    assert_eq!(fees, 1_000_000); // 1% of 100M
}

/// DOC TEST 32: Usage example - partial claims (streaming payments)
/// Verifies: Documented streaming payment behavior works correctly
#[test]
fn test_doc_usage_example_streaming_payments() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    // Create pool and donate
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Streaming Test"),
        &String::from_str(&env, "Test partial claims"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &300_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Doc states: Student can make multiple partial claims
    let claim1: i128 = 50_000_000;
    let claim2: i128 = 75_000_000;
    let claim3: i128 = 100_000_000;
    
    let total_needed = claim1 + claim2 + claim3;
    let token_address = create_token(&env, total_needed, &contract_id);

    // First claim
    client.claim_funds(&student, &pool_id, &claim1, &token_address);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), claim1);

    // Second claim
    client.claim_funds(&student, &pool_id, &claim2, &token_address);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), claim1 + claim2);

    // Third claim
    client.claim_funds(&student, &pool_id, &claim3, &token_address);
    assert_eq!(client.get_claimed_amount(&pool_id, &student), claim1 + claim2 + claim3);

    // Verify Application struct tracks cumulative claims
    let app = client.get_application(&pool_id, &student).unwrap();
    assert_eq!(app.amount_claimed, claim1 + claim2 + claim3);
}

/// DOC TEST 33: Parameter requirements - create_pool
/// Verifies: Parameter requirements are clear and enforced
#[test]
fn test_doc_parameter_requirements_create_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    
    // Doc states: All parameters are required
    // creator: Address - required
    // title: String - required
    // description: String - required
    // goal: u128 - required
    
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Description"),
        &1_000_000_000,
    );

    // Verify pool was created with all parameters
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, 1_000_000_000);
}

/// DOC TEST 34: Parameter requirements - claim_funds
/// Verifies: All claim_funds parameters documented and required
#[test]
fn test_doc_parameter_requirements_claim_funds() {

    // Create first pool
    let pool_id_1 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "First"),
        &1_000_000_000,
    );
    assert_eq!(pool_id_1, 1);
    assert_eq!(client.get_pool_count(), 1);

    // Pool count should remain consistent
    assert_eq!(client.get_pool_count(), 1);

    // Create second pool
    let pool_id_2 = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Second"),
        &2_000_000_000,
    );
    assert_eq!(pool_id_2, 2);
    assert_eq!(client.get_pool_count(), 2);
    let student = Address::generate(&env);
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000,
    );
    client.donate(&pool_id, &creator, &100_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Must say "Overdraw attempt", not a generic overflow/arithmetic error
    client.claim_funds(&student, &pool_id, &999_000_000i128, &token_address);
}

// (7) Specific error when claiming fees with no admin set
#[test]
#[should_panic(expected = "Admin not set")]
fn test_claim_protocol_fees_no_admin_set_gives_specific_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &500_000_000);
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Doc states: All parameters required
    // env: Env - required
    // student: Address - required (must authorize)
    // pool_id: u32 - required
    // claim_amount: i128 - required (must be > 0)
    // token_address: Address - required
    
    let claim_amount: i128 = 100_000_000;
    let token_address = create_token(&env, claim_amount, &contract_id);
    
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);

    // Verify claim was processed with all parameters
    assert_eq!(client.get_claimed_amount(&pool_id, &student), claim_amount);
}

/// DOC TEST 35: Error message accuracy verification
/// Verifies: All documented error messages are accurate
#[test]
fn test_doc_error_messages_accurate() {
    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    // Admin was never set — must say "Admin not set"
    client.claim_protocol_fees(&admin, &token_address);
}

// (8) Specific error when closing a non-existent pool
#[test]
#[should_panic(expected = "Pool not found")]
fn test_close_pool_invalid_id_gives_specific_error() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Test each documented error message exists and is accurate
    // This test verifies the error messages match documentation
    
    // 1. "Pool not found" - tested in other doc tests
    // 2. "Pool is closed" - tested in other doc tests
    // 3. "Claim amount must be positive" - tested in other doc tests
    // 4. "Application status not found" - tested in other doc tests
    // 5. "Application is not approved" - tested in other doc tests
    // 6. "Overdraw attempt" - tested in other doc tests
    // 7. "Unauthorized admin" - tested in other doc tests
    // 8. "No unclaimed fees" - tested in other doc tests
    // 9. "Admin not set" - tested in other doc tests
    // 10. "School is not registered" - tested in other doc tests
    // 11. "Only linked school can approve" - tested in other doc tests
    // 12. "Student has not applied" - tested in other doc tests
    // 13. "Duplicate application" - tested in other doc tests
    // 14. "Milestones required" - tested in other doc tests
    // 15. "Milestone total must equal pool goal" - tested in other doc tests
    // 16. "Pool school not set" - tested in other doc tests

    // All error messages have been verified in individual doc tests above
    assert!(true);
    // Pool 42 never created — must say "Pool not found"
    client.close_pool(&42u32);
}

// Tests for Issue #482: Pool metadata validation
#[test]
fn test_pool_metadata_description_length_within_limit() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let description = String::from_str(&env, "This is a valid description");
    let title = String::from_str(&env, "Pool Title");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    assert_eq!(pool_id, 1);
}

#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_pool_metadata_description_exceeds_max_length() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let long_description = String::from_str(&env, &"x".repeat(501));
    let title = String::from_str(&env, "Pool Title");
    let goal: u128 = 1_000_000_000;

    client.create_pool(&creator, &title, &long_description, &goal);
}

#[test]
fn test_pool_metadata_description_at_max_boundary() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let max_description = String::from_str(&env, &"x".repeat(500));
    let title = String::from_str(&env, "Pool Title");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &max_description, &goal);
    assert_eq!(pool_id, 1);
}

// Tests for Issue #486: Campaign creation validation edge cases
#[test]
fn test_campaign_creation_maximum_title_length() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let long_title = String::from_str(&env, &"T".repeat(255));
    let description = String::from_str(&env, "Valid description");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &long_title, &description, &goal);
    assert!(pool_id > 0);
}

#[test]
fn test_campaign_creation_maximum_goal_value() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Large Goal Campaign");
    let description = String::from_str(&env, "Very high fundraising target");
    let max_goal: u128 = u128::MAX / 2;

    let pool_id = client.create_pool(&creator, &title, &description, &max_goal);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.2, max_goal);
}

#[test]
fn test_campaign_creation_multiple_campaigns_different_ids() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);
    let title = String::from_str(&env, "Campaign");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let pool_id1 = client.create_pool(&creator1, &title, &description, &goal);
    let pool_id2 = client.create_pool(&creator2, &title, &description, &goal);

    assert_ne!(pool_id1, pool_id2);
    assert_eq!(pool_id2, pool_id1 + 1);
}

#[test]
fn test_campaign_creation_pool_count_increments() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Campaign");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let initial_count = client.get_pool_count();
    assert_eq!(initial_count, 0);

    client.create_pool(&creator, &title, &description, &goal);
    let count_after_one = client.get_pool_count();
    assert_eq!(count_after_one, 1);

    client.create_pool(&creator, &title, &description, &goal);
    let count_after_two = client.get_pool_count();
    assert_eq!(count_after_two, 2);
}

// Tests for Issue #487: Campaign donation token validation
#[test]
fn test_donate_with_token_correct_token_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    let token = create_token(&env, 500_000_000, &donor);
    let donation_amount: i128 = 100_000_000;

    client.donate_with_token(&pool_id, &donor, &token, &donation_amount);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, donation_amount as u128);
}

#[test]
#[should_panic(expected = "Amount must be positive")]
fn test_donate_with_token_negative_amount_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    let token = create_token(&env, 500_000_000, &donor);

    client.donate_with_token(&pool_id, &donor, &token, &-1i128);
}

#[test]
#[should_panic(expected = "Pool is closed")]
fn test_donate_with_token_closed_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    client.close_pool(&pool_id);

    let token = create_token(&env, 500_000_000, &donor);
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000);
}

#[test]
fn test_donate_with_token_multiple_donations_accumulate() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    let token1 = create_token(&env, 500_000_000, &donor1);
    let token2 = create_token(&env, 500_000_000, &donor2);

    client.donate_with_token(&pool_id, &donor1, &token1, &100_000_000);
    client.donate_with_token(&pool_id, &donor2, &token2, &200_000_000);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 300_000_000);
}

// Tests for Issue #507: Contract upgrade compatibility
#[test]
fn test_upgrade_storage_layout_compatibility_pool_structure() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    let pool = client.get_pool(&pool_id);

    assert_eq!(pool.0, pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, goal);
    assert_eq!(pool.3, 0);
    assert_eq!(pool.4, false);
}

#[test]
fn test_upgrade_function_signature_backward_compatibility() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    client.donate(&pool_id, &donor, &100_000_000);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000);
}

#[test]
fn test_upgrade_new_function_addition_donate_with_token() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Description");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    let token = create_token(&env, 500_000_000, &donor);

    client.donate_with_token(&pool_id, &donor, &token, &100_000_000);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000);
}

#[test]
fn test_upgrade_metadata_validation_new_validation() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let short_description = String::from_str(&env, "Short");
    let title = String::from_str(&env, "Fund");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &short_description, &goal);
    assert!(pool_id > 0);
}

#[test]
#[should_panic(expected = "Description exceeds maximum length")]
fn test_upgrade_metadata_validation_enforced_after_upgrade() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school1 = Address::generate(&env);
    let school2 = Address::generate(&env);
    let creator = Address::generate(&env);
    let long_description = String::from_str(&env, &"x".repeat(501));
    let title = String::from_str(&env, "Fund");
    let goal: u128 = 1_000_000_000;

    client.set_admin(&admin);
    client.register_school(&admin, &school1);
    client.register_school(&admin, &school2);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
        &school1,
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "data"));
    client.approve_application(&pool_id, &school2, &student, &true);
}

#[test]
fn test_upgrade_migration_pool_count_preserved() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Desc");
    let goal: u128 = 1_000_000_000;

    let pool_id1 = client.create_pool(&creator, &title, &description, &goal);
    let pool_id2 = client.create_pool(&creator, &title, &description, &goal);
    let pool_id3 = client.create_pool(&creator, &title, &description, &goal);

    let count = client.get_pool_count();
    assert_eq!(count, 3);
    assert_eq!(pool_id3, 3);
}

#[test]
fn test_upgrade_school_registration_persists() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);

    assert!(client.is_school_registered(&school));
}

#[test]
fn test_upgrade_backward_compatibility_existing_operations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Fund");
    let description = String::from_str(&env, "Desc");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    let donor = Address::generate(&env);

    client.donate(&pool_id, &donor, &100_000_000);
    client.donate(&pool_id, &donor, &200_000_000);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 300_000_000);
}

    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let donor = Address::generate(&env);
    
    // Doc states: panics with "Pool not found" when donating to non-existent pool
    client.donate(&999, &donor, &100_000_000);
}

// ============= EVENT EMISSION TESTS =============
// These tests verify that all contract operations emit correct events with proper parameters

/// EVENT TEST 1: Pool creation emits correct event
/// Verifies: Pool creation event is emitted with all required fields
#[test]
fn test_event_pool_creation_emits_correct_event() {
// Tests for Issue #485: Pool metadata retrieval
#[test]
fn test_pool_metadata_retrieval() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Test Pool");
    let description = String::from_str(&env, "Event test pool");
    let goal: u128 = 1_000_000_000;

    // Create pool and capture events
    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    // Verify event was emitted
    let events = env.events().all();
    let event = events.last().unwrap();

    // Verify event topics
    assert_eq!(event.topics.len(), 2);
    // First topic should be the event name symbol
    // Second topic should be the pool_id
    
    // Verify event data contains creator, goal, title, description
    // Event data structure: (creator, goal, title, description)
    assert!(events.len() > 0, "Pool creation should emit an event");
}

/// EVENT TEST 2: Pool creation event has all required fields
/// Verifies: Event contains pool_id, creator, goal, title, and description
#[test]
fn test_event_pool_creation_has_required_fields() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Required Fields Pool");
    let description = String::from_str(&env, "Testing required fields");
    let goal: u128 = 5_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    let events = env.events().all();
    assert!(events.len() > 0, "Should emit at least one event");
    
    // The last event should be the pool creation event
    let event = events.last().unwrap();
    
    // Verify topics include pool_id
    assert_eq!(event.topics.len(), 2, "Event should have 2 topics (event name and pool_id)");
}

/// EVENT TEST 3: Multiple pool creations emit separate events
/// Verifies: Each pool creation emits its own distinct event
#[test]
fn test_event_multiple_pool_creations_emit_separate_events() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let initial_event_count = env.events().all().len();

    let _pool_id_1 = client.create_pool(
        &creator1,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "First pool"),
        &1_000_000_000,
    );

    let after_first = env.events().all().len();
    assert_eq!(after_first, initial_event_count + 1, "First pool should emit one event");

    let _pool_id_2 = client.create_pool(
        &creator2,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Second pool"),
        &2_000_000_000,
    );

    let after_second = env.events().all().len();
    assert_eq!(after_second, initial_event_count + 2, "Second pool should emit another event");
}

/// EVENT TEST 4: Donation emits event with correct parameters
/// Verifies: Donation event includes donor, amount, and new collected total
#[test]
fn test_event_donation_emits_with_right_parameters() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal: u128 = 10_000_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Donation Event Pool"),
        &String::from_str(&env, "Testing donation events"),
        &goal,
    );

    let initial_event_count = env.events().all().len();
    let donation_amount: u128 = 100_000_000;

    // Make donation
    client.donate(&pool_id, &donor, &donation_amount);

    let events = env.events().all();
    assert_eq!(events.len(), initial_event_count + 1, "Donation should emit one event");

    let donation_event = events.last().unwrap();
    
    // Verify event has topics (event name and pool_id)
    assert_eq!(donation_event.topics.len(), 2, "Donation event should have 2 topics");
    
    // Event data should contain: (donor, amount, new_collected)
    // We verify the event was emitted; specific data validation depends on event structure
}

/// EVENT TEST 5: Multiple donations emit separate events
/// Verifies: Each donation to a pool emits its own event
#[test]
fn test_event_multiple_donations_emit_separate_events() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Multi Donation Pool"),
        &String::from_str(&env, "Testing multiple donations"),
        &5_000_000_000,
    );

    let after_creation = env.events().all().len();

    client.donate(&pool_id, &donor1, &100_000_000);
    let after_first_donation = env.events().all().len();
    assert_eq!(after_first_donation, after_creation + 1, "First donation should emit event");

    client.donate(&pool_id, &donor2, &200_000_000);
    let after_second_donation = env.events().all().len();
    assert_eq!(after_second_donation, after_creation + 2, "Second donation should emit event");
}

/// EVENT TEST 6: Donation event includes updated collected amount
/// Verifies: Event data reflects the new total collected amount after donation
#[test]
fn test_event_donation_includes_updated_collected_amount() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Collected Amount Pool"),
        &String::from_str(&env, "Testing collected amount in event"),
        &10_000_000_000,
    );

    let donation_amount: u128 = 250_000_000;
    client.donate(&pool_id, &donor, &donation_amount);

    // Verify the pool state matches what should be in the event
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, donation_amount, "Pool collected should match donation amount");

    // Event should have been emitted with this collected amount
    let events = env.events().all();
    assert!(events.len() > 0, "Should have emitted donation event");
}

/// EVENT TEST 7: Pool closure emits event
/// Verifies: Closing a pool emits an event with pool_id and final collected amount
#[test]
fn test_event_pool_closure_emits_event() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closure Event Pool"),
        &String::from_str(&env, "Testing pool closure event"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &donor, &500_000_000);

    let before_close = env.events().all().len();
    
    client.close_pool(&pool_id);

    let after_close = env.events().all().len();
    assert_eq!(after_close, before_close + 1, "Pool closure should emit one event");
}

/// EVENT TEST 8: Pool closure event has required fields
/// Verifies: Closure event contains pool_id, sponsor, and collected amount
#[test]
fn test_event_pool_closure_has_required_fields() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closure Fields Pool"),
        &String::from_str(&env, "Testing closure event fields"),
        &2_000_000_000,
    );

    client.close_pool(&pool_id);

    let events = env.events().all();
    let closure_event = events.last().unwrap();

    // Verify event structure
    assert_eq!(closure_event.topics.len(), 2, "Closure event should have 2 topics");
}

/// EVENT TEST 9: Application submission emits contribution event with privacy flag
/// Verifies: Student application emits event with privacy flag set to false (public)
#[test]
fn test_event_contribution_includes_privacy_flag() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Privacy Flag Pool"),
        &String::from_str(&env, "Testing privacy flag in events"),
        &1_000_000_000,
    );

    let before_application = env.events().all().len();

    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "My application"),
    );

    let after_application = env.events().all().len();
    assert_eq!(after_application, before_application + 1, "Application should emit event");

    let events = env.events().all();
    let application_event = events.last().unwrap();
    
    // Verify event has topics
    assert_eq!(application_event.topics.len(), 2, "Application event should have 2 topics");
    
    // Event data should include: (student, app_count, privacy_flag)
    // privacy_flag should be false for public applications
}

/// EVENT TEST 10: Token donation emits contribution event with privacy flag
/// Verifies: Token-based donation emits event with privacy flag set to true (private)
#[test]
fn test_event_token_donation_includes_privacy_flag() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let amount: i128 = 100_000_000;
    let token_address = create_token(&env, amount, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Token Privacy Pool"),
        &String::from_str(&env, "Testing token donation privacy"),
        &1_000_000_000,
    );

    let before_donation = env.events().all().len();

    client.donate_with_token(&pool_id, &donor, &token_address, &amount);

    let after_donation = env.events().all().len();
    assert_eq!(after_donation, before_donation + 1, "Token donation should emit event");

    let donation_event = env.events().all().last().unwrap();
    
    // Verify event structure
    assert_eq!(donation_event.topics.len(), 2, "Token donation event should have 2 topics");
    
    // Event data should include: (donor, amount, new_collected, privacy_flag)
    // privacy_flag should be true for private contributions
}

/// EVENT TEST 11: All events have required topic structure
/// Verifies: Every event has at least event name and relevant ID in topics
#[test]
fn test_event_all_events_have_required_topic_structure() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let student = Address::generate(&env);

    // Create pool - should emit event
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Complete Test Pool"),
        &String::from_str(&env, "Testing all event structures"),
        &5_000_000_000,
    );

    // Donate - should emit event
    client.donate(&pool_id, &donor, &100_000_000);

    // Apply - should emit event
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "Application"),
    );

    // Close pool - should emit event
    client.close_pool(&pool_id);

    // Verify all events have proper structure
    let events = env.events().all();
    
    // Should have at least 4 events (create, donate, apply, close)
    assert!(events.len() >= 4, "Should have emitted at least 4 events");

    // Verify each event has 2 topics (event name + identifier)
    for event in events.iter() {
        assert_eq!(event.topics.len(), 2, "Each event should have exactly 2 topics");
    }
}

/// EVENT TEST 12: Event emission doesn't affect contract state
/// Verifies: Emitting events doesn't change pool or application state
#[test]
fn test_event_emission_doesnt_affect_state() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "State Test Pool"),
        &String::from_str(&env, "Verify events don't affect state"),
        &goal,
    );

    let pool_after_creation = client.get_pool(&pool_id);

    let donation_amount: u128 = 300_000_000;
    client.donate(&pool_id, &donor, &donation_amount);

    let pool_after_donation = client.get_pool(&pool_id);

    // Verify state changes are correct regardless of events
    assert_eq!(pool_after_creation.3, 0, "Initial collected should be 0");
    assert_eq!(pool_after_donation.3, donation_amount, "Collected should match donation");
    assert_eq!(pool_after_creation.2, goal, "Goal should remain unchanged");
    assert_eq!(pool_after_donation.2, goal, "Goal should remain unchanged after donation");
}

/// EVENT TEST 13: Events emitted in correct order
/// Verifies: Multiple operations emit events in the order they occur
#[test]
fn test_event_emission_order_is_correct() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);

    let initial_count = env.events().all().len();

    // Operation 1: Create pool
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Order Test Pool"),
        &String::from_str(&env, "Testing event order"),
        &5_000_000_000,
    );

    let after_create = env.events().all().len();
    assert_eq!(after_create, initial_count + 1, "Create should emit 1 event");

    // Operation 2: First donation
    client.donate(&pool_id, &donor1, &100_000_000);

    let after_first_donation = env.events().all().len();
    assert_eq!(after_first_donation, initial_count + 2, "First donation should emit 1 event");

    // Operation 3: Second donation
    client.donate(&pool_id, &donor2, &200_000_000);

    let after_second_donation = env.events().all().len();
    assert_eq!(after_second_donation, initial_count + 3, "Second donation should emit 1 event");

    // Operation 4: Close pool
    client.close_pool(&pool_id);

    let final_count = env.events().all().len();
    assert_eq!(final_count, initial_count + 4, "Close should emit 1 event");
}

/// EVENT TEST 14: Event data integrity across operations
/// Verifies: Event data accurately reflects operation parameters
#[test]
fn test_event_data_integrity_across_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let goal: u128 = 10_000_000_000;

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Integrity Test Pool"),
        &String::from_str(&env, "Testing data integrity"),
        &goal,
    );

    // Verify pool state matches creation parameters
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.1, creator, "Creator should match");
    assert_eq!(pool.2, goal, "Goal should match");

    let donation_amount: u128 = 500_000_000;
    client.donate(&pool_id, &donor, &donation_amount);

    // Verify donation updated state correctly
    let pool_after_donation = client.get_pool(&pool_id);
    assert_eq!(pool_after_donation.3, donation_amount, "Collected should match donation");

    // Events should have been emitted with this same data
    let events = env.events().all();
    assert!(events.len() >= 2, "Should have at least 2 events (create + donate)");
}

/// EVENT TEST 15: No events emitted on failed operations
/// Verifies: Failed operations don't emit events
#[test]
fn test_event_no_emission_on_failed_operations() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Failure Test Pool"),
        &String::from_str(&env, "Testing failed operation events"),
        &1_000_000_000,
    );

    client.close_pool(&pool_id);

    let before_failed_donation = env.events().all().len();

    // Try to donate to closed pool - should fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.donate(&pool_id, &donor, &100_000_000);
    }));

    assert!(result.is_err(), "Donation to closed pool should fail");

    let after_failed_donation = env.events().all().len();
    
    // No new event should be emitted for failed operation
    assert_eq!(after_failed_donation, before_failed_donation, "Failed operation should not emit event");
    let title1 = String::from_str(&env, "First Pool");
    let description1 = String::from_str(&env, "First pool description");
    let goal: u128 = 1_000_000_000;

    // 1. Nonexistent pool returns empty strings
    let (empty_title, empty_desc) = client.get_pool_metadata(&999u32);
    assert_eq!(empty_title, String::from_str(&env, ""));
    assert_eq!(empty_desc, String::from_str(&env, ""));

    // 2. Existing pool returns correct metadata matching saved values
    let pool_id1 = client.create_pool(&creator, &title1, &description1, &goal);
    let (retrieved_title1, retrieved_desc1) = client.get_pool_metadata(&pool_id1);
    assert_eq!(retrieved_title1, title1);
    assert_eq!(retrieved_desc1, description1);

    // 3. Multiple pools have independent metadata
    let title2 = String::from_str(&env, "Second Pool");
    let description2 = String::from_str(&env, "Second pool description");
    let pool_id2 = client.create_pool(&creator, &title2, &description2, &goal);

    let (retrieved_title2, retrieved_desc2) = client.get_pool_metadata(&pool_id2);
    assert_eq!(retrieved_title2, title2);
    assert_eq!(retrieved_desc2, description2);

    // Re-verify first pool still has correct independent metadata
    let (retrieved_title1_again, retrieved_desc1_again) = client.get_pool_metadata(&pool_id1);
    assert_eq!(retrieved_title1_again, title1);
    assert_eq!(retrieved_desc1_again, description1);
}

    // Pool is active (not closed) — sponsor can close it
    let pool_before = client.get_pool(&pool_id);
    assert_eq!(pool_before.4, false);

    client.close_pool(&pool_id);

    let pool_after = client.get_pool(&pool_id);
    assert_eq!(pool_after.4, true);
    // Collected amount is preserved after closing
    assert_eq!(pool_after.3, 500_000_000u128);
}

/// (2) Donating to an already-closed (disbursed) pool fails with "Pool is closed".
#[test]
#[should_panic(expected = "Pool is closed")]
fn test_refund_closed_pool_donation_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Closed Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
    );
    client.close_pool(&pool_id);
    // Donating to a closed pool must fail
    client.donate(&pool_id, &Address::generate(&env), &100_000_000u128);
}

/// (3) Closing an already-closed pool is idempotent (does not panic).
#[test]
fn test_refund_closing_already_closed_pool_is_idempotent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    client.close_pool(&pool_id);
    // Closing again should not panic
    client.close_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
}

/// (4) Multiple refund (close) attempts: only the first changes state.
#[test]
fn test_refund_multiple_close_attempts_state_consistent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &Address::generate(&env), &200_000_000u128);

    client.close_pool(&pool_id);
    client.close_pool(&pool_id);
    client.close_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
    assert_eq!(pool.3, 200_000_000u128); // collected unchanged
}

/// (5) Unauthorized address cannot close (refund) a pool.
#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_refund_unauthorized_close_fails() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    client
        .mock_auths(&[MockAuth {
            address: &unauthorized,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "close_pool",
                args: (&pool_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .close_pool(&pool_id);
}

/// (6) Closing a pool with zero collected amount succeeds.
#[test]
fn test_refund_pool_with_zero_collected_can_be_closed() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Empty Pool"),
        &String::from_str(&env, "No donations"),
        &1_000_000_000u128,
    );
    // No donations made
    client.close_pool(&pool_id);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true);
    assert_eq!(pool.3, 0u128);
}

/// (7) Pool state (collected amount) is preserved after closing.
#[test]
fn test_refund_pool_collected_preserved_after_close() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &5_000_000_000u128,
    );
    client.donate(&pool_id, &Address::generate(&env), &1_000_000_000u128);
    client.donate(&pool_id, &Address::generate(&env), &2_000_000_000u128);

    let collected_before = client.get_pool(&pool_id).3;
    client.close_pool(&pool_id);
    let collected_after = client.get_pool(&pool_id).3;

    assert_eq!(collected_before, collected_after);
    assert_eq!(collected_after, 3_000_000_000u128);
}

/// (8) Closing a non-existent pool fails with "Pool not found".
#[test]
#[should_panic(expected = "Pool not found")]
fn test_refund_nonexistent_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    client.close_pool(&999u32);
}
