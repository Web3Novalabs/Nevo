#![cfg(test)]

// ============= ISSUE #1340: `set_admin` REASSIGNMENT FLOW TESTS =============

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, IntoVal};

fn setup(env: &Env) -> ContractClient<'_> {
    let contract_id = env.register(Contract, ());
    ContractClient::new(env, &contract_id)
}

/// The initial admin can be set once and gains admin-gated privileges
/// (verified indirectly via `register_school`, which is gated on the
/// stored admin's authorization).
#[test]
fn test_initial_admin_can_be_set() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);
    let admin = Address::generate(&env);

    client.set_admin(&admin);

    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.register_school(&school, &metadata_hash);

    assert!(client.is_school_registered(&school));
}

/// `set_admin`'s only guard is that the new admin address itself must
/// authorize the call — reassigning to a new admin who has not authorized
/// the call fails.
#[test]
#[should_panic]
fn test_set_admin_requires_new_admin_self_authorization() {
    let env = Env::default();
    let client = setup(&env);
    let new_admin = Address::generate(&env);
    let other = Address::generate(&env);

    // Only `other` is authorized, not `new_admin` itself.
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &other,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_admin",
            args: (new_admin.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);

    client.set_admin(&new_admin);
}

/// Reassigning admin takes effect immediately: the new admin can invoke
/// admin-gated functions right after `set_admin` returns.
#[test]
fn test_new_admin_gains_privileges_immediately() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup(&env);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    client.set_admin(&admin1);
    client.set_admin(&admin2);

    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[2u8; 32]);
    client.register_school(&school, &metadata_hash);

    let auths = env.auths();
    let (addr, _) = auths.last().unwrap().clone();
    assert_eq!(addr, admin2);
}

/// After reassignment, the old admin's authorization no longer satisfies
/// admin-gated functions — `register_school` reads the currently stored
/// admin, which is now `admin2`, so `admin1`'s signature is insufficient.
#[test]
#[should_panic]
fn test_old_admin_loses_privileges_after_reassignment() {
    let env = Env::default();
    let client = setup(&env);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    env.mock_all_auths();
    client.set_admin(&admin1);
    client.set_admin(&admin2);

    let school = Address::generate(&env);
    let metadata_hash = BytesN::from_array(&env, &[3u8; 32]);

    // Only admin1 (the old admin) authorizes this call.
    env.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &admin1,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &client.address,
            fn_name: "register_school",
            args: (school.clone(), metadata_hash.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    client.register_school(&school, &metadata_hash);
}
