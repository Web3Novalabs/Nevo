#![cfg(test)]

//! Issue #1308: Comprehensive authorization bypass prevention tests.
//!
//! Covers:
//! (1) Mock auth cannot bypass admin checks
//! (2) Require_auth properly enforced
//! (3) Cross-user authorization prevented
//! (4) Admin-only functions protected
//! (5) User-specific data isolated

use super::*;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, String, Vec,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

// ─────────────────────────────────────────────────────────────────────────────
// (1) Mock auth cannot bypass admin checks
// ─────────────────────────────────────────────────────────────────────────────

/// Mock auth authorizing an attacker cannot bypass the stored admin check in claim_protocol_fees.
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_mock_auth_cannot_bypass_admin_for_claim_protocol_fees() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token = create_token(&env, 1_000_000i128, &contract_id);

    client.set_admin(&admin);

    // Attacker authorizes itself via mock, but is not the stored admin (ContractError::UnauthorizedAdmin #3)
    client.claim_protocol_fees(&attacker, &token);
}

/// Mock auth authorizing an attacker cannot bypass the stored admin check in set_creation_fee.
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_mock_auth_cannot_bypass_admin_for_set_creation_fee() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    client.set_admin(&admin);
    client.set_creation_fee(&attacker, &500i128);
}

/// Mock auth authorizing an attacker cannot bypass the stored admin check in set_crowdfunding_token.
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_mock_auth_cannot_bypass_admin_for_set_crowdfunding_token() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token = create_token(&env, 1_000_000i128, &contract_id);

    client.set_admin(&admin);
    client.set_crowdfunding_token(&attacker, &token);
}

/// Mock auth authorizing an attacker cannot bypass the stored admin check in request_emergency_withdraw.
#[test]
#[should_panic(expected = "Error(Auth, InvalidAction)")]
fn test_mock_auth_cannot_bypass_admin_for_emergency_withdraw() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token = create_token(&env, 1_000_000i128, &contract_id);

    client.set_admin(&admin);
    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000u128,
        &100_000u64,
    );

    client.request_emergency_withdraw(&attacker, &pool_id, &token, &100i128);
}

// ─────────────────────────────────────────────────────────────────────────────
// (2) Require_auth properly enforced
// ─────────────────────────────────────────────────────────────────────────────

/// Signing as an unrelated address does not satisfy admin.require_auth() in set_admin.
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_admin_auth_cannot_be_satisfied_by_another_signer() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);

    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_admin",
                args: (admin.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_admin(&admin);
}

/// A student's funds cannot be claimed without that student's authorization.
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_claim_funds_requires_student_auth() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token = create_token(&env, 1_000_000i128, &contract_id);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &100_000u64,
    );

    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "claim_funds",
                args: (student.clone(), pool_id, 100i128, token.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .claim_funds(&student, &pool_id, &100i128, &token);
}

/// create_pool_for_school strictly enforces creator.require_auth().
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_create_pool_for_school_requires_creator_auth() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let attacker = Address::generate(&env);
    let school = Address::generate(&env);

    client.mock_all_auths();
    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(&env, &[1u8; 32]));

    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "create_pool_for_school",
                args: (
                    creator.clone(),
                    String::from_str(&env, "Title"),
                    String::from_str(&env, "Desc"),
                    1_000_000u128,
                    school.clone(),
                    100_000u64,
                )
                    .into_val(&env),
                sub_invokes: &[],
            },
        }])
        .create_pool_for_school(
            &creator,
            &String::from_str(&env, "Title"),
            &String::from_str(&env, "Desc"),
            &1_000_000u128,
            &school,
            &100_000u64,
        );
}

/// donate_with_token requires donor.require_auth().
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_donate_with_token_requires_donor_auth() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token = create_token(&env, 1_000_000i128, &donor);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000u128,
        &100_000u64,
    );

    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "donate_with_token",
                args: (pool_id, donor.clone(), token.clone(), 100i128).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .donate_with_token(&pool_id, &donor, &token, &100i128);
}

