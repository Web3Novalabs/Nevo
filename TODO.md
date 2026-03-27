# Task: Mark event funds as drained to prevent double withdrawal + test

## Approved Plan Summary
- Add per-pool withdraw_event_pool for EventPool(pool_id) with drained flag
- Auth: pool creator/admin
- Separate StorageKey::EventPoolDrained(u64) bool

## Steps to Complete (In Order)

### 1. Update types.rs
- Add `EventPoolDrained(u64)` to StorageKey enum

### 2. Update events.rs
- Add `pub fn event_pool_drained(env: &Env, pool_id: u64, to: Address, amount: i128)`

### 3. Update interfaces/crowdfunding.rs
- Add trait method: `fn withdraw_event_pool(env: Env, pool_id: u64, to: Address, amount: i128) -> Result<(), CrowdfundingError>;`

### 4. Update crowdfunding.rs
- Add StorageKey import if needed
- Implement `withdraw_event_pool` logic with checks, transfer, set drained=true, emit event

### 5. Add tests in test/crowdfunding_test.rs
- test_withdraw_event_pool_success
- test_double_withdraw_event_pool_fails (first succeeds, second fails Drained)
- test_withdraw_event_pool_wrong_auth_fails
- test_withdraw_event_pool_insufficient_balance_fails
- test_withdraw_event_pool_active_pool_fails

### 6. Test & Verify
```
cd contract/contract
cargo test
make test  # if Makefile has full suite
```

### 7. Create PR
- Branch: blackboxai/event-funds-drained
- Commit changes
- gh pr create

## Progress
- [ ] Step 1
- [ ] Step 2  
- [ ] Step 3
- [ ] Step 4
- [ ] Step 5
- [ ] Step 6
- [ ] Step 7

