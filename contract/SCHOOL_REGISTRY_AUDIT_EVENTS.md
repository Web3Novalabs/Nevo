# School Registry Audit Events Feature

## Overview

This feature implements auditable verification events for school registry management, providing real-time registry status updates for external indexers when schools receive or lose validation access.

## Key Components

### 1. Core Functions

#### `verify_cause(cause: Address)`
- **Purpose**: Registers a school as a verified cause in the protocol
- **Authorization**: Only admin can verify schools
- **Event**: Emits `SchReg` event for external indexers
- **Storage**: Sets `VerifiedCause(address) = true`

#### `reject_cause(cause: Address)`
- **Purpose**: Revokes a school's verification status
- **Authorization**: Only admin can reject schools
- **Event**: Emits `SchRev` event for external indexers
- **Storage**: Removes `VerifiedCause(address)` entry

#### `is_cause_verified(cause: Address)`
- **Purpose**: Checks if a school is currently verified
- **Returns**: Boolean verification status
- **Public**: Available to all callers for verification checks

### 2. Event Specifications

#### School Registration Event (`SchReg`)
```rust
pub fn school_registered(env: &Env, admin: Address, school_addr: Address) {
    let topics = (symbol_short!("SchReg"), school_addr);
    env.events().publish(topics, admin);
}
```
- **Topics**: `(symbol_short!("SchReg"), school_address)`
- **Data**: `admin_address` (who performed the registration)
- **Trigger**: When `verify_cause()` is called successfully

#### School Revocation Event (`SchRev`)
```rust
pub fn school_revoked(env: &Env, admin: Address, school_addr: Address) {
    let topics = (symbol_short!("SchRev"), school_addr);
    env.events().publish(topics, admin);
}
```
- **Topics**: `(symbol_short!("SchRev"), school_address)`
- **Data**: `admin_address` (who performed the revocation)
- **Trigger**: When `reject_cause()` is called successfully

### 3. External Indexer Integration

#### Event Filtering
External indexers can filter events by:
```rust
// Listen for all school registrations
filter_by_topic_0 = symbol_short!("SchReg")

// Listen for specific school events
filter_by_topic_1 = specific_school_address

// Listen for all registry changes
filter_by_topic_0 = [symbol_short!("SchReg"), symbol_short!("SchRev")]
```

#### Real-Time Registry Status
- Events provide immediate notification of registry changes
- No need to poll contract state for updates
- Complete audit trail of all registry modifications
- Admin accountability through event data

## Usage Flow

### 1. School Onboarding
```rust
// Admin verifies a new school
client.verify_cause(&school_address);

// Event emitted: (SchReg, school_address) with admin data
// External indexers immediately see new school registration
```

### 2. School Revocation
```rust
// Admin revokes school access
client.reject_cause(&school_address);

// Event emitted: (SchRev, school_address) with admin data
// External indexers immediately see school revocation
```

### 3. Status Verification
```rust
// Anyone can check current status
let is_verified = client.is_cause_verified(&school_address);

// External indexers can maintain local registry state
// by processing SchReg/SchRev events chronologically
```

## Security Features

### 1. Admin-Only Operations
- Only contract admin can verify or reject schools
- Soroban signature verification ensures authenticity
- Unauthorized attempts return `Unauthorized` error

### 2. Immutable Audit Trail
- All registry changes are permanently recorded on-chain
- Events cannot be modified or deleted after emission
- Complete history of admin actions available

### 3. Idempotent Operations
- Multiple `verify_cause()` calls on same school emit multiple events
- Multiple `reject_cause()` calls are safe (no error on unverified school)
- Supports administrative workflows with retry logic

## Test Coverage

The implementation includes 12 comprehensive test cases covering:

### 1. **Success Scenarios**
- School registration with event emission
- School revocation with event emission
- Multiple schools management
- Repeated operations on same school

### 2. **Authorization Tests**
- Unauthorized verification attempts
- Unauthorized revocation attempts
- Contract not initialized scenarios

### 3. **Event Verification**
- Correct event topic format (`SchReg`, `SchRev`)
- Proper event data (admin address)
- Event emission timing and ordering

### 4. **Edge Cases**
- Revoking unverified schools
- Multiple operations on same school
- Event format compliance

## Integration Benefits

### 1. **Real-Time Monitoring**
- External systems get immediate registry updates
- No polling required for status changes
- Efficient event-driven architecture

### 2. **Audit Compliance**
- Complete trail of all registry modifications
- Admin accountability through event data
- Immutable record of school onboarding/revocation

### 3. **Scalable Indexing**
- Events enable efficient database synchronization
- Topic-based filtering for targeted monitoring
- Supports multiple concurrent indexers

### 4. **Transparency**
- Public visibility of registry changes
- Community oversight of school verification process
- Open audit trail for regulatory compliance

## Event Format Specification

### SchReg Event Structure
```
Topics: [
  0: Symbol("SchReg")     // Event type identifier
  1: Address              // School address being registered
]
Data: Address             // Admin who performed registration
```

### SchRev Event Structure
```
Topics: [
  0: Symbol("SchRev")     // Event type identifier  
  1: Address              // School address being revoked
]
Data: Address             // Admin who performed revocation
```

## Error Handling

- **`NotInitialized`**: Contract admin not set
- **`Unauthorized`**: Caller is not contract admin
- **Success**: All operations complete successfully, even on already-verified/unverified schools

## Compliance

- **Soroban Standards**: Uses proper event emission patterns
- **Gas Efficiency**: Minimal storage operations and computation
- **Event Standards**: Follows Soroban event topic/data conventions
- **Indexer Compatibility**: Events designed for efficient external processing