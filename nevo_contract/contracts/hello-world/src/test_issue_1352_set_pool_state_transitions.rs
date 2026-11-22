#![cfg(test)]
//! Tests for issue #1352 — `set_pool_state` transition matrix.
//!
//! `set_pool_state` is the mock state setter used to drive pools through their
//! lifecycle: it writes any `PoolState` and publishes `pool_stat`. The lifecycle
//! rules themselves are enforced by the operations that read the state
//! (`donate`, `apply_to_pool`, `claim_funds`, `close_pool`,
//! `request_emergency_withdraw`), so the "invalid transition" cases are asserted
//! through those operations.

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    token::StellarAssetClient,
    Address, BytesN, Env, IntoVal, String, Val,
};

const ALL_STATES: [PoolState; 6] = [
    PoolState::Active,
    PoolState::Paused,
    PoolState::Completed,
    PoolState::Cancelled,
    PoolState::Disbursed,
    PoolState::Closed,
];

struct Setup<'a> {
    contract_id: Address,
    client: ContractClient<'a>,
    admin: Address,
    school: Address,
    sponsor: Address,
}

fn setup(env: &Env) -> Setup<'_> {
    env.mock_all_auths();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    client.set_admin(&admin);
    let school = Address::generate(env);
    client.register_school(&school, &BytesN::from_array(env, &[3u8; 32]));

    Setup {
        contract_id,
        client,
        admin,
        school,
        sponsor: Address::generate(env),
    }
}

fn new_pool(env: &Env, s: &Setup<'_>) -> u32 {
    s.client.create_pool_for_school(
        &s.sponsor,
        &String::from_str(env, "State Pool"),
        &String::from_str(env, "set_pool_state transition matrix"),
        &1_000_000u128,
        &s.school,
        &200_000u64,
    )
}

fn stored_state(env: &Env, contract_id: &Address, pool_id: u32) -> PoolState {
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .get::<_, Pool>(&pool_id)
            .unwrap()
            .state
    })
}

// (1)/(4) Every from -> to transition is written by set_pool_state and is
// visible immediately in the stored pool.
#[test]
fn test_every_transition_is_applied_and_visible_immediately() {
    let env = Env::default();
    let s = setup(&env);

    for from in ALL_STATES.iter() {
        for to in ALL_STATES.iter() {
            let pool_id = new_pool(&env, &s);
            s.client.set_pool_state(&pool_id, from);
            assert_eq!(stored_state(&env, &s.contract_id, pool_id), *from);

            s.client.set_pool_state(&pool_id, to);
            assert_eq!(
                stored_state(&env, &s.contract_id, pool_id),
                *to,
                "{:?} -> {:?} was not applied",
                from,
                to
            );
        }
    }
}

// (4) Each transition publishes exactly one pool_stat event with (old, new).
#[test]
fn test_each_transition_publishes_old_and_new_state() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = new_pool(&env, &s);

    let mut previous = PoolState::Active;
    for next in ALL_STATES.iter() {
        s.client.set_pool_state(&pool_id, next);
        let expected: Vec<(Address, Vec<Val>, Val)> = soroban_sdk::vec![
            &env,
            (
                s.contract_id.clone(),
                soroban_sdk::vec![&env, POOL_STATE_SET.into_val(&env), pool_id.into_val(&env)],
                (previous.clone(), next.clone()).into_val(&env),
            ),
        ];
        assert_eq!(
            env.events().all().filter_by_contract(&s.contract_id),
            expected
        );
        previous = next.clone();
    }
}

// (4) get_pool reports the pool as closed only once close_pool has run, not
// merely because the state was set to Closed.
#[test]
fn test_get_pool_closed_flag_follows_close_pool_not_state_setter() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = new_pool(&env, &s);

    s.client.set_pool_state(&pool_id, &PoolState::Closed);
    assert_eq!(
        stored_state(&env, &s.contract_id, pool_id),
        PoolState::Closed
    );
    assert!(!s.client.get_pool(&pool_id).4);

    let closable = new_pool(&env, &s);
    s.client.set_pool_state(&closable, &PoolState::Disbursed);
    s.client.close_pool(&closable);
    assert!(s.client.get_pool(&closable).4);
    assert_eq!(
        stored_state(&env, &s.contract_id, closable),
        PoolState::Closed
    );
}

// (2) close_pool is the only lifecycle exit: it is rejected from every state
// other than Disbursed or Cancelled.
#[test]
fn test_close_pool_only_allowed_from_disbursed_or_cancelled() {
    let env = Env::default();
    let s = setup(&env);

    for state in ALL_STATES.iter() {
        let pool_id = new_pool(&env, &s);
        s.client.set_pool_state(&pool_id, state);
        let result = s.client.try_close_pool(&pool_id);
        match state {
            PoolState::Disbursed | PoolState::Cancelled => {
                assert!(result.is_ok(), "close from {:?} must succeed", state)
            }
            _ => assert_eq!(
                result,
                Err(Ok(ContractError::PoolNotDisbursedOrRefunded)),
                "close from {:?} must be rejected",
                state
            ),
        }
    }
}

// (2) A closed pool cannot be closed again, even after its state is reset.
#[test]
fn test_closed_pool_cannot_be_closed_again_after_state_reset() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = new_pool(&env, &s);

    s.client.set_pool_state(&pool_id, &PoolState::Cancelled);
    s.client.close_pool(&pool_id);

    s.client.set_pool_state(&pool_id, &PoolState::Cancelled);
    assert_eq!(
        s.client.try_close_pool(&pool_id),
        Err(Ok(ContractError::PoolAlreadyClosed))
    );
}

