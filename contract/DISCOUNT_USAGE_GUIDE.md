# Asset Discount Feature - Quick Usage Guide

## For Contract Administrators

### 1. Initialize Platform Fee
```rust
// Set a 5% platform fee on all contributions
contract.set_platform_fee_percentage(&500); // 500 basis points = 5%
```

### 2. Configure Token Discounts
```rust
// Give NEVO token holders a 50% discount on platform fees
let nevo_token_address = Address::from_string("GCXXX..."); // NEVO token address
contract.set_asset_discount(&nevo_token_address, &5000); // 5000 bps = 50% discount

// Give another token a 25% discount
let premium_token_address = Address::from_string("GDXXX...");
contract.set_asset_discount(&premium_token_address, &2500); // 2500 bps = 25% discount
```

### 3. Query Current Settings
```rust
// Check current platform fee
let fee = contract.get_platform_fee_percentage(); // Returns basis points

// Check discount for a specific token
let discount = contract.get_asset_discount(&nevo_token_address); // Returns basis points
```

## For Contributors

### How Fees Work

When you contribute to a pool, the platform may charge a fee. However, if you use certain tokens (like NEVO), you get a discount!

#### Example Scenario:
- Platform fee: 10%
- NEVO token discount: 50%
- Your contribution: 1000 NEVO tokens

**Calculation:**
1. Base fee: 10% of 1000 = 100 tokens
2. With 50% discount: 100 × 50% = 50 tokens saved
3. Actual fee: 50 tokens
4. Amount to pool: 950 tokens

**You saved 50 tokens by using NEVO!**

### Comparison Table

| Token Type | Contribution | Base Fee (10%) | Discount | Actual Fee | To Pool | Savings |
|------------|--------------|----------------|----------|------------|---------|---------|
| Regular    | 1000         | 100            | 0%       | 100        | 900     | 0       |
| NEVO       | 1000         | 100            | 50%      | 50         | 950     | 50      |
| Premium    | 1000         | 100            | 75%      | 25         | 975     | 75      |

## For Frontend Developers

### Display Fee Information

```javascript
// Fetch platform fee and discount
const platformFeeBps = await contract.get_platform_fee_percentage();
const discountBps = await contract.get_asset_discount(selectedTokenAddress);

// Calculate effective fee
const effectiveFeeBps = platformFeeBps - (platformFeeBps * discountBps / 10000);
const effectiveFeePercent = effectiveFeeBps / 100;

// Calculate amounts for display
const contributionAmount = 1000;
const feeAmount = (contributionAmount * effectiveFeeBps) / 10000;
const netAmount = contributionAmount - feeAmount;

// Display to user
console.log(`Contributing: ${contributionAmount} tokens`);
console.log(`Platform fee (${effectiveFeePercent}%): ${feeAmount} tokens`);
console.log(`To pool: ${netAmount} tokens`);
if (discountBps > 0) {
    const savedAmount = (contributionAmount * platformFeeBps / 10000) - feeAmount;
    console.log(`You saved: ${savedAmount} tokens with ${discountBps/100}% discount!`);
}
```

### UI Recommendations

1. **Token Selection**: Show discount badge next to tokens with discounts
   ```
   [NEVO Token] 🏷️ 50% fee discount
   [XLM Token]
   ```

2. **Contribution Preview**: Show fee breakdown before confirming
   ```
   Contribution Amount:     1000 NEVO
   Platform Fee (5%):       -50 NEVO
   ─────────────────────────────────
   To Pool:                 950 NEVO
   
   💰 You saved 50 NEVO with your discount!
   ```

3. **Discount Indicator**: Highlight savings
   ```css
   .discount-badge {
       background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
       color: white;
       padding: 4px 8px;
       border-radius: 4px;
       font-size: 12px;
   }
   ```

## Testing Checklist

Before deploying to production, verify:

- [ ] Platform fee is set correctly
- [ ] Discounts are configured for intended tokens
- [ ] Fee calculations are accurate
- [ ] Events are emitted properly
- [ ] Only admin can modify fees/discounts
- [ ] Invalid values (>100%) are rejected
- [ ] Zero fee scenario works
- [ ] 100% discount scenario works
- [ ] Multiple tokens with different discounts work correctly

## Common Issues & Solutions

### Issue: Discount not applying
**Solution**: Verify the token address matches exactly. Addresses are case-sensitive.

### Issue: Fee seems wrong
**Solution**: Remember fees are in basis points (10000 = 100%). Double-check your conversion.

### Issue: Cannot set discount
**Solution**: Ensure you're calling from the admin account and the contract is initialized.

## API Reference

### Admin Functions

| Function | Parameters | Description |
|----------|------------|-------------|
| `set_platform_fee_percentage` | `fee_bps: u32` | Set base platform fee (0-10000 bps) |
| `set_asset_discount` | `asset: Address, discount_bps: u32` | Set discount for specific token (0-10000 bps) |

### Query Functions

| Function | Parameters | Returns | Description |
|----------|------------|---------|-------------|
| `get_platform_fee_percentage` | - | `u32` | Get current platform fee in bps |
| `get_asset_discount` | `asset: Address` | `u32` | Get discount for token in bps |

### Events

| Event | Data | Description |
|-------|------|-------------|
| `platform_fee_percentage_set` | `admin: Address, fee_bps: u32` | Platform fee updated |
| `asset_discount_set` | `admin: Address, asset: Address, discount_bps: u32` | Token discount configured |

## Support

For questions or issues:
1. Check the main documentation: `ASSET_DISCOUNT_FEATURE.md`
2. Review test cases: `contract/contract/test/asset_discount_test.rs`
3. Verify your basis points calculations
4. Ensure contract is properly initialized

## Basis Points Quick Reference

```
   1 bps = 0.01%
  10 bps = 0.1%
 100 bps = 1%
 500 bps = 5%
1000 bps = 10%
2500 bps = 25%
5000 bps = 50%
7500 bps = 75%
10000 bps = 100%
```
