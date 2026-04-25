#![cfg(test)]

use crate::{
    base::types::{PoolConfig, PoolState},
    crowdfunding::{CrowdfundingContract, CrowdfundingContractClient},
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events},
    token::StellarAssetClient,
    Address, Env, String, Symbol,
};

fn setup(env: &Env) -> (CrowdfundingContractClient<'_>, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundingContract, ());
    let client = CrowdfundingContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_admin = Address::generate(env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    client.initialize(&admin, &token, &0);
    (client, admin, token)
}

fn make_pool_config(env: &Env, token: &Address) -> PoolConfig {
    PoolConfig {
        name: String::from_str(env, "Lifecycle Pool"),
        description: String::from_str(env, "Testing pool lifecycle events"),
        target_amount: 10_000,
        min_contribution: 0,
        is_private: false,
        duration: 86_400,
        created_at: env.ledger().timestamp(),
        token_address: token.clone(),
        validator: admin.clone(),
    }
}

fn mint_and_create(
    env: &Env,
    client: &CrowdfundingContractClient<'_>,
    token: &Address,
    creator: &Address,
) -> u64 {
    let cfg = make_pool_config(env, token);
    StellarAssetClient::new(env, token).mint(creator, &cfg.target_amount);
    client.create_pool(creator, &cfg)
}

// ---------------------------------------------------------------------------
// PoolCre — emitted by create_pool
// ---------------------------------------------------------------------------

#[test]
fn test_pool_cre_event_emitted_on_create_pool() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let creator = Address::generate(&env);

    mint_and_create(&env, &client, &token, &creator);

    let pool_cre = symbol_short!("PoolCre");
    let found = env.events().all().iter().any(|(_, topics, _)| {
        !topics.is_empty() && {
            use soroban_sdk::FromVal;
            Symbol::from_val(&env, &topics.get(0).unwrap()) == pool_cre
        }
    });

    assert!(found, "PoolCre event must be emitted by create_pool");
}

#[test]
fn test_pool_cre_event_contains_pool_id_and_creator() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let creator = Address::generate(&env);

    let pool_id = mint_and_create(&env, &client, &token, &creator);

    let pool_cre = symbol_short!("PoolCre");
    let found = env.events().all().iter().any(|(_, topics, _)| {
        if topics.len() < 3 {
            return false;
        }
        use soroban_sdk::FromVal;
        let sym = Symbol::from_val(&env, &topics.get(0).unwrap());
        if sym != pool_cre {
            return false;
        }
        let id = u64::from_val(&env, &topics.get(1).unwrap());
        let addr = Address::from_val(&env, &topics.get(2).unwrap());
        id == pool_id && addr == creator
    });

    assert!(
        found,
        "PoolCre topics must include pool_id and creator address"
    );
}

// ---------------------------------------------------------------------------
// PoolUpd — emitted by update_pool_metadata_hash
// ---------------------------------------------------------------------------

#[test]
fn test_pool_upd_event_emitted_on_metadata_update() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let creator = Address::generate(&env);

    let pool_id = mint_and_create(&env, &client, &token, &creator);
    let new_hash = String::from_str(&env, "QmTestHash42");

    client.update_pool_metadata_hash(&pool_id, &creator, &new_hash);

    let pool_upd = symbol_short!("PoolUpd");
    let found = env.events().all().iter().any(|(_, topics, _)| {
        !topics.is_empty() && {
            use soroban_sdk::FromVal;
            Symbol::from_val(&env, &topics.get(0).unwrap()) == pool_upd
        }
    });

    assert!(
        found,
        "PoolUpd event must be emitted by update_pool_metadata_hash"
    );
}

