# Blacklist Feature Implementation

## Overview
The blacklist feature allows administrators to prevent malicious users from creating campaigns or contributing funds to the crowdfunding platform.

## Implementation Details

### 1. Error Type
Added `UserBlacklisted = 48` to `CrowdfundingError` enum in `contract/contract/src/base/errors.rs`

### 2. Storage
- Blacklisted addresses are stored using `env.storage().persistent()` for long-term persistence
- Storage key: `StorageKey::Blacklist(Address)` added to `contract/contract/src/base/types.rs`

### 3. Interface Methods
Added three new methods to `CrowdfundingTrait` in `contract/contract/src/interfaces/crowdfunding.rs`:

```rust
fn blacklist_address(env: Env, address: Address) -> Result<(), CrowdfundingError>;
fn unblacklist_address(env: Env, address: Address) -> Result<(), CrowdfundingError>;
fn is_blacklisted(env: Env, address: Address) -> bool;
```

### 4. Implementation
In `contract/contract/src/crowdfunding.rs`:

#### `blacklist_address`
- Requires admin authentication
- Stores the address in persistent storage with value `true`
- Emits `address_blacklisted` event

#### `unblacklist_address`
- Requires admin authentication
- Removes the address from persistent storage
- Emits `address_unblacklisted` event

#### `is_blacklisted`
- Public read-only function
- Returns `true` if address is blacklisted, `false` otherwise

### 5. Integration Points
The blacklist check is integrated into:

1. **`create_campaign`**: Checks if the creator is blacklisted before allowing campaign creation
2. **`donate`**: Checks if the donor is blacklisted before accepting donations

Both functions return `CrowdfundingError::UserBlacklisted` if the address is blacklisted.

### 6. Events
Added two new events in `contract/contract/src/base/events.rs`:

```rust
pub fn address_blacklisted(env: &Env, admin: Address, address: Address)
pub fn address_unblacklisted(env: &Env, admin: Address, address: Address)
```

## Test Coverage

Created comprehensive tests in `contract/contract/test/blacklist_test.rs`:

1. **`test_blacklist_address_prevents_donation`**: Verifies blacklisted users cannot donate
2. **`test_blacklist_address_prevents_campaign_creation`**: Verifies blacklisted users cannot create campaigns
3. **`test_unblacklist_address_allows_operations`**: Verifies unblacklisted users can perform operations
4. **`test_only_admin_can_blacklist`**: Verifies only admin can blacklist addresses
5. **`test_blacklist_persists_across_operations`**: Verifies blacklist status persists

## Usage Example

```rust
// Admin blacklists a malicious address
client.blacklist_address(&malicious_user);

// Check if address is blacklisted
let is_blocked = client.is_blacklisted(&malicious_user);

// Malicious user attempts to donate (will fail with UserBlacklisted error)
let result = client.try_donate(&campaign_id, &malicious_user, &token, &100);

// Admin removes user from blacklist
client.unblacklist_address(&malicious_user);

// User can now perform operations
client.donate(&campaign_id, &malicious_user, &token, &100);
```

## Security Considerations

1. **Admin-only access**: Only the contract admin can blacklist/unblacklist addresses
2. **Persistent storage**: Blacklist data is stored in persistent storage to survive contract upgrades
3. **Early validation**: Blacklist checks occur early in the function flow, before any state changes
4. **Event logging**: All blacklist operations are logged via events for transparency and auditing

## Future Enhancements

Potential improvements for future versions:
- Batch blacklist/unblacklist operations
- Temporary blacklists with expiration times
- Blacklist reasons/metadata
- Multi-signature approval for blacklist operations
- Blacklist appeal mechanism
