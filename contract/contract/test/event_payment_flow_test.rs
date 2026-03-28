#![cfg(test)]

use soroban_sdk::{testutils::Address as _, token, Address, BytesN, Env, String};

use crate::{
    base::{
        errors::CrowdfundingError,
        types::{EventDetails, PoolConfig, PoolState, StorageKey},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
    interfaces::second_crowdfunding::SecondCrowdfundingTrait,
};

#[test]
fn test_event_payment_flow_create_buy_withdraw() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token, &0);

    // Platform fee bps persisted at init (issue 4) and readable before any set_* call
    let stored_bps: u32 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get(&StorageKey::PlatformFeeBps)
            .expect("PlatformFeeBps must be stored at initialize")
    });
    assert_eq!(stored_bps, 0);
    assert_eq!(client.get_platform_fee_bps(), 0);

    let creator = Address::generate(&env);
    let pool_config = PoolConfig {
        name: String::from_str(&env, "Integration Gala"),
        description: String::from_str(&env, "End-to-end event payment flow"),
        target_amount: 500_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
    };
    let pool_id = client.create_pool(&creator, &pool_config);

    let event_id = BytesN::from_array(&env, &[7u8; 32]);
    let ticket_price: i128 = 20_000;
    env.as_contract(&contract_id, || {
        <CrowdfundingContract as SecondCrowdfundingTrait>::create_event(
            env.clone(),
            event_id.clone(),
            String::from_str(&env, "Gala Night"),
            creator.clone(),
            ticket_price,
            500,
            env.ledger().timestamp() + 86_400,
            token.clone(),
        )
        .expect("create_event");
    });

    let details: EventDetails = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get(&StorageKey::Event(event_id.clone()))
            .expect("event details stored")
    });
    assert_eq!(details.ticket_price, ticket_price);

    client.set_platform_fee_bps(&500);
    assert_eq!(
        env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<StorageKey, u32>(&StorageKey::PlatformFeeBps)
                .unwrap()
        }),
        500
    );

    let buyer = Address::generate(&env);
    let token_mint = token::StellarAssetClient::new(&env, &token);
    token_mint.mint(&buyer, &ticket_price);

    let token_client = token::Client::new(&env, &token);
    let (event_amount, fee_amount) = client.buy_ticket(&pool_id, &buyer, &token, &ticket_price);
    assert_eq!(fee_amount, 1_000);
    assert_eq!(event_amount, 19_000);
    assert_eq!(fee_amount + event_amount, ticket_price);

    assert_eq!(
        token_client.balance(&client.address),
        ticket_price,
        "contract custody matches ticket payment"
    );

    let payout = Address::generate(&env);
    let try_early = client.try_withdraw_event_proceeds(&pool_id, &creator, &payout, &event_amount);
    assert_eq!(
        try_early,
        Err(Ok(CrowdfundingError::InvalidPoolState)),
        "proceeds only after pool is Completed"
    );

    client.update_pool_state(&pool_id, &PoolState::Completed);

    client.withdraw_event_proceeds(&pool_id, &creator, &payout, &event_amount);
    assert_eq!(token_client.balance(&payout), event_amount);

    let fee_receiver = Address::generate(&env);
    client.withdraw_event_fees(&admin, &fee_receiver, &fee_amount);
    assert_eq!(token_client.balance(&fee_receiver), fee_amount);

    assert_eq!(token_client.balance(&client.address), 0);

    let remaining_event_pool: i128 = env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get(&StorageKey::EventPool(pool_id))
            .unwrap_or(0)
    });
    assert_eq!(remaining_event_pool, 0);
}