#[test]
fn test_pool_upd_event_payload_contains_new_hash() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let creator = Address::generate(&env);

    let pool_id = mint_and_create(&env, &client, &token, &creator);
    let new_hash = String::from_str(&env, "QmPayloadHash99");

    client.update_pool_metadata_hash(&pool_id, &creator, &new_hash);

    let pool_upd = symbol_short!("PoolUpd");
    let found = env.events().all().iter().any(|(_, topics, data)| {
        if topics.is_empty() {
            return false;
        }
        use soroban_sdk::{FromVal, TryFromVal};
        let sym = Symbol::from_val(&env, &topics.get(0).unwrap());
        if sym != pool_upd {
            return false;
        }
        matches!(String::try_from_val(&env, &data), Ok(h) if h == new_hash)
    });

    assert!(
        found,
        "PoolUpd event payload must contain the new metadata hash"
    );
}

// ---------------------------------------------------------------------------
// PoolPau — emitted by update_pool_state when transitioning to Paused
// ---------------------------------------------------------------------------

#[test]
fn test_pool_pau_event_emitted_when_pool_paused() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let creator = Address::generate(&env);

    let pool_id = mint_and_create(&env, &client, &token, &creator);
    client.update_pool_state(&pool_id, &admin, &PoolState::Paused);

    let pool_pau = symbol_short!("PoolPau");
    let found = env.events().all().iter().any(|(_, topics, _)| {
        !topics.is_empty() && {
            use soroban_sdk::FromVal;
            Symbol::from_val(&env, &topics.get(0).unwrap()) == pool_pau
        }
    });

    assert!(
        found,
        "PoolPau event must be emitted when pool transitions to Paused"
    );
}

#[test]
fn test_pool_pau_not_emitted_for_non_pause_transitions() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let creator = Address::generate(&env);

    let pool_id = mint_and_create(&env, &client, &token, &creator);
    client.update_pool_state(&pool_id, &admin, &PoolState::Completed);

    let pool_pau = symbol_short!("PoolPau");
    let found = env.events().all().iter().any(|(_, topics, _)| {
        !topics.is_empty() && {
            use soroban_sdk::FromVal;
            Symbol::from_val(&env, &topics.get(0).unwrap()) == pool_pau
        }
    });

    assert!(
        !found,
        "PoolPau must NOT be emitted for non-Paused state transitions"
    );
}

#[test]
fn test_pool_pau_topic_contains_pool_id() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let creator = Address::generate(&env);

    let pool_id = mint_and_create(&env, &client, &token, &creator);
    client.update_pool_state(&pool_id, &admin, &PoolState::Paused);

    let pool_pau = symbol_short!("PoolPau");
    let found = env.events().all().iter().any(|(_, topics, _)| {
        if topics.len() < 2 {
            return false;
        }
        use soroban_sdk::FromVal;
        let sym = Symbol::from_val(&env, &topics.get(0).unwrap());
        if sym != pool_pau {
            return false;
        }
        u64::from_val(&env, &topics.get(1).unwrap()) == pool_id
    });

    assert!(found, "PoolPau topic must include the pool_id for indexing");
}

// ---------------------------------------------------------------------------
// State accuracy — events reflect the actual updated state
// ---------------------------------------------------------------------------

#[test]
fn test_pool_state_updated_event_reflects_accurate_state() {
    let env = Env::default();
    let (client, _, token) = setup(&env);
    let creator = Address::generate(&env);

    let pool_id = mint_and_create(&env, &client, &token, &creator);
    client.update_pool_state(&pool_id, &admin, &PoolState::Paused);

    let state_updated = Symbol::new(&env, "pool_state_updated");
    let found = env.events().all().iter().any(|(_, topics, data)| {
        if topics.is_empty() {
            return false;
        }
        use soroban_sdk::{FromVal, TryFromVal};
        let sym = Symbol::from_val(&env, &topics.get(0).unwrap());
        if sym != state_updated {
            return false;
        }
        matches!(PoolState::try_from_val(&env, &data), Ok(PoolState::Paused))
    });

    assert!(
        found,
        "pool_state_updated event payload must accurately reflect the new Paused state"
    );
}
