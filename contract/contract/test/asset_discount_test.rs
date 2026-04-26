#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{
    base::{errors::CrowdfundingError, types::PoolMetadata},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup_test(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    // Initialize with 0 creation fee
    client.initialize(&admin, &token_address, &0);

    (client, admin, token_address)
}

#[test]
fn test_set_platform_fee_percentage() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    // Set platform fee to 5% (500 basis points)
    client.set_platform_fee_percentage(&500);

    let fee = client.get_platform_fee_percentage();
    assert_eq!(fee, 500);
}

#[test]
fn test_set_platform_fee_percentage_invalid() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    // Try to set fee > 100% (10000 basis points)
    let result = client.try_set_platform_fee_percentage(&10001);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidFee)));
}

#[test]
fn test_set_asset_discount() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let nevo_token = Address::generate(&env);

    // Set 50% discount for NEVO token (5000 basis points)
    client.set_asset_discount(&nevo_token, &5000);

    let discount = client.get_asset_discount(&nevo_token);
    assert_eq!(discount, 5000);
}

#[test]
fn test_set_asset_discount_invalid() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let nevo_token = Address::generate(&env);

    // Try to set discount > 100% (10000 basis points)
    let result = client.try_set_asset_discount(&nevo_token, &10001);
    assert_eq!(result, Err(Ok(CrowdfundingError::InvalidFee)));
}

#[test]
fn test_get_asset_discount_default() {
    let env = Env::default();
    let (client, _admin, _token) = setup_test(&env);

    let random_token = Address::generate(&env);

    // Should return 0 for tokens without discount
    let discount = client.get_asset_discount(&random_token);
    assert_eq!(discount, 0);
}

#[test]
fn test_contribute_with_platform_fee_no_discount() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _token) = setup_test(&env);

    // Set platform fee to 5% (500 basis points)
    client.set_platform_fee_percentage(&500);

    // Create a pool
    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None,
        &None,
    );

    // Setup token and contributor
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
    let token_admin_client =
        soroban_sdk::token::StellarAssetClient::new(&env, &token_address);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &10000i128);

    // Contribute 1000 tokens
    let amount = 1000i128;
    client.contribute(&pool_id, &contributor, &token_address, &amount, &false);

    // Expected: 5% fee = 50 tokens, net contribution = 950 tokens
    // Verify token balances
    assert_eq!(token_client.balance(&contributor), 9000i128); // 10000 - 1000
    assert_eq!(
        token_client.balance(&client.address),
        1000i128 // Full amount transferred
    );
}

#[test]
fn test_contribute_with_platform_fee_and_discount() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _token) = setup_test(&env);

    // Set platform fee to 10% (1000 basis points)
    client.set_platform_fee_percentage(&1000);

    // Setup NEVO token with 50% discount
    let token_admin = Address::generate(&env);
    let nevo_token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let nevo_token_address = nevo_token_contract.address();
    let nevo_token_client = soroban_sdk::token::Client::new(&env, &nevo_token_address);
    let nevo_token_admin_client =
        soroban_sdk::token::StellarAssetClient::new(&env, &nevo_token_address);

    // Set 50% discount for NEVO token
    client.set_asset_discount(&nevo_token_address, &5000);

    // Create a pool
    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    nevo_token_admin_client.mint(&contributor, &10000i128);

    // Contribute 1000 NEVO tokens
    let amount = 1000i128;
    client.contribute(&pool_id, &contributor, &nevo_token_address, &amount, &false);

    // Expected calculation:
    // Base fee: 10% = 1000 bps
    // Discount: 50% = 5000 bps
    // Effective fee: 1000 - (1000 * 5000 / 10000) = 1000 - 500 = 500 bps = 5%
    // Fee amount: 1000 * 500 / 10000 = 50 tokens
    // Net contribution: 1000 - 50 = 950 tokens

    // Verify token balances
    assert_eq!(nevo_token_client.balance(&contributor), 9000i128); // 10000 - 1000
    assert_eq!(
        nevo_token_client.balance(&client.address),
        1000i128 // Full amount transferred
    );
}

