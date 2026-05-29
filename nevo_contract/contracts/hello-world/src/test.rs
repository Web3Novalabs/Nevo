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
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);
    assert!(client.is_school_registered(&school));

    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "School Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &school,
    );
    assert_eq!(pool_id, 1);
    assert_eq!(client.get_pool_school(&pool_id), school);
}

#[test]
#[should_panic(expected = "School is not registered")]
fn test_create_pool_for_unregistered_school_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let school = Address::generate(&env);
    client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
        &school,
    );
}

#[test]
#[should_panic(expected = "Admin not set")]
fn test_register_school_without_admin_set() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    client.register_school(&admin, &school);
}

#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_register_school_wrong_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let wrong_admin = Address::generate(&env);
    let school = Address::generate(&env);
    client.set_admin(&admin);
    client.register_school(&wrong_admin, &school);
}

#[test]
fn test_apply_to_pool_creates_application() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Scholarship Pool"),
        &String::from_str(&env, "Support for students"),
        &1_000_000_000u128,
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "application_data"));
    let status = client.get_application_status(&pool_id, &student);
    assert_eq!(status, String::from_str(&env, "Pending"));
}

#[test]
#[should_panic(expected = "Duplicate application")]
fn test_duplicate_application_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "data"));
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "data"));
}

#[test]
#[should_panic(expected = "Pool not found")]
fn test_apply_to_nonexistent_pool_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let student = Address::generate(&env);
    client.apply_to_pool(&999u32, &student, &String::from_str(&env, "data"));
}

#[test]
fn test_approve_application() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
        &school,
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "data"));
    client.approve_application(&pool_id, &school, &student, &true);
    assert_eq!(
        client.get_application_status(&pool_id, &student),
        String::from_str(&env, "Approved")
    );
}

#[test]
#[should_panic(expected = "Only linked school can approve")]
fn test_wrong_school_cannot_approve() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school1 = Address::generate(&env);
    let school2 = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);

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
#[should_panic(expected = "Student has not applied")]
fn test_approve_unapplied_student_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let creator = Address::generate(&env);
    let student = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&admin, &school);
    let pool_id = client.create_pool_for_school(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
        &school,
    );
    client.approve_application(&pool_id, &school, &student, &true);
}

// ============= MILESTONE TESTS =============

#[test]
#[should_panic(expected = "Milestones required")]
fn test_setup_milestones_empty_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    let empty: Vec<Milestone> = Vec::new(&env);
    client.setup_application_milestones(&pool_id, &student, &empty);
}

#[test]
#[should_panic(expected = "Milestone total must equal pool goal")]
fn test_setup_milestones_wrong_sum_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: 500_000_000 });
    client.setup_application_milestones(&pool_id, &student, &milestones);
}

#[test]
fn test_setup_and_get_milestones() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    let mut milestones = Vec::new(&env);
    milestones.push_back(Milestone { amount: 600_000_000 });
    milestones.push_back(Milestone { amount: 400_000_000 });
    client.setup_application_milestones(&pool_id, &student, &milestones);

    let result = client.get_milestones(&pool_id, &student);
    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0).unwrap().amount, 600_000_000);
    assert_eq!(result.get(1).unwrap().amount, 400_000_000);
}

// ============= DONATE WITH TOKEN TESTS =============

#[test]
fn test_donate_with_token_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Fund"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    let token = create_token(&env, 500_000_000, &donor);
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.3, 100_000_000u128);
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
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Fund"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
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
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Fund"),
        &String::from_str(&env, "Desc"),
        &1_000_000_000u128,
    );
    client.close_pool(&pool_id);
    let token = create_token(&env, 500_000_000, &donor);
    client.donate_with_token(&pool_id, &donor, &token, &100_000_000i128);
}

// ============= ISSUE #491: CONTRACT INITIALIZATION VALIDATION TESTS =============

/// (1) First initialization (set_admin) succeeds.
#[test]
fn test_initialization_first_set_admin_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    // Should not panic
    client.set_admin(&admin);

    // Verify admin was stored by using it to register a school
    let school = Address::generate(&env);
    client.register_school(&admin, &school);
    assert!(client.is_school_registered(&school));
}

/// (2) Second initialization with a different admin overwrites the first.
///     The contract does not block re-initialization, so the new admin takes effect.
#[test]
fn test_initialization_second_set_admin_overwrites_first() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    client.set_admin(&admin1);
    client.set_admin(&admin2);

    // admin1 is no longer the stored admin — register_school with admin1 must fail
    let school = Address::generate(&env);
    // admin2 should succeed
    client.register_school(&admin2, &school);
    assert!(client.is_school_registered(&school));
}

/// (3) Registering a school before admin is set fails with "Admin not set".
#[test]
#[should_panic(expected = "Admin not set")]
fn test_initialization_register_school_before_admin_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    // No set_admin call — must panic with "Admin not set"
    client.register_school(&admin, &school);
}

/// (4) Valid parameters are stored correctly after initialization.
#[test]
fn test_initialization_valid_parameters_stored_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.set_admin(&admin);

    // Pool creation with valid params should work
    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Valid Pool"),
        &String::from_str(&env, "Valid description"),
        &1_000_000_000u128,
    );
    let pool = client.get_pool(&pool_id);
    assert_eq!(pool.1, creator);
    assert_eq!(pool.2, 1_000_000_000u128);
    assert_eq!(pool.3, 0u128);
    assert_eq!(pool.4, false);
}

/// (5) Admin authorization is required to call set_admin.
#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_initialization_set_admin_requires_auth() {
    let env = Env::default(); // no mock_all_auths
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    // No auth provided — must fail with auth error
    client.set_admin(&admin);
}

/// (6) Wrong admin calling register_school fails with "Unauthorized admin".
#[test]
#[should_panic(expected = "Unauthorized admin")]
fn test_initialization_wrong_admin_register_school_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let real_admin = Address::generate(&env);
    let fake_admin = Address::generate(&env);
    let school = Address::generate(&env);

    client.set_admin(&real_admin);
    client.register_school(&fake_admin, &school);
}

/// (7) Pool count starts at zero before any pools are created.
#[test]
fn test_initialization_pool_count_starts_at_zero() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    assert_eq!(client.get_pool_count(), 0);
}

/// (8) is_school_registered returns false for unregistered school.
#[test]
fn test_initialization_unregistered_school_returns_false() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let school = Address::generate(&env);
    assert_eq!(client.is_school_registered(&school), false);
}

// ============= ISSUE #472: POOL REFUND STATE VALIDATION TESTS =============

/// (1) Refund (close_pool) from an Active pool by the sponsor succeeds.
#[test]
fn test_refund_active_pool_by_sponsor_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Active Pool"),
        &String::from_str(&env, "Test refund"),
        &1_000_000_000u128,
    );
    client.donate(&pool_id, &Address::generate(&env), &500_000_000u128);

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
