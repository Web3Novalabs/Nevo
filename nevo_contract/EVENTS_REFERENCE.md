# Nevo Contract Events Reference Guide

## Quick Reference

This document provides a quick reference for all events emitted by the Nevo smart contract.

---

## Event Topics

| Event Name | Symbol | Description |
|------------|--------|-------------|
| Pool Created | `pool_crtd` | Emitted when a new pool is created |
| Donation Made | `donation` | Emitted when a donation is made to a pool |
| Contribution | `contrib` | Emitted for token-based private donations |
| Pool Closed | `pool_cls` | Emitted when a pool is closed |
| Application Submitted | `app_sub` | Emitted when a student applies to a pool |

---

## Event Details

### 1. POOL_CREATED

**Emitted by**: `create_pool()`, `create_pool_for_school()`

**Topics**:
- `[0]`: Event symbol (`pool_crtd`)
- `[1]`: Pool ID (u32)

**Data**:
```rust
(
    creator: Address,      // Address of pool creator
    goal: u128,           // Funding goal
    title: String,        // Pool title
    description: String   // Pool description
)
```

**Example Usage**:
```rust
// Listen for pool creation
let events = env.events().all();
for event in events {
    if event.topics[0] == symbol_short!("pool_crtd") {
        let pool_id = event.topics[1];
        // Process pool creation...
    }
}
```

---

### 2. DONATION_MADE

**Emitted by**: `donate()`

**Topics**:
- `[0]`: Event symbol (`donation`)
- `[1]`: Pool ID (u32)

**Data**:
```rust
(
    donor: Address,        // Address of donor
    amount: u128,         // Donation amount
    new_collected: u128   // Total collected after donation
)
```

**Privacy**: Public (donor address visible)

**Example Usage**:
```rust
// Track donations to a specific pool
let pool_id = 1;
for event in env.events().all() {
    if event.topics[0] == symbol_short!("donation") 
       && event.topics[1] == pool_id {
        // Process donation...
    }
}
```

---

### 3. CONTRIBUTION

**Emitted by**: `donate_with_token()`

**Topics**:
- `[0]`: Event symbol (`contrib`)
- `[1]`: Pool ID (u32)

**Data**:
```rust
(
    donor: Address,        // Address of donor
    amount: i128,         // Contribution amount
    new_collected: u128,  // Total collected after contribution
    privacy_flag: bool    // true = private, false = public
)
```

**Privacy**: Private (privacy_flag = true)

**Example Usage**:
```rust
// Filter private contributions
for event in env.events().all() {
    if event.topics[0] == symbol_short!("contrib") {
        let (donor, amount, collected, is_private) = event.data;
        if is_private {
            // Handle private contribution
        }
    }
}
```

---

### 4. POOL_CLOSED

**Emitted by**: `close_pool()`

**Topics**:
- `[0]`: Event symbol (`pool_cls`)
- `[1]`: Pool ID (u32)

**Data**:
```rust
(
    sponsor: Address,     // Pool creator/sponsor
    collected: u128       // Final amount collected
)
```

**Example Usage**:
```rust
// Monitor pool closures
for event in env.events().all() {
    if event.topics[0] == symbol_short!("pool_cls") {
        let pool_id = event.topics[1];
        // Process pool closure...
    }
}
```

---

### 5. APPLICATION_SUBMITTED

**Emitted by**: `apply_to_pool()`

**Topics**:
- `[0]`: Event symbol (`app_sub`)
- `[1]`: Pool ID (u32)

**Data**:
```rust
(
    student: Address,      // Student applicant address
    app_count: u32,       // Application number
    privacy_flag: bool    // false = public application
)
```

**Privacy**: Public (privacy_flag = false)

**Example Usage**:
```rust
// Track applications to a pool
let pool_id = 1;
for event in env.events().all() {
    if event.topics[0] == symbol_short!("app_sub") 
       && event.topics[1] == pool_id {
        // Process application...
    }
}
```

---

## Event Filtering Examples

