#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token::StellarAssetClient,
    Address, Env, IntoVal, String,
};

/// Create a mock SAC token, mint `amount` into `recipient`, and return the token address.
fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

#[test]
fn test_create_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Emergency Relief Fund");
    let description = String::from_str(&env, "Helping those in need");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    assert_eq!(pool_id, 1);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.0, 1); // id
    assert_eq!(pool.1, creator); // creator
    assert_eq!(pool.2, goal); // goal
    assert_eq!(pool.3, 0); // collected
    assert_eq!(pool.4, false); // is_closed
}

#[test]
fn test_donate() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Educational Scholarship");
    let description = String::from_str(&env, "Support for students");
    let goal: u128 = 10_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    let donation_amount: u128 = 100_000_000;
    client.donate(&pool_id, &donor, &donation_amount);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, donation_amount); // collected amount
}

#[test]
fn test_apply_for_scholarship_creates_application() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let title = String::from_str(&env, "Scholarship Pool");
    let description = String::from_str(&env, "Support for students");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    // Apply to pool
    env.mock_all_auths();
    client.apply_to_pool(
        &pool_id,
        &student,
        &String::from_str(&env, "application_data"),
    );
}

#[test]
fn test_multiple_donations() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor1 = Address::generate(&env);
    let donor2 = Address::generate(&env);
    let title = String::from_str(&env, "Community Project");
    let description = String::from_str(&env, "Building together");
    let goal: u128 = 5_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    client.donate(&pool_id, &donor1, &100_000_000);
    client.donate(&pool_id, &donor2, &200_000_000);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 300_000_000); // collected amount
}

#[test]
fn test_close_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Closed Pool");
    let description = String::from_str(&env, "Test pool");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    client
        .mock_auths(&[MockAuth {
            address: &creator,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "close_pool",
                args: (&pool_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .close_pool(&pool_id);

    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.4, true); // is_closed
}

#[test]
#[should_panic(expected = "Pool is closed")]
fn test_donate_to_closed_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let title = String::from_str(&env, "Test Pool");
    let description = String::from_str(&env, "Test");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);
    client
        .mock_auths(&[MockAuth {
            address: &creator,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "close_pool",
                args: (&pool_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .close_pool(&pool_id);

    client.donate(&pool_id, &donor, &100_000_000);
}

#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_close_pool_unauthorized() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let unauthorized = Address::generate(&env);
    let title = String::from_str(&env, "Test Pool");
    let description = String::from_str(&env, "Test");
    let goal: u128 = 1_000_000_000;

    let pool_id = client.create_pool(&creator, &title, &description, &goal);

    // Try to close pool with unauthorized user - should panic
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

    let creator1 = Address::generate(&env);
    let creator2 = Address::generate(&env);

    let pool_id_1 = client.create_pool(
        &creator1,
        &String::from_str(&env, "Pool 1"),
        &String::from_str(&env, "First pool"),
        &1_000_000_000,
    );

    let pool_id_2 = client.create_pool(
        &creator2,
        &String::from_str(&env, "Pool 2"),
        &String::from_str(&env, "Second pool"),
        &2_000_000_000,
    );

    assert_eq!(pool_id_1, 1);
    assert_eq!(pool_id_2, 2);
    assert_eq!(client.get_pool_count(), 2);
}

// ============= CLAIM_FUNDS TESTS =============

#[test]
#[should_panic(expected = "Application status not found")]
fn test_claim_funds_no_status() {
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

    // Donate to the pool
    client.donate(&pool_id, &creator, &500_000_000);

    // Try to claim without setting status - should panic
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token_address);
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
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Donate to the pool
    client.donate(&pool_id, &creator, &500_000_000);

    // Set status to "Rejected"
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Rejected"));

    // Try to claim with rejected status - should panic
    client.claim_funds(&student, &pool_id, &100_000_000i128, &token_address);
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
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Donate only 100_000_000 to the pool
    client.donate(&pool_id, &creator, &100_000_000);

    // Set status to "Approved"
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Try to claim more than available - should panic
    client.claim_funds(&student, &pool_id, &500_000_000i128, &token_address);
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
    let token_address = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Donate to the pool
    client.donate(&pool_id, &creator, &500_000_000);

    // Set status to "Approved"
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Try to claim negative amount - should panic
    client.claim_funds(&student, &pool_id, &-100_000_000i128, &token_address);
}

#[test]
fn test_claim_funds_get_claimed_amount() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    // Initially, claimed amount should be 0
    let initial_claimed = client.get_claimed_amount(&pool_id, &student);
    assert_eq!(initial_claimed, 0);

    // Donate to the pool
    client.donate(&pool_id, &creator, &500_000_000);

    // Set status to "Approved"
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));
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
        &1_000_000_000,
    );

    // Initially, status should be empty
    let initial_status = client.get_application_status(&pool_id, &student);
    assert_eq!(initial_status, String::from_str(&env, ""));

    // Set status to "Approved"
    let approved_status = String::from_str(&env, "Approved");
    client.set_application_status(&pool_id, &student, &approved_status);

    // Check that status was set correctly
    let status_after_set = client.get_application_status(&pool_id, &student);
    assert_eq!(status_after_set, approved_status);
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

    // Create a real SAC token and fund the contract so it can transfer
    let claim_amount: i128 = 100_000_000;
    let token_address = create_token(&env, claim_amount, &contract_id);

    // Set admin
    client.set_admin(&admin);

    // Create pool and donate
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &500_000_000);

    // Approve student and allow them to claim
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Student claims funds - should accumulate 1% fee
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);

    // Verify application recorded the claim
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
    let token_address = Address::generate(&env);

    // Set admin
    client.set_admin(&admin);

    // Try to claim as non-admin - should panic
    client.claim_protocol_fees(&non_admin, &token_address);
}