/// apply_to_pool requires student.require_auth().
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_apply_to_pool_requires_student_auth() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student = Address::generate(&env);
    let attacker = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000u128,
        &100_000u64,
    );

    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "apply_to_pool",
                args: (pool_id, student.clone(), String::from_str(&env, "Data")).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .apply_to_pool(&pool_id, &student, &String::from_str(&env, "Data"));
}

// ─────────────────────────────────────────────────────────────────────────────
// (3) Cross-user authorization prevented
// ─────────────────────────────────────────────────────────────────────────────

/// Only the school linked to a pool may approve that pool's applications.
#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_unlinked_school_cannot_approve_application() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let rogue_school = Address::generate(&env);
    let student = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(&env, &[7u8; 32]));

    let pool_id = client.create_pool_for_school(
        &school,
        &String::from_str(&env, "Linked Pool"),
        &String::from_str(&env, "Test"),
        &1_000_000_000u128,
        &school,
        &100_000u64,
    );
    client.apply_to_pool(&pool_id, &student, &String::from_str(&env, "app"));

    // A school that is not linked to this pool must be rejected with OnlyLinkedSchoolCanApprove (#7)
    client.approve_application(&pool_id, &rogue_school, &student, &true);
}

/// An attacker cannot modify pool metadata or signers via save_pool on another creator's pool.
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_cross_user_cannot_save_pool_metadata() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let attacker = Address::generate(&env);

    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000u128,
        &100_000u64,
    );

    let signers = Vec::from_array(&env, [attacker.clone()]);
    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "save_pool",
                args: (
                    pool_id,
                    String::from_str(&env, "New Desc"),
                    String::from_str(&env, "https://example.com"),
                    String::from_str(&env, "hash"),
                    1u32,
                    signers.clone(),
                )
                    .into_val(&env),
                sub_invokes: &[],
            },
        }])
        .save_pool(
            &pool_id,
            &String::from_str(&env, "New Desc"),
            &String::from_str(&env, "https://example.com"),
            &String::from_str(&env, "hash"),
            &1u32,
            &signers,
        );
}

/// An attacker cannot close a pool created by another sponsor.
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_cross_user_cannot_close_pool() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let attacker = Address::generate(&env);

    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000u128,
        &100_000u64,
    );

    // Set pool to Disbursed state
    client.mock_all_auths();
    client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Attacker attempts to close pool
    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "close_pool",
                args: (pool_id,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .close_pool(&pool_id);
}

/// An attacker cannot set pool token on another sponsor's pool.
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_cross_user_cannot_set_pool_token() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let attacker = Address::generate(&env);
    let token = Address::generate(&env);

    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Pool"),
        &String::from_str(&env, "Desc"),
        &1_000_000u128,
        &100_000u64,
    );

    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_pool_token",
                args: (pool_id, token.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_pool_token(&pool_id, &token);
}

// ─────────────────────────────────────────────────────────────────────────────
// (4) Admin-only functions protected
// ─────────────────────────────────────────────────────────────────────────────

/// Unconfigured admin causes set_creation_fee to panic with AdminNotSet (#9).
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_admin_only_set_creation_fee_unconfigured_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let caller = Address::generate(&env);
    client.set_creation_fee(&caller, &500i128);
}

/// Unconfigured admin causes claim_protocol_fees to panic with AdminNotSet (#9).
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_admin_only_claim_protocol_fees_unconfigured_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let caller = Address::generate(&env);
    let token = Address::generate(&env);
    client.claim_protocol_fees(&caller, &token);
}

/// Unconfigured admin causes set_crowdfunding_token to panic with AdminNotSet (#9).
#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_admin_only_set_crowdfunding_token_unconfigured_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let caller = Address::generate(&env);
    let token = Address::generate(&env);
    client.set_crowdfunding_token(&caller, &token);
}