#[test]
fn test_contribute_with_different_assets_different_fees() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _token) = setup_test(&env);

    // Set platform fee to 10% (1000 basis points)
    client.set_platform_fee_percentage(&1000);

    // Setup NEVO token with 50% discount
    let nevo_admin = Address::generate(&env);
    let nevo_contract = env.register_stellar_asset_contract_v2(nevo_admin.clone());
    let nevo_address = nevo_contract.address();
    let nevo_client = soroban_sdk::token::Client::new(&env, &nevo_address);
    let nevo_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &nevo_address);

    client.set_asset_discount(&nevo_address, &5000); // 50% discount

    // Setup regular token with no discount
    let regular_admin = Address::generate(&env);
    let regular_contract = env.register_stellar_asset_contract_v2(regular_admin.clone());
    let regular_address = regular_contract.address();
    let regular_client = soroban_sdk::token::Client::new(&env, &regular_address);
    let regular_admin_client =
        soroban_sdk::token::StellarAssetClient::new(&env, &regular_address);

    // Create a pool
    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None,
        &None,
    );

    // Contributor 1 uses NEVO token (with discount)
    let contributor1 = Address::generate(&env);
    nevo_admin_client.mint(&contributor1, &10000i128);

    let amount = 1000i128;
    client.contribute(&pool_id, &contributor1, &nevo_address, &amount, &false);

    // NEVO: 10% base fee with 50% discount = 5% effective fee
    // Fee: 1000 * 5% = 50, Net: 950
    assert_eq!(nevo_client.balance(&contributor1), 9000i128);

    // Contributor 2 uses regular token (no discount)
    let contributor2 = Address::generate(&env);
    regular_admin_client.mint(&contributor2, &10000i128);

    client.contribute(&pool_id, &contributor2, &regular_address, &amount, &false);

    // Regular: 10% base fee with 0% discount = 10% effective fee
    // Fee: 1000 * 10% = 100, Net: 900
    assert_eq!(regular_client.balance(&contributor2), 9000i128);

    // Both transferred same amount but different fees were collected
    // Total in contract: 1000 (NEVO) + 1000 (regular) = 2000
    assert_eq!(
        nevo_client.balance(&client.address) + regular_client.balance(&client.address),
        2000i128
    );
}

#[test]
fn test_contribute_with_zero_platform_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _token) = setup_test(&env);

    // Platform fee is 0 by default (not set)
    assert_eq!(client.get_platform_fee_percentage(), 0);

    // Create a pool
    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None,
        &None,
    );

    // Setup token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
    let token_admin_client =
        soroban_sdk::token::StellarAssetClient::new(&env, &token_address);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &10000i128);

    // Contribute 1000 tokens
    let amount = 1000i128;
    client.contribute(&pool_id, &contributor, &token_address, &amount, &false);

    // With 0% fee, full amount should be net contribution
    assert_eq!(token_client.balance(&contributor), 9000i128);
    assert_eq!(token_client.balance(&client.address), 1000i128);
}

#[test]
fn test_contribute_with_100_percent_discount() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _token) = setup_test(&env);

    // Set platform fee to 10% (1000 basis points)
    client.set_platform_fee_percentage(&1000);

    // Setup token with 100% discount
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
    let token_admin_client =
        soroban_sdk::token::StellarAssetClient::new(&env, &token_address);

    // Set 100% discount (10000 basis points)
    client.set_asset_discount(&token_address, &10000);

    // Create a pool
    let creator = Address::generate(&env);
    let name = String::from_str(&env, "Test Pool");
    let metadata = PoolMetadata {
        description: String::from_str(&env, "Test description"),
        external_url: String::from_str(&env, ""),
        image_hash: String::from_str(&env, ""),
    };
    let target_amount = 10_000i128;
    let deadline = env.ledger().timestamp() + 86400;

    let pool_id = client.save_pool(
        &name,
        &metadata,
        &creator,
        &target_amount,
        &deadline,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &10000i128);

    // Contribute 1000 tokens
    let amount = 1000i128;
    client.contribute(&pool_id, &contributor, &token_address, &amount, &false);

    // With 100% discount, effective fee = 0%, so full amount is net contribution
    assert_eq!(token_client.balance(&contributor), 9000i128);
    assert_eq!(token_client.balance(&client.address), 1000i128);
}
