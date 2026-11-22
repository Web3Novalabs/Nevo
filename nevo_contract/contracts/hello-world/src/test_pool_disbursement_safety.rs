use super::*;
use soroban_sdk::testutils::{Address as _, Ledger};

struct PoolFixture {
    env: Env,
    contract: Address,
    sponsor: Address,
    school: Address,
    donor: Address,
    student: Address,
    token: Address,
    pool: u32,
}

impl PoolFixture {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger()
            .with_mut(|ledger| ledger.min_persistent_entry_ttl = 20_000);
        let contract = env.register(Contract, ());
        let client = ContractClient::new(&env, &contract);
        let sponsor = Address::generate(&env);
        let school = Address::generate(&env);
        let donor = Address::generate(&env);
        let student = Address::generate(&env);
        client.set_admin(&sponsor);
        client.register_school(&school, &BytesN::from_array(&env, &[0; 32]));
        let pool = client.create_pool_for_school(
            &sponsor,
            &String::from_str(&env, "Student pool"),
            &String::from_str(&env, "Disbursement test"),
            &1_000,
            &school,
            &100_000,
        );
        let token = env
            .register_stellar_asset_contract_v2(sponsor.clone())
            .address();
        client.set_pool_token(&pool, &token);
        Self {
            env,
            contract,
            sponsor,
            school,
            donor,
            student,
            token,
            pool,
        }
    }

    fn client(&self) -> ContractClient<'_> {
        ContractClient::new(&self.env, &self.contract)
    }

    fn asset(&self) -> token::StellarAssetClient<'_> {
        token::StellarAssetClient::new(&self.env, &self.token)
    }

    fn balance(&self, address: &Address) -> i128 {
        token::Client::new(&self.env, &self.token).balance(address)
    }

    fn donate(&self, amount: i128) {
        self.asset().mint(&self.donor, &amount);
        self.client()
            .donate_with_token(&self.pool, &self.donor, &self.token, &amount);
    }

    fn approve(&self) {
        let client = self.client();
        client.apply_to_pool(
            &self.pool,
            &self.student,
            &String::from_str(&self.env, "Application"),
        );
        client.set_application_status(
            &self.school,
            &self.pool,
            &self.student,
            &ApplicationStatus::Approved,
        );
    }

    fn expire_refunds(&self) {
        // Pools without an explicit refund deadline become refundable once the
        // funding deadline plus the grace period has passed.
        let deadline = self.env.ledger().sequence() + 1;
        self.client().set_pool_deadline(&self.pool, &deadline);
        self.env
            .ledger()
            .set_sequence_number(deadline + REFUND_GRACE_PERIOD_LEDGERS);
    }
}

#[test]
fn contribution_totals_are_isolated_by_donor_and_pool() {
    let fixture = PoolFixture::new();
    let client = fixture.client();
    let other_pool = client.create_pool(
        &fixture.sponsor,
        &String::from_str(&fixture.env, "Other pool"),
        &String::from_str(&fixture.env, "Contribution isolation"),
        &1_000,
        &100_000,
    );
    let other_donor = Address::generate(&fixture.env);

    assert_eq!(client.get_contribution(&fixture.pool, &fixture.donor), 0);
    client.donate(&fixture.pool, &fixture.donor, &100);
    assert_eq!(client.get_contribution(&fixture.pool, &fixture.donor), 100);
    fixture.donate(50);
    assert_eq!(client.get_contribution(&fixture.pool, &fixture.donor), 150);

    assert_eq!(client.get_contribution(&other_pool, &fixture.donor), 0);
    client.donate(&other_pool, &fixture.donor, &300);
    client.donate(&fixture.pool, &other_donor, &200);
    assert_eq!(client.get_contribution(&other_pool, &fixture.donor), 300);
    assert_eq!(client.get_contribution(&fixture.pool, &other_donor), 200);
    assert_eq!(client.get_contribution(&fixture.pool, &fixture.donor), 150);
}

#[test]
fn ordinary_pool_lifecycle_cannot_reach_closable_state() {
    let fixture = PoolFixture::new();
    let client = fixture.client();
    let close_error = Err(Ok(ContractError::PoolNotDisbursedOrRefunded.into()));
    assert_eq!(client.try_close_pool(&fixture.pool), close_error);
    fixture.approve();
    client.donate(&fixture.pool, &fixture.donor, &1_000);
    let state = fixture.env.as_contract(&fixture.contract, || {
        fixture
            .env
            .storage()
            .persistent()
            .get::<_, Pool>(&fixture.pool)
            .unwrap()
            .state
    });
    assert_eq!(state, PoolState::Completed);
    assert_eq!(client.try_close_pool(&fixture.pool), close_error);

    fixture.asset().mint(&fixture.contract, &1_000);
    client.claim_funds(&fixture.student, &fixture.pool, &1_000, &fixture.token);
    assert_eq!(
        client.get_claimed_amount(&fixture.pool, &fixture.student),
        1_000
    );
    assert_eq!(client.try_close_pool(&fixture.pool), close_error);
}