### Filter by Event Type
```rust
fn get_pool_creation_events(env: &Env) -> Vec<Event> {
    env.events().all()
        .into_iter()
        .filter(|e| e.topics[0] == symbol_short!("pool_crtd"))
        .collect()
}
```

### Filter by Pool ID
```rust
fn get_pool_events(env: &Env, pool_id: u32) -> Vec<Event> {
    env.events().all()
        .into_iter()
        .filter(|e| e.topics.len() > 1 && e.topics[1] == pool_id)
        .collect()
}
```

### Filter Private Contributions
```rust
fn get_private_contributions(env: &Env) -> Vec<Event> {
    env.events().all()
        .into_iter()
        .filter(|e| {
            e.topics[0] == symbol_short!("contrib") 
            // Check privacy flag in data
        })
        .collect()
}
```

---

## Privacy Flags

| Operation | Privacy Flag | Visibility |
|-----------|--------------|------------|
| `donate()` | N/A | Public (donor visible) |
| `donate_with_token()` | `true` | Private (donor visible but marked private) |
| `apply_to_pool()` | `false` | Public (student visible) |

**Note**: Privacy flags indicate the intended visibility level. Actual privacy depends on how the frontend/indexer handles these flags.

---

## Event Ordering

Events are emitted in the order operations occur:

1. **Pool Creation** → `POOL_CREATED`
2. **Donations** → `DONATION_MADE` or `CONTRIBUTION`
3. **Applications** → `APPLICATION_SUBMITTED`
4. **Pool Closure** → `POOL_CLOSED`

Multiple events of the same type maintain chronological order.

---

## Best Practices

### For Frontend Developers

1. **Subscribe to Events**: Use event listeners to update UI in real-time
2. **Cache Events**: Store events locally to reduce RPC calls
3. **Filter Efficiently**: Use topic filtering at the RPC level when possible
4. **Handle Privacy**: Respect privacy flags in UI display

### For Indexers

1. **Index by Pool ID**: Create indexes on `topics[1]` for fast pool queries
2. **Track Privacy**: Maintain separate indexes for public/private contributions
3. **Aggregate Data**: Calculate totals from events for analytics
4. **Monitor Order**: Ensure events are processed in chronological order

### For Smart Contract Developers

1. **Emit After State Changes**: Always emit events after state is updated
2. **Include Relevant Data**: Provide all data needed to reconstruct state
3. **Use Consistent Structure**: Maintain consistent topic/data structure
4. **Document Changes**: Update this reference when adding new events

---

## Testing Events

### Basic Event Test Pattern
```rust
#[test]
fn test_event_emission() {
    let env = Env::default();
    let contract_id = env.register(Contract, ());
    let client = ContractClient::new(&env, &contract_id);
    
    let before = env.events().all().len();
    
    // Perform operation
    client.create_pool(...);
    
    let after = env.events().all().len();
    assert_eq!(after, before + 1, "Should emit one event");
    
    let event = env.events().all().last().unwrap();
    assert_eq!(event.topics[0], symbol_short!("pool_crtd"));
}
```

---

## Troubleshooting

### Event Not Emitted
- Check if operation completed successfully
- Verify event emission code is after state changes
- Ensure no panic occurred before emission

### Wrong Event Data
- Verify data tuple matches expected structure
- Check data types match event definition
- Ensure variables are in correct order

### Missing Events in Tests
- Use `env.events().all()` to get all events
- Check event count before and after operation
- Verify test environment is properly initialized

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | May 29, 2026 | Initial event system implementation |

---

## Related Documentation

- [Event Emission Tests](./EVENT_EMISSION_TESTS.md) - Comprehensive test documentation
- [Contract README](./README.md) - Main contract documentation
- [Soroban Events Guide](https://soroban.stellar.org/docs/learn/events) - Official Soroban documentation

---

## Support

For questions or issues related to events:
1. Check test files in `src/test.rs`
2. Review event emission code in `src/lib.rs`
3. Consult Soroban SDK documentation
4. Open an issue in the project repository
