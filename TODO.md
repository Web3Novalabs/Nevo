# Task #196: Implement buy_ticket payment logic

## Information Gathered
- buy_ticket in crowdfunding.rs already uses token::Client transfer from buyer to contract
- buy_ticket_test.rs mocks StellarAssetClient::mint and token::Client balances to verify successful transfer
- Comprehensive tests: full success, fee splits (0-100%), rounding, accumulation, validation
- Only issue: compilation error in errors.rs blocking CI

## Plan
1. Fix errors.rs syntax (add missing derives)
2. Verify all buy_ticket tests pass
3. Create branch blackboxai/196-buy-ticket-payment
4. Commit "fix: complete buy_ticket token transfer verification #196"
5. gh pr create

## Dependent Files
- None (logic complete)

## Followup steps
- cargo test passes
- PR created with compare link

Approve plan to proceed?
# Task #195: Implement buy_ticket capacity validation

## Breakdown of approved plan into logical steps:


### 5. Run `cd contract/contract && cargo check && cargo test` to verify
### 6. Create git branch `fix/195-buy-ticket-capacity`, commit changes
### 7. `gh pr create --title "feat: buy_ticket capacity validation #195" --body "Implements #195: capacity check, EventSoldOut error, test. All CI passes." --base main`

**Progress: Starting step 2**