#[test]
#[should_panic(expected = "No unclaimed fees")]
fn test_claim_protocol_fees_no_fees() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    // Set admin
    client.set_admin(&admin);

    // Try to claim when no fees accumulated - should panic
    client.claim_protocol_fees(&admin, &token_address);
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

    // Total needed by contract: claim1 + claim2 = 150_000_000
    // net to students: 99_000_000 + 49_500_000; fees: 1_000_000 + 500_000
    let claim_amount1: i128 = 100_000_000;
    let claim_amount2: i128 = 50_000_000;
    let token_address = create_token(&env, claim_amount1 + claim_amount2, &contract_id);

    // Set admin
    client.set_admin(&admin);

    // Create pool with sufficient funds
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &500_000_000);

    // Approve students
    client.set_application_status(&pool_id, &student1, &String::from_str(&env, "Approved"));
    client.set_application_status(&pool_id, &student2, &String::from_str(&env, "Approved"));

    // Multiple students claim - each generates 1% fee
    client.claim_funds(&student1, &pool_id, &claim_amount1, &token_address);
    client.claim_funds(&student2, &pool_id, &claim_amount2, &token_address);

    // Admin claims accumulated fees
    // Expected: (100_000_000 / 100) + (50_000_000 / 100) = 1_000_000 + 500_000 = 1_500_000
    let collected_fees = client.claim_protocol_fees(&admin, &token_address);

    assert_eq!(collected_fees, 1_500_000);
}

#[test]
fn test_fees_isolated_from_student_allocations() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    let claim_amount: i128 = 100_000_000;
    // Fund the contract with enough tokens to cover the transfer
    let token_address = create_token(&env, claim_amount, &contract_id);

    // Set admin
    client.set_admin(&admin);

    // Create pool
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    let donated_amount: u128 = 500_000_000;
    client.donate(&pool_id, &creator, &donated_amount);

    // Approve student
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Student claims
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);

    // Verify student application amount_claimed is correct (not affected by fees)
    let app = client.get_application(&pool_id, &student);
    assert!(app.is_some());
    let app_unwrapped = app.unwrap();

    // amount_claimed should be exactly the claim_amount, fees are tracked separately
    assert_eq!(app_unwrapped.amount_claimed, claim_amount);
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
    // Mint enough for student transfer AND for fee payout to admin later
    // net_transfer = claim_amount - fee = 99_000_000; fee = 1_000_000
    // total needed: 100_000_000 (student net 99M + fee held 1M)
    let token_address = create_token(&env, claim_amount, &contract_id);

    // Set admin
    client.set_admin(&admin);

    // Create pool and donate
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Test Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000,
    );

    client.donate(&pool_id, &creator, &500_000_000);

    // Approve student
    client.set_application_status(&pool_id, &student, &String::from_str(&env, "Approved"));

    // Student claims - net goes to student, 1% fee stays in contract
    client.claim_funds(&student, &pool_id, &claim_amount, &token_address);

    // Admin claims fees - transfers the 1_000_000 fee out
    let _fees = client.claim_protocol_fees(&admin, &token_address);

    // Try to claim again - should panic (no fees accumulated)
    client.claim_protocol_fees(&admin, &token_address);
}
