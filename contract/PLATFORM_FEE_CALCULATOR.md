# Platform Fee Calculator

## Overview
Internal helper function to calculate platform fees based on basis points for the Nevo crowdfunding platform.

## Function Signature

```rust
fn calculate_platform_fee(amount: i128, fee_bps: u32) -> i128
```

## Parameters

- `amount` (i128): The donation/contribution amount to calculate the fee from
  - Must be non-negative
  - Supports large amounts up to i128::MAX / 10,001

- `fee_bps` (u32): Fee percentage in basis points
  - Range: 0 to 10,000 (0% to 100%)
  - Common values:
    - 100 bps = 1%
    - 250 bps = 2.5%
    - 500 bps = 5%
    - 1,000 bps = 10%

## Returns

- `i128`: The calculated fee amount (rounded down)

## Basis Points Explained

Basis points (bps) are a unit of measure used in finance to describe percentages:
- 1 basis point = 0.01%
- 100 basis points = 1%
- 10,000 basis points = 100%

### Examples:
- 250 bps = 2.5%
- 500 bps = 5%
- 1,000 bps = 10%

## Calculation Formula

```
fee = (amount × fee_bps) / 10,000
```

## Safety Features

### Overflow Protection
- Uses checked arithmetic to prevent overflow
- Validates inputs before calculation
- Panics with descriptive messages on invalid inputs

### Input Validation
- Amount must be non-negative
- Fee basis points must be ≤ 10,000 (100%)
- Checks for potential overflow before multiplication

## Usage Examples

### Basic Usage

```rust
// Calculate 2.5% fee on 10,000 tokens
let fee = CrowdfundingContract::calculate_platform_fee(10_000, 250);
assert_eq!(fee, 250); // 2.5% of 10,000 = 250
```

### Realistic Scenarios

```rust
// Small donation: $50 with 2.5% fee
let fee = CrowdfundingContract::calculate_platform_fee(5_000, 250);
assert_eq!(fee, 125); // $1.25 fee

// Medium donation: $1,000 with 2.5% fee
let fee = CrowdfundingContract::calculate_platform_fee(100_000, 250);
assert_eq!(fee, 2_500); // $25 fee

// Large donation: $10,000 with 2.5% fee
let fee = CrowdfundingContract::calculate_platform_fee(1_000_000, 250);
assert_eq!(fee, 25_000); // $250 fee
```

### Stellar XLM Amounts

Stellar XLM has 7 decimal places (1 XLM = 10,000,000 stroops):

```rust
// 100 XLM donation with 2.5% fee
let amount_stroops = 1_000_000_000; // 100 XLM
let fee = CrowdfundingContract::calculate_platform_fee(amount_stroops, 250);
assert_eq!(fee, 25_000_000); // 2.5 XLM fee
```

## Test Coverage

The implementation includes comprehensive tests covering:

### Standard Cases
- Zero amount and zero fee
- Various percentage calculations (1%, 2.5%, 5%, 10%, 100%)
- Small and large amounts
- Rounding behavior

### Edge Cases
- Maximum safe amounts
- Overflow scenarios
- Negative amount validation
- Invalid basis points validation

### Realistic Scenarios
- Community pools ($500 - $1,000)
- Education funds ($5,000 - $10,000)
- Medical campaigns ($50,000 - $100,000)
- Disaster relief ($1,000,000+)

### Mathematical Properties
- Proportionality: doubling amount doubles fee
- Additivity: fee(a) + fee(b) = fee(a+b)
- Consistency: deterministic results
- Precision: proper rounding behavior

## Error Handling

### Panics

The function panics in the following cases:

1. **Negative Amount**
   ```rust
   // Panics: "amount must be non-negative"
   calculate_platform_fee(-1000, 250);
   ```

2. **Invalid Basis Points**
   ```rust
   // Panics: "fee_bps must be <= 10,000 (100%)"
   calculate_platform_fee(1000, 10_001);
   ```

3. **Overflow**
   ```rust
   // Panics: "fee calculation would overflow"
   calculate_platform_fee(i128::MAX, 10_000);
   ```

## Integration with Contract

### Current Usage

The function is currently an internal helper and can be integrated into:

1. **Donation Processing**
   ```rust
   fn donate(...) {
       // Calculate platform fee
       let fee = Self::calculate_platform_fee(amount, 250); // 2.5%
       let net_amount = amount - fee;
       
       // Transfer net amount to pool
       // Track fee separately
   }
   ```

2. **Pool Contributions**
   ```rust
   fn contribute(...) {
       let fee = Self::calculate_platform_fee(amount, fee_bps);
       // Process contribution with fee deduction
   }
   ```

3. **Fee Reporting**
   ```rust
   fn get_estimated_fee(amount: i128) -> i128 {
       Self::calculate_platform_fee(amount, 250)
   }
   ```

## Performance Considerations

- **Time Complexity**: O(1) - constant time calculation
- **Space Complexity**: O(1) - no additional memory allocation
- **Gas Efficiency**: Minimal operations (multiply, divide, compare)

## Best Practices

1. **Always validate user input** before calling this function
2. **Use consistent basis points** across the platform (e.g., 250 bps)
3. **Document fee structure** clearly to users
4. **Test with realistic amounts** from your use case
5. **Consider rounding** when displaying fees to users

## Future Enhancements

Potential improvements:

1. **Configurable Fee Tiers**
   - Different fees based on amount ranges
   - Volume discounts for large donations

2. **Dynamic Fee Adjustment**
   - Admin-controlled fee percentage
   - Time-based fee variations

3. **Fee Caps**
   - Maximum fee amount regardless of donation size
   - Minimum fee thresholds

4. **Fee Distribution**
   - Split fees between platform and pool creators
   - Referral fee sharing

## Testing

Run the test suite:

```bash
cd contract/contract
cargo test calculate_platform_fee
```

Expected output:
```
running 25 tests
test platform_fee_test::test_calculate_platform_fee_zero_amount ... ok
test platform_fee_test::test_calculate_platform_fee_standard_case ... ok
test platform_fee_test::test_calculate_platform_fee_large_amount ... ok
...
test result: ok. 25 passed; 0 failed; 0 ignored
```

## CI/CD Integration

The function is automatically tested in the CI pipeline:
- Formatting checks with `cargo fmt`
- Unit tests with `cargo test`
- Build verification with `cargo build --release`

## References

- [Basis Points - Investopedia](https://www.investopedia.com/terms/b/basispoint.asp)
- [Soroban Smart Contracts](https://soroban.stellar.org/)
- [Stellar XLM Precision](https://developers.stellar.org/docs/fundamentals-and-concepts/stellar-data-structures/assets)
