#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::{
    base::{
        errors::SecondCrowdfundingError,
        types::{EventDetails, EventMetrics, StorageKey},
    },
    crowdfunding::CrowdfundingContract,
    interfaces::second_crowdfunding::SecondCrowdfundingTrait,
};

fn string_of_len(env: &Env, len: usize) -> String {
    String::from_str(env, &"a".repeat(len))
}

#[test]
fn test_create_event_success_path_returns_ok_for_valid_titles() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());

    let creator = Address::generate(&env);
    let token = Address::generate(&env);
    let base_deadline = env.ledger().timestamp() + 86_400;

    let valid_title_lengths = [1usize, 50usize, 200usize];

    for (index, title_len) in valid_title_lengths.into_iter().enumerate() {
        let id = BytesN::from_array(&env, &[(index + 1) as u8; 32]);
        let title = string_of_len(&env, title_len);

        let result = env.as_contract(&contract_id, || {
            <CrowdfundingContract as SecondCrowdfundingTrait>::create_event(
                env.clone(),
                id,
                title,
                creator.clone(),
                100,
                500,
                base_deadline + index as u64,
                token.clone(),
            )
        });

        assert_eq!(
            result,
            Ok::<(), SecondCrowdfundingError>(()),
            "create_event should succeed for title length {title_len}"
        );
    }
}

#[test]
fn test_create_event_stores_event_details_and_initializes_metrics() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundingContract, ());

    let creator = Address::generate(&env);
    let token = Address::generate(&env);
    let id = BytesN::from_array(&env, &[42u8; 32]);
    let title = String::from_str(&env, "Soroban Hackathon");
    let ticket_price: i128 = 250;
    let max_attendees: u32 = 100;
    let deadline: u64 = env.ledger().timestamp() + 7 * 86_400;

    env.as_contract(&contract_id, || {
        let result = <CrowdfundingContract as SecondCrowdfundingTrait>::create_event(
            env.clone(),
            id.clone(),
            title.clone(),
            creator.clone(),
            ticket_price,
            max_attendees,
            deadline,
            token.clone(),
        );

        assert_eq!(result, Ok(()), "create_event should return Ok");

        // Verify EventDetails stored correctly
        let stored_details: EventDetails = env
            .storage()
            .instance()
            .get(&StorageKey::Event(id.clone()))
            .expect("EventDetails should be stored");

        assert_eq!(stored_details.id, id, "id mismatch");
        assert_eq!(stored_details.title, title, "title mismatch");
        assert_eq!(stored_details.creator, creator, "creator mismatch");
        assert_eq!(
            stored_details.ticket_price, ticket_price,
            "ticket_price mismatch"
        );
        assert_eq!(
            stored_details.max_attendees, max_attendees,
            "max_attendees mismatch"
        );
        assert_eq!(stored_details.deadline, deadline, "deadline mismatch");
        assert_eq!(stored_details.token, token, "token mismatch");

        // Verify EventMetrics initialized with 0 tickets sold
        let stored_metrics: EventMetrics = env
            .storage()
            .instance()
            .get(&StorageKey::EventMetrics(id.clone()))
            .expect("EventMetrics should be stored");

        assert_eq!(
            stored_metrics.tickets_sold, 0,
            "tickets_sold should be initialized to 0"
        );
    });
}
