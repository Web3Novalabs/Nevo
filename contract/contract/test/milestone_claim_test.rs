#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{Milestone, MilestoneStatus, PoolConfig},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(Address::generate(env))
        .address();
    client.initialize(&admin, &token_address, &0);
    (client, admin, token_address)
}

/// Create a pool with `milestones` stored under `PoolMilestones`, fund it, and
/// register `student` as a contributor. Returns `(pool_id, student)`.
fn setup_milestone_pool(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token: &Address,
    milestones: soroban_sdk::Vec<Milestone>,
) -> (u64, Address) {
    let sponsor = Address::generate(env);
    let total: i128 = milestones.iter().map(|m| m.amount).sum();
    StellarAssetClient::new(env, token).mint(&sponsor, &total);

    let config = PoolConfig {
        name: String::from_str(env, "Milestone Pool"),
        description: String::from_str(env, "Pool with milestone payouts"),
        target_amount: total,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: sponsor.clone(),
        application_deadline: env.ledger().timestamp(),
        milestones,
    };

    let pool_id = client.create_pool(&sponsor, &config);

    // Register student as contributor (0-amount contribution = application marker).
    let student = Address::generate(env);
    client.verify_cause(&student);
    client.contribute(&pool_id, &student, token, &0i128, &false);

    (pool_id, student)
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// Claiming before unlock_date returns MilestoneLocked.
#[test]
fn test_claim_before_unlock_date_is_blocked() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (client, _, token) = setup(&env);

    let mut milestones = soroban_sdk::Vec::new(&env);
    milestones.push_back(Milestone {
        unlock_date: 2_000, // in the future
        amount: 10_000,
        status: MilestoneStatus::Pending,
    });

    let (pool_id, student) = setup_milestone_pool(&env, &client, &token, milestones);

    let result = client.try_claim_pool_funds(&pool_id, &student);
    assert_eq!(result, Err(Ok(CrowdfundingError::MilestoneLocked)));
}

/// Claiming at or after unlock_date succeeds and transfers the correct amount.
#[test]
fn test_claim_at_unlock_date_succeeds() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (client, _, token) = setup(&env);

    let amount = 10_000i128;
    let mut milestones = soroban_sdk::Vec::new(&env);
    milestones.push_back(Milestone {
        unlock_date: 1_000,
        amount,
        status: MilestoneStatus::Pending,
    });

    let (pool_id, student) = setup_milestone_pool(&env, &client, &token, milestones);

    client.claim_pool_funds(&pool_id, &student);

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&student), amount);
}

/// Sequential milestones: second claim before second unlock_date is blocked.
#[test]
fn test_sequential_milestones_second_locked() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (client, _, token) = setup(&env);

    let mut milestones = soroban_sdk::Vec::new(&env);
    milestones.push_back(Milestone {
        unlock_date: 1_000,
        amount: 5_000,
        status: MilestoneStatus::Pending,
    });
    milestones.push_back(Milestone {
        unlock_date: 3_000, // not yet reached
        amount: 5_000,
        status: MilestoneStatus::Pending,
    });

    let (pool_id, student) = setup_milestone_pool(&env, &client, &token, milestones);

    // First milestone: succeeds
    client.claim_pool_funds(&pool_id, &student);

    // Second milestone: locked
    let result = client.try_claim_pool_funds(&pool_id, &student);
    assert_eq!(result, Err(Ok(CrowdfundingError::MilestoneLocked)));
}

/// Sequential milestones: both claimable after advancing time.
#[test]
fn test_sequential_milestones_both_claimable() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (client, _, token) = setup(&env);

    let mut milestones = soroban_sdk::Vec::new(&env);
    milestones.push_back(Milestone {
        unlock_date: 1_000,
        amount: 4_000,
        status: MilestoneStatus::Pending,
    });
    milestones.push_back(Milestone {
        unlock_date: 2_000,
        amount: 6_000,
        status: MilestoneStatus::Pending,
    });

    let (pool_id, student) = setup_milestone_pool(&env, &client, &token, milestones);

    // Claim first milestone
    client.claim_pool_funds(&pool_id, &student);

    // Advance past second unlock_date
    env.ledger().with_mut(|l| l.timestamp = 2_000);

    // Claim second milestone
    client.claim_pool_funds(&pool_id, &student);

    let token_client = soroban_sdk::token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&student), 10_000);
}

/// Double-claiming the same milestone returns PoolAlreadyDisbursed (no more Pending milestones).
#[test]
fn test_double_claim_single_milestone_blocked() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    let (client, _, token) = setup(&env);

    let mut milestones = soroban_sdk::Vec::new(&env);
    milestones.push_back(Milestone {
        unlock_date: 1_000,
        amount: 10_000,
        status: MilestoneStatus::Pending,
    });

    let (pool_id, student) = setup_milestone_pool(&env, &client, &token, milestones);

    client.claim_pool_funds(&pool_id, &student);

    // No more Pending milestones → PoolAlreadyDisbursed
    let result = client.try_claim_pool_funds(&pool_id, &student);
    assert_eq!(result, Err(Ok(CrowdfundingError::PoolAlreadyDisbursed)));
}