// (3) The lifecycle exit is authorized: close_pool requires the sponsor.
#[test]
fn test_close_pool_requires_sponsor_authorization() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = new_pool(&env, &s);
    s.client.set_pool_state(&pool_id, &PoolState::Disbursed);

    // Authorized by someone other than the sponsor: rejected.
    let outsider = Address::generate(&env);
    let invoke = MockAuthInvoke {
        contract: &s.contract_id,
        fn_name: "close_pool",
        args: (pool_id,).into_val(&env),
        sub_invokes: &[],
    };
    assert!(s
        .client
        .mock_auths(&[MockAuth {
            address: &outsider,
            invoke: &invoke,
        }])
        .try_close_pool(&pool_id)
        .is_err());
    assert!(!s.client.get_pool(&pool_id).4);

    // Authorized by the sponsor: accepted.
    s.client
        .mock_auths(&[MockAuth {
            address: &s.sponsor,
            invoke: &invoke,
        }])
        .close_pool(&pool_id);
    assert!(s.client.get_pool(&pool_id).4);
}

// (3) set_pool_state itself is the test-only mock setter and records no
// authorization.
#[test]
fn test_set_pool_state_records_no_authorization() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = new_pool(&env, &s);

    env.mock_auths(&[]);
    s.client.set_pool_state(&pool_id, &PoolState::Paused);
    assert!(env.auths().is_empty());
    assert_eq!(
        stored_state(&env, &s.contract_id, pool_id),
        PoolState::Paused
    );
}

// (5) Donations are only accepted in Active; every other state rejects them
// and leaves the amount raised unchanged.
#[test]
fn test_donations_blocked_in_every_non_active_state() {
    let env = Env::default();
    let s = setup(&env);
    let donor = Address::generate(&env);

    for state in ALL_STATES.iter() {
        let pool_id = new_pool(&env, &s);
        s.client.set_pool_state(&pool_id, state);
        let result = s.client.try_donate(&pool_id, &donor, &100u128);
        if *state == PoolState::Active {
            assert!(result.is_ok());
            assert_eq!(s.client.get_total_raised(&pool_id), 100);
        } else {
            assert_eq!(
                result,
                Err(Ok(ContractError::InvalidPoolState)),
                "donation in {:?} must be rejected",
                state
            );
            assert_eq!(s.client.get_total_raised(&pool_id), 0);
        }
    }
}

// (5) Moving back to Active re-enables donations (e.g. Paused -> Active).
#[test]
fn test_resuming_to_active_reenables_donations() {
    let env = Env::default();
    let s = setup(&env);
    let pool_id = new_pool(&env, &s);
    let donor = Address::generate(&env);

    s.client.set_pool_state(&pool_id, &PoolState::Paused);
    assert!(s.client.try_donate(&pool_id, &donor, &100u128).is_err());

    s.client.set_pool_state(&pool_id, &PoolState::Active);
    s.client.donate(&pool_id, &donor, &100u128);
    assert_eq!(s.client.get_total_raised(&pool_id), 100);
}

// (5) Student applications are only accepted in Active.
#[test]
fn test_applications_blocked_in_every_non_active_state() {
    let env = Env::default();
    let s = setup(&env);

    for state in ALL_STATES.iter() {
        let pool_id = new_pool(&env, &s);
        s.client.set_pool_state(&pool_id, state);
        let student = Address::generate(&env);
        let result =
            s.client
                .try_apply_to_pool(&pool_id, &student, &String::from_str(&env, "application"));
        if *state == PoolState::Active {
            assert!(result.is_ok());
        } else {
            assert_eq!(result, Err(Ok(ContractError::InvalidPoolState)));
        }
    }
}

// (5) Claims follow the state: allowed in Active/Paused/Completed, rejected in
// Cancelled/Disbursed (InvalidPoolState) and Closed (PoolIsClosed).
#[test]
fn test_claims_follow_pool_state() {
    let env = Env::default();
    let s = setup(&env);
    let token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    StellarAssetClient::new(&env, &token).mint(&s.contract_id, &1_000_000i128);

    for state in ALL_STATES.iter() {
        let pool_id = new_pool(&env, &s);
        s.client
            .donate(&pool_id, &Address::generate(&env), &1_000u128);
        let student = Address::generate(&env);
        s.client.set_application_status(
            &s.admin,
            &pool_id,
            &student,
            &String::from_str(&env, "Approved"),
        );

        s.client.set_pool_state(&pool_id, state);
        let result = s
            .client
            .try_claim_funds(&student, &pool_id, &100i128, &token);
        match state {
            PoolState::Active | PoolState::Paused | PoolState::Completed => {
                assert!(result.is_ok(), "claim in {:?} must succeed", state)
            }
            PoolState::Cancelled | PoolState::Disbursed => {
                assert_eq!(result, Err(Ok(ContractError::InvalidPoolState)))
            }
            PoolState::Closed => assert_eq!(result, Err(Ok(ContractError::PoolIsClosed))),
        }
    }
}

// (5) Emergency withdrawal requests are only accepted while Active.
#[test]
fn test_emergency_requests_blocked_outside_active() {
    let env = Env::default();
    let s = setup(&env);
    let token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    for state in ALL_STATES.iter() {
        let pool_id = new_pool(&env, &s);
        s.client.set_pool_state(&pool_id, state);
        let result = s
            .client
            .try_request_emergency_withdraw(&s.admin, &pool_id, &token, &1i128);
        if *state == PoolState::Active {
            assert!(result.is_ok());
        } else {
            assert_eq!(result, Err(Ok(ContractError::InvalidPoolState)));
        }
    }
}
