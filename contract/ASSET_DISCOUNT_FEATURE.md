# Asset-Based Discount Feature

## Overview
This feature implements a token-based discount system for platform fees. When users donate using specific tokens (e.g., NEVO token), they receive a reduced platform fee.

## Implementation Details

### 1. Storage Keys Added
- `AssetDiscount(Address)`: Maps asset addresses to their discount percentage in basis points
- `PlatformFeePercentage`: Stores the base platform fee percentage in basis points

### 2. New Functions

#### Admin Functions
```rust
fn set_platform_fee_percentage(env: Env, fee_bps: u32) -> Result<(), CrowdfundingError>
```
- Sets the base platform fee percentage (in basis points, where 10000 = 100%)
- Only callable by admin
- Validates fee is not more than 100% (10000 bps)

```rust
fn set_asset_discount(env: Env, asset: Address, discount_bps: u32) -> Result<(), CrowdfundingError>
```
- Sets a discount percentage for a specific asset (in basis points)
- Only callable by admin
- Validates discount is not more than 100% (10000 bps)
- Example: 5000 bps = 50% discount

#### Query Functions
```rust
fn get_platform_fee_percentage(env: Env) -> u32
```
- Returns the current base platform fee percentage
- Returns 0 if not set

```rust
fn get_asset_discount(env: Env, asset: Address) -> u32
```
- Returns the discount percentage for a specific asset
- Returns 0 if no discount is set for the asset

### 3. Modified Functions

#### `contribute` Function
The contribute function now:
1. Retrieves the base platform fee percentage
2. Retrieves the asset-specific discount (if any)
3. Calculates the effective fee: `effective_fee = base_fee * (1 - discount/10000)`
4. Deducts the platform fee from the contribution
5. Credits the net amount to the pool
6. Tracks platform fees separately

**Fee Calculation Formula:**
```
effective_fee_bps = base_fee_bps - (base_fee_bps * discount_bps / 10000)
platform_fee = amount * effective_fee_bps / 10000
net_contribution = amount - platform_fee
```

### 4. Events Added
- `asset_discount_set`: Emitted when an asset discount is configured
- `platform_fee_percentage_set`: Emitted when the platform fee is updated

## Usage Examples

### Setting Up Platform Fee and Discounts

```rust
// Set base platform fee to 10% (1000 basis points)
client.set_platform_fee_percentage(&1000);

// Set 50% discount for NEVO token (5000 basis points)
client.set_asset_discount(&nevo_token_address, &5000);

// Set 25% discount for another token (2500 basis points)
client.set_asset_discount(&other_token_address, &2500);
```

### Fee Calculation Examples

#### Example 1: Regular Token (No Discount)
- Base fee: 10% (1000 bps)
- Discount: 0% (0 bps)
- Effective fee: 10%
- Contribution: 1000 tokens
- Platform fee: 100 tokens
- Net to pool: 900 tokens

#### Example 2: NEVO Token (50% Discount)
- Base fee: 10% (1000 bps)
- Discount: 50% (5000 bps)
- Effective fee: 10% * (1 - 0.5) = 5%
- Contribution: 1000 tokens
- Platform fee: 50 tokens
- Net to pool: 950 tokens

#### Example 3: Premium Token (100% Discount)
- Base fee: 10% (1000 bps)
- Discount: 100% (10000 bps)
- Effective fee: 0%
- Contribution: 1000 tokens
- Platform fee: 0 tokens
- Net to pool: 1000 tokens

## Testing

The implementation includes comprehensive tests in `contract/contract/test/asset_discount_test.rs`:

### Test Cases
1. ✅ `test_set_platform_fee_percentage` - Setting base platform fee
2. ✅ `test_set_platform_fee_percentage_invalid` - Validation for invalid fees
3. ✅ `test_set_asset_discount` - Setting asset-specific discounts
4. ✅ `test_set_asset_discount_invalid` - Validation for invalid discounts
5. ✅ `test_get_asset_discount_default` - Default discount is 0
6. ✅ `test_contribute_with_platform_fee_no_discount` - Fee without discount
7. ✅ `test_contribute_with_platform_fee_and_discount` - Fee with discount
8. ✅ `test_contribute_with_different_assets_different_fees` - Multiple assets with different fees
9. ✅ `test_contribute_with_zero_platform_fee` - No fee scenario
10. ✅ `test_contribute_with_100_percent_discount` - Full discount scenario

### Running Tests

```bash
# Run all asset discount tests
cargo test --manifest-path contract/contract/Cargo.toml asset_discount

# Run a specific test
cargo test --manifest-path contract/contract/Cargo.toml test_contribute_with_platform_fee_and_discount
```

## Security Considerations

1. **Admin-Only Access**: Only the contract admin can set fees and discounts
2. **Validation**: Both fees and discounts are validated to not exceed 100%
3. **Overflow Protection**: Uses `saturating_sub` to prevent underflow in fee calculations
4. **Separate Tracking**: Platform fees are tracked separately from pool contributions

## Integration Notes

### For Frontend Integration
1. Query `get_platform_fee_percentage()` to display base fee
2. Query `get_asset_discount(asset)` to show discount for specific tokens
3. Calculate and display effective fee before user confirms contribution
4. Show breakdown: Total amount, Platform fee, Net to pool

### For Admin Dashboard
1. Provide UI to set/update platform fee percentage
2. Provide UI to configure discounts for specific tokens
3. Display list of tokens with configured discounts
4. Show total platform fees collected

## Future Enhancements

Potential improvements:
1. Time-limited discounts (promotional periods)
2. Tiered discounts based on contribution amount
3. Discount multipliers for verified causes
4. Dynamic discounts based on pool performance
5. Discount caps per user or per pool

## Files Modified

1. `contract/contract/src/base/types.rs` - Added storage keys
2. `contract/contract/src/base/events.rs` - Added event functions
3. `contract/contract/src/interfaces/crowdfunding.rs` - Added function signatures
4. `contract/contract/src/crowdfunding.rs` - Implemented functions and modified contribute
5. `contract/contract/test/asset_discount_test.rs` - Added comprehensive tests
6. `contract/contract/test/mod.rs` - Registered test module

## Basis Points Reference

Basis points (bps) are used for precise percentage representation:
- 1 bps = 0.01%
- 100 bps = 1%
- 1000 bps = 10%
- 5000 bps = 50%
- 10000 bps = 100%

This allows for fine-grained control over fees and discounts.