#[test]
fn mock_setter_supplies_each_closable_state() {
    for state in [PoolState::Disbursed, PoolState::Cancelled] {
        let fixture = PoolFixture::new();
        let client = fixture.client();
        fixture.env.mock_auths(&[]);
        client.set_pool_state(&fixture.pool, &state);
        assert!(fixture.env.auths().is_empty());
        fixture.env.mock_all_auths();
        client.close_pool(&fixture.pool);
        assert!(client.get_pool(&fixture.pool).4);
    }
}

#[test]
fn refund_after_partial_claim_is_rejected_even_with_sufficient_token_balance() {
    let fixture = PoolFixture::new();
    fixture.donate(200);
    fixture.approve();
    fixture.asset().mint(&fixture.contract, &100);
    let client = fixture.client();
    client.claim_funds(&fixture.student, &fixture.pool, &100, &fixture.token);
    fixture.expire_refunds();

    assert_eq!(fixture.balance(&fixture.student), 99);
    assert_eq!(fixture.balance(&fixture.contract), 201);
    assert_eq!(
        client.try_refund_donation(&fixture.pool, &fixture.donor, &fixture.token),
        Err(Ok(ContractError::InvalidPoolState.into()))
    );
    assert_eq!(fixture.balance(&fixture.donor), 0);
    assert_eq!(fixture.balance(&fixture.contract), 201);
    assert_eq!(client.get_contribution(&fixture.pool, &fixture.donor), 200);
    assert_eq!(client.get_total_raised(&fixture.pool), 200);
}

#[test]
fn refund_is_rejected_after_an_indexed_legacy_claim() {
    let fixture = PoolFixture::new();
    fixture.donate(200);
    fixture.approve();
    let client = fixture.client();
    client.claim_funds(&fixture.student, &fixture.pool, &100, &fixture.token);
    fixture.env.as_contract(&fixture.contract, || {
        let key = (Symbol::new(&fixture.env, POOL_CLAIMED_PREFIX), fixture.pool);
        fixture.env.storage().persistent().remove(&key);
    });
    fixture.expire_refunds();

    assert_eq!(
        client.try_refund_donation(&fixture.pool, &fixture.donor, &fixture.token),
        Err(Ok(ContractError::InvalidPoolState.into()))
    );
}

#[test]
fn refund_is_rejected_for_disbursed_and_closed_pools_without_claims() {
    let fixture = PoolFixture::new();
    fixture.donate(200);
    fixture.expire_refunds();
    let client = fixture.client();
    client.set_pool_state(&fixture.pool, &PoolState::Disbursed);
    assert_eq!(
        client.try_refund_donation(&fixture.pool, &fixture.donor, &fixture.token),
        Err(Ok(ContractError::InvalidPoolState.into()))
    );

    client.close_pool(&fixture.pool);
    assert_eq!(
        client.try_refund_donation(&fixture.pool, &fixture.donor, &fixture.token),
        Err(Ok(ContractError::PoolIsClosed.into()))
    );
    assert_eq!(fixture.balance(&fixture.contract), 200);
}

#[test]
fn failed_claim_transfer_preserves_amount_fees_and_refund_eligibility() {
    let fixture = PoolFixture::new();
    let client = fixture.client();
    client.donate(&fixture.pool, &fixture.donor, &200);
    fixture.approve();

    assert!(client
        .try_claim_funds(&fixture.student, &fixture.pool, &100, &fixture.token)
        .is_err());
    assert_eq!(
        client.get_claimed_amount(&fixture.pool, &fixture.student),
        0
    );
    assert_eq!(fixture.balance(&fixture.student), 0);
    assert_eq!(
        client.try_claim_protocol_fees(&fixture.sponsor, &fixture.token),
        Err(Ok(ContractError::NoUnclaimedFees.into()))
    );

    fixture.asset().mint(&fixture.contract, &200);
    fixture.expire_refunds();
    client.refund_donation(&fixture.pool, &fixture.donor, &fixture.token);
    assert_eq!(fixture.balance(&fixture.donor), 200);
}

#[contract]
struct RepeatedOperations;

#[contractimpl]
impl RepeatedOperations {
    pub fn claim_twice(env: Env, target: Address, student: Address, pool: u32, asset: Address) {
        let client = ContractClient::new(&env, &target);
        client.claim_funds(&student, &pool, &100, &asset);
        assert_eq!(
            client.try_claim_funds(&student, &pool, &100, &asset),
            Err(Ok(ContractError::OverdrawAttempt.into()))
        );
    }

    pub fn execute_twice(env: Env, target: Address, pool: u32) {
        let client = ContractClient::new(&env, &target);
        client.execute_emergency_withdraw(&pool);
        assert_eq!(
            client.try_execute_emergency_withdraw(&pool),
            Err(Ok(ContractError::EmergencyWithdrawalNotRequested.into()))
        );
    }
}