/// Non-admin cannot register a school.
#[test]
#[should_panic(expected = "InvalidAction")]
fn test_admin_only_register_school_protected() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let school = Address::generate(&env);

    client.mock_all_auths();
    client.set_admin(&admin);

    let meta = BytesN::from_array(&env, &[9u8; 32]);
    client
        .mock_auths(&[MockAuth {
            address: &attacker,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "register_school",
                args: (school.clone(), meta.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .register_school(&school, &meta);
}

// ─────────────────────────────────────────────────────────────────────────────
// (5) User-specific data isolated
// ─────────────────────────────────────────────────────────────────────────────

/// User-specific applications and claim tracking are strictly isolated between students.
#[test]
fn test_user_data_isolation_applications_and_claims() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let school = Address::generate(&env);
    let student_a = Address::generate(&env);
    let student_b = Address::generate(&env);

    client.set_admin(&admin);
    client.register_school(&school, &BytesN::from_array(&env, &[1u8; 32]));

    let pool_id = client.create_pool_for_school(
        &school,
        &String::from_str(&env, "Scholarship Pool"),
        &String::from_str(&env, "Education fund"),
        &1_000_000_000u128,
        &school,
        &100_000u64,
    );

    let token = create_token(&env, 10_000_000i128, &contract_id);

    // Both students apply
    client.apply_to_pool(&pool_id, &student_a, &String::from_str(&env, "App A"));
    client.apply_to_pool(&pool_id, &student_b, &String::from_str(&env, "App B"));

    // Approve only Student A
    client.approve_application(&pool_id, &school, &student_a, &true);

    assert_eq!(
        client.get_application_status(&pool_id, &student_a),
        String::from_str(&env, "Approved")
    );
    assert_eq!(
        client.get_application_status(&pool_id, &student_b),
        String::from_str(&env, "Pending")
    );

    // Provide collected funds to pool
    client.donate(&pool_id, &admin, &500_000u128);

    // Student A claims partial funds
    client.claim_funds(&student_a, &pool_id, &10_000i128, &token);

    // Verify Student A's claimed amount is updated
    assert_eq!(client.get_claimed_amount(&pool_id, &student_a), 10_000i128);
    // Student B's claimed amount remains completely isolated at 0
    assert_eq!(client.get_claimed_amount(&pool_id, &student_b), 0i128);
}

/// Donor contributions are strictly isolated per donor address.
#[test]
fn test_user_data_isolation_donor_contributions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let donor_1 = Address::generate(&env);
    let donor_2 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Community Pool"),
        &String::from_str(&env, "Description"),
        &10_000_000u128,
        &100_000u64,
    );

    client.donate(&pool_id, &donor_1, &1_000_000u128);
    client.donate(&pool_id, &donor_2, &2_500_000u128);

    assert_eq!(client.get_contribution(&pool_id, &donor_1), 1_000_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor_2), 2_500_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 2);

    // Donor 1 donates again
    client.donate(&pool_id, &donor_1, &500_000u128);

    // Donor 1 updated, Donor 2 untouched, donor count unchanged
    assert_eq!(client.get_contribution(&pool_id, &donor_1), 1_500_000u128);
    assert_eq!(client.get_contribution(&pool_id, &donor_2), 2_500_000u128);
    assert_eq!(client.get_donor_count(&pool_id), 2);
}

/// Student milestones are isolated by student address and do not leak between users.
#[test]
fn test_user_data_isolation_milestones() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let creator = Address::generate(&env);
    let student_1 = Address::generate(&env);
    let student_2 = Address::generate(&env);

    let pool_id = client.create_pool(
        &creator,
        &String::from_str(&env, "Milestone Isolation Pool"),
        &String::from_str(&env, "Description"),
        &500_000u128,
        &100_000u64,
    );

    client.apply_to_pool(&pool_id, &student_1, &String::from_str(&env, "App 1"));
    client.apply_to_pool(&pool_id, &student_2, &String::from_str(&env, "App 2"));

    let milestones = Vec::from_array(&env, [Milestone { amount: 500_000u128 }]);
    client.setup_application_milestones(&pool_id, &student_1, &milestones);

    assert_eq!(client.get_milestones(&pool_id, &student_1).len(), 1);
    assert_eq!(client.get_milestones(&pool_id, &student_2).len(), 0);
}
