#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::StellarAssetClient,
    Address, Env, String, Symbol,
};

fn create_token(env: &Env, amount: i128, recipient: &Address) -> Address {
    let admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(admin.clone());
    let sac = StellarAssetClient::new(env, &token.address());
    sac.mint(recipient, &amount);
    token.address()
}

fn seed_application(
    env: &Env,
    contract_id: &Address,
    pool_id: u32,
    index: u32,
    student: &Address,
    status: &str,
    approved_amount: i128,
    amount_claimed: i128,
) {
    env.as_contract(contract_id, || {
        let app_key = (Symbol::new(env, APPLICATION_PREFIX), pool_id, index);
        env.storage().persistent().set(
            &app_key,
            &(index, student.clone(), String::from_str(env, "application")),
        );

        let status_key = (
            Symbol::new(env, APPLICATION_STATUS_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage()
            .persistent()
            .set(&status_key, &String::from_str(env, status));

        let claim_key = (
            Symbol::new(env, CLAIMED_AMOUNT_PREFIX),
            pool_id,
            student.clone(),
        );
        env.storage().persistent().set(
            &claim_key,
            &Application {
                approved_amount,
                amount_claimed,
            },
        );

        let count_key = (Symbol::new(env, APPLICATION_COUNT_PREFIX), pool_id);
        env.storage().persistent().set(&count_key, &index);
    });
}

#[test]
fn test_withdraw_unallocated_funds_with_no_applications_withdraws_entire_surplus() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "No Applications"),
        &String::from_str(&env, "All funds should be withdrawable"),
        &100_000_000u128,
        &200_000u64,
    );
    client.donate(&pool_id, &sponsor, &100_000_000u128);

    let token_address = create_token(&env, 100_000_000i128, &contract_id);
    client.withdraw_unallocated_funds(&pool_id, &token_address);

    let pool_after = client.get_pool(&pool_id);
    assert_eq!(pool_after.3, 0u128);
}

#[test]
fn test_withdraw_unallocated_funds_excludes_locked_approved_application_funds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Locked Funds"),
        &String::from_str(&env, "Approved funds stay reserved"),
        &100_000_000u128,
        &200_000u64,
    );
    client.donate(&pool_id, &sponsor, &100_000_000u128);

    let student = Address::generate(&env);
    seed_application(
        &env,
        &contract_id,
        pool_id,
        1,
        &student,
        "Approved",
        80_000_000i128,
        20_000_000i128,
    );

    let token_address = create_token(&env, 40_000_000i128, &contract_id);
    client.withdraw_unallocated_funds(&pool_id, &token_address);

    let pool_after = client.get_pool(&pool_id);
    assert_eq!(
        pool_after.3, 60_000_000u128,
        "Only the 40M surplus should be withdrawn"
    );
}

#[test]
#[should_panic(expected = "Insolvency: locked funds exceed collected")]
fn test_withdraw_unallocated_funds_panics_when_locked_funds_exceed_collected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);

    let sponsor = Address::generate(&env);
    let pool_id = client.create_pool(
        &sponsor,
        &String::from_str(&env, "Insolvent Pool"),
        &String::from_str(&env, "Locked funds exceed total collected"),
        &100_000_000u128,
        &200_000u64,
    );
    client.donate(&pool_id, &sponsor, &50_000_000u128);

    let student = Address::generate(&env);
    seed_application(
        &env,
        &contract_id,
        pool_id,
        1,
        &student,
        "Approved",
        60_000_000i128,
        0i128,
    );

    let token_address = create_token(&env, 50_000_000i128, &contract_id);
    client.withdraw_unallocated_funds(&pool_id, &token_address);
}