#[test]
fn repeated_claims_cannot_spend_past_the_pool_total() {
    let fixture = PoolFixture::new();
    fixture.approve();
    fixture.donate(150);
    fixture.asset().mint(&fixture.contract, &200);
    let other_student = Address::generate(&fixture.env);
    let client = fixture.client();
    client.apply_to_pool(
        &fixture.pool,
        &other_student,
        &String::from_str(&fixture.env, "Second application"),
    );
    client.set_application_status(
        &fixture.school,
        &fixture.pool,
        &other_student,
        &ApplicationStatus::Approved,
    );

    // The student authorizes a nested call in this test-only batch.
    fixture.env.mock_all_auths_allowing_non_root_auth();
    let batch = fixture.env.register(RepeatedOperations, ());
    RepeatedOperationsClient::new(&fixture.env, &batch).claim_twice(
        &fixture.contract,
        &fixture.student,
        &fixture.pool,
        &fixture.token,
    );
    assert_eq!(
        client.get_claimed_amount(&fixture.pool, &fixture.student),
        100
    );
    assert_eq!(fixture.balance(&fixture.student), 99);
    assert_eq!(
        client.try_claim_funds(&other_student, &fixture.pool, &51, &fixture.token),
        Err(Ok(ContractError::OverdrawAttempt.into()))
    );
    client.claim_funds(&other_student, &fixture.pool, &50, &fixture.token);
    assert_eq!(fixture.balance(&other_student), 50);
    assert_eq!(fixture.balance(&fixture.contract), 201);
}

#[test]
fn emergency_transfer_failure_preserves_request_for_exactly_one_execution() {
    let fixture = PoolFixture::new();
    let client = fixture.client();
    client.request_emergency_withdraw(&fixture.sponsor, &fixture.pool, &fixture.token, &100);
    fixture.env.ledger().set_timestamp(GRACE_PERIOD_SECS);
    assert!(client
        .try_execute_emergency_withdraw(&fixture.pool)
        .is_err());
    assert_eq!(fixture.balance(&fixture.sponsor), 0);

    fixture.asset().mint(&fixture.contract, &200);
    let batch = fixture.env.register(RepeatedOperations, ());
    RepeatedOperationsClient::new(&fixture.env, &batch)
        .execute_twice(&fixture.contract, &fixture.pool);
    assert_eq!(
        client.try_execute_emergency_withdraw(&fixture.pool),
        Err(Ok(ContractError::EmergencyWithdrawalNotRequested.into()))
    );
    assert_eq!(fixture.balance(&fixture.sponsor), 100);
    assert_eq!(fixture.balance(&fixture.contract), 100);
}

#[contracttype]
#[derive(Clone)]
enum TransferExpectation {
    Claim(u32, Address),
    Emergency(u32),
}

#[contract]
struct TransferObserver;

#[contractimpl]
impl TransferObserver {
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        let expectation: TransferExpectation = env.storage().instance().get(&0u32).unwrap();
        // Inspect pending writes through the SDK test frame during the transfer.
        env.as_contract(&from, || match expectation {
            TransferExpectation::Claim(pool, student) => {
                assert_eq!(to, student);
                assert_eq!(amount, 99);
                assert_eq!(
                    Contract::get_claimed_amount(env.clone(), pool, student),
                    100
                );
                assert_eq!(pool_claimed(&env, pool), 100);
                assert_eq!(
                    env.storage()
                        .persistent()
                        .get(&Symbol::new(&env, UNCLAIMED_FEES)),
                    Some(1i128)
                );
            }
            TransferExpectation::Emergency(pool) => {
                assert_eq!(amount, 100);
                let key = (Symbol::new(&env, EMERGENCY_WITHDRAWAL_PREFIX), pool);
                assert!(!env.storage().persistent().has(&key));
            }
        });
    }
}

#[test]
fn claim_and_emergency_effects_are_visible_before_token_transfer() {
    let fixture = PoolFixture::new();
    let client = fixture.client();
    fixture.approve();
    fixture.donate(200);
    let observer = fixture.env.register(TransferObserver, ());
    fixture.env.as_contract(&observer, || {
        fixture.env.storage().instance().set(
            &0u32,
            &TransferExpectation::Claim(fixture.pool, fixture.student.clone()),
        );
    });
    assert_eq!(
        client.try_claim_funds(&fixture.student, &fixture.pool, &100, &observer),
        Err(Ok(ContractError::TokenTransferFailed.into()))
    );
    client.set_pool_token(&fixture.pool, &observer);
    client.claim_funds(&fixture.student, &fixture.pool, &100, &observer);

    fixture.env.as_contract(&observer, || {
        fixture
            .env
            .storage()
            .instance()
            .set(&0u32, &TransferExpectation::Emergency(fixture.pool));
    });
    client.request_emergency_withdraw(&fixture.sponsor, &fixture.pool, &observer, &100);
    fixture.env.ledger().set_timestamp(GRACE_PERIOD_SECS);
    client.execute_emergency_withdraw(&fixture.pool);
}
