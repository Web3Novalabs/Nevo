#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

use crate::{
    base::{
        errors::EventError,
        types::{EventStatus, TicketType},
    },
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let token_admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    (client, token_admin, token_address)
}

#[test]
fn test_buy_ticket_success() {
    let env = Env::default();
    let (client, token_admin, token_address) = setup(&env);

    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);

    // Mint tokens to buyer
    token_admin_client.mint(&buyer, &1_000i128);

    let deadline = env.ledger().timestamp() + 86400;
    let ticket_price = 100i128;

    let event_id = client.create_event(
        &creator,
        &String::from_str(&env, "Concert"),
        &deadline,
        &ticket_price,
        &token_address,
    );

    client.buy_ticket(&event_id, &buyer, &TicketType::Standard);

    // Buyer balance reduced by ticket price
    assert_eq!(token_client.balance(&buyer), 900i128);
}

#[test]
fn test_buy_ticket_event_not_found() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let buyer = Address::generate(&env);
    let non_existent_event_id = 999u64;

    let result = client.try_buy_ticket(&non_existent_event_id, &buyer, &TicketType::Standard);
    assert_eq!(result, Err(Ok(EventError::EventNotFound)));
}

#[test]
fn test_buy_ticket_event_expired() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Set current time to 1000
    env.ledger().with_mut(|li| li.timestamp = 1000);

    let deadline = 2000u64;
    let ticket_price = 100i128;

    let event_id = client.create_event(
        &creator,
        &String::from_str(&env, "Expired Event"),
        &deadline,
        &ticket_price,
        &token_address,
    );

    // Advance time past deadline
    env.ledger().with_mut(|li| li.timestamp = 2001);

    let result = client.try_buy_ticket(&event_id, &buyer, &TicketType::Standard);
    assert_eq!(result, Err(Ok(EventError::EventExpired)));
}

#[test]
fn test_buy_ticket_event_not_active() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);

    let deadline = env.ledger().timestamp() + 86400;
    let ticket_price = 100i128;

    let event_id = client.create_event(
        &creator,
        &String::from_str(&env, "Cancelled Event"),
        &deadline,
        &ticket_price,
        &token_address,
    );

    // Manually overwrite event status to Cancelled via storage
    use crate::base::types::{Event, StorageKey};
    let event: Event = client.get_event(&event_id);
    let cancelled_event = Event {
        status: EventStatus::Cancelled,
        ..event
    };
    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .set(&StorageKey::Event(event_id), &cancelled_event);
    });

    let result = client.try_buy_ticket(&event_id, &buyer, &TicketType::Standard);
    assert_eq!(result, Err(Ok(EventError::EventNotActive)));
}

#[test]
fn test_get_event_not_found() {
    let env = Env::default();
    let (client, _, _) = setup(&env);

    let result = client.try_get_event(&999u64);
    assert_eq!(result, Err(Ok(EventError::EventNotFound)));
}

#[test]
fn test_create_event_and_get_event() {
    let env = Env::default();
    let (client, _, token_address) = setup(&env);

    let creator = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 86400;
    let ticket_price = 250i128;

    let event_id = client.create_event(
        &creator,
        &String::from_str(&env, "Gala Night"),
        &deadline,
        &ticket_price,
        &token_address,
    );

    let event = client.get_event(&event_id);
    assert_eq!(event.id, event_id);
    assert_eq!(event.deadline, deadline);
    assert_eq!(event.ticket_price, ticket_price);
    assert_eq!(event.status, EventStatus::Active);
    assert_eq!(event.token_address, token_address);
}
