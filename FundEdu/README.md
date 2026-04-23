# FundEdu — Phase 1 Conceptual Overview

FundEdu is a vertical built on top of the Nevo donation-pool infrastructure. It
channels the general-purpose `PoolConfig` / `contribute` / `close_pool`
primitives into a structured education-funding workflow with three distinct
actors: **Sponsors**, **Students**, and **Validators**.

---

## Architecture at a Glance

```
Sponsor ──creates──► PoolConfig (FundEdu pool)
                          │
                          ▼
Student ──applies──► Application (off-chain metadata + on-chain contribution claim)
                          │
                          ▼
Validator ──reviews──► verify_cause  ──► disburse to Student wallet
```

All on-chain state lives inside the existing `CrowdfundingContract`. FundEdu
adds no new contract; it is a **usage convention** layered over the existing
interface.

---

## Roles

### Sponsor

A Sponsor is any Stellar address that wants to fund student education. In Phase 1
a Sponsor:

1. Calls `initialize` (once, admin only) to set the accepted token and creation
   fee.
2. Calls `create_pool` (or `save_pool`) with a `PoolConfig` that describes the
   scholarship round.
3. Calls `contribute` to deposit XLM or USDC into the pool.
4. Optionally calls `update_pool_state` to pause or cancel the round.
5. After a Validator approves a student cause, calls `close_pool` to finalise
   disbursement.

Key `PoolConfig` fields a Sponsor sets for FundEdu:

| Field | Recommended value |
|---|---|
| `name` | Human-readable scholarship name, e.g. `"STEM 2026 Q1"` |
| `description` | ≤ 500 chars; eligibility criteria |
| `target_amount` | Total scholarship budget in token smallest units |
| `min_contribution` | Minimum top-up per transaction (0 = no minimum) |
| `is_private` | `true` to restrict contributions to whitelisted addresses |
| `duration` | Seconds the pool stays open for contributions |
| `token_address` | Must match the token set via `set_crowdfunding_token` |

---

### Student

A Student is a Stellar address that wants to receive scholarship funds. In Phase 1
a Student:

1. Submits an **application** — a JSON document stored off-chain (IPFS / Arweave)
   whose content hash is recorded in `PoolMetadata.image_hash`.
2. Calls `contribute` with `amount = 0` and `is_private = false` to register
   their address against the pool (acts as an on-chain application receipt).
3. Waits for a Validator to call `verify_cause` with the student's address.
4. Once verified, the Sponsor (or admin) disburses funds via `close_pool`.

#### Application layout (off-chain JSON)

```json
{
  "version": 1,
  "pool_id": 42,
  "student_address": "G...",
  "full_name": "<name>",
  "institution": "<university or school name>",
  "program": "<degree / course>",
  "requested_amount": 5000000,
  "supporting_docs_hash": "<sha256 of uploaded PDF bundle>",
  "submitted_at": 1745395200
}
```

The `supporting_docs_hash` ties the off-chain documents to the on-chain record
without exposing personal data on the ledger.

---

### Validator

A Validator is a trusted address (set by the admin) responsible for due-diligence
on student applications. In Phase 1 a Validator:

1. Reviews the off-chain application JSON and supporting documents.
2. Calls `verify_cause(env, cause: Address)` where `cause` is the student's
   Stellar address.
3. The contract records the address as verified via `StorageKey::VerifiedCause`.
4. Calls `is_cause_verified(env, cause)` to confirm the state was persisted.

Only the contract admin can grant Validator privileges. Validators **cannot**
move funds — they only flip the verification flag that gates disbursement.

---

## Lifecycle State Machine

```
Pool created (Active)
      │
      ├─► Sponsor tops up via contribute()
      │
      ├─► Students register (contribute amount=0)
      │
      ├─► Validator calls verify_cause() per approved student
      │
      └─► Sponsor calls close_pool()  ──► Closed / Disbursed
```

`PoolState` variants used in FundEdu:

| State | Meaning |
|---|---|
| `Active` | Accepting contributions and applications |
| `Paused` | Temporarily halted; no new contributions |
| `Completed` | Target reached; awaiting disbursement |
| `Disbursed` | Funds sent to verified students |
| `Closed` | Pool finalised |

---

## Interaction Examples

All examples use the Soroban SDK test client pattern. Replace
`CrowdfundingContractClient` with your deployed contract address on testnet/mainnet.

### 1 — Sponsor creates a FundEdu pool

```rust
use soroban_sdk::{Address, Env, String};
use crate::base::types::{PoolConfig, PoolMetadata};

let env = Env::default();
env.mock_all_auths();

// Assume contract is already initialized with `token_address`.
let config = PoolConfig {
    name: String::from_str(&env, "STEM 2026 Q1"),
    description: String::from_str(&env, "Scholarship for STEM students in Sub-Saharan Africa"),
    target_amount: 50_000_000, // 50 USDC (7 decimals)
    min_contribution: 0,
    is_private: false,
    duration: 30 * 24 * 60 * 60, // 30 days
    created_at: env.ledger().timestamp(),
    token_address: token_address.clone(),
};

let pool_id = client.create_pool(&sponsor, &config);
// pool_id is the u64 handle used in all subsequent calls
```

### 2 — Student registers an application

```rust
// amount = 0 registers the address without transferring tokens.
client.contribute(
    &pool_id,
    &student_address,
    &token_address,
    &0i128,
    &false,
);
```

### 3 — Validator approves a student

```rust
// Only callable by the contract admin or a delegated validator address.
client.verify_cause(&student_address);

// Confirm the flag is set before disbursement.
let verified = client.is_cause_verified(&student_address);
assert!(verified);
```

### 4 — Query pool state and remaining time

```rust
let pool = client.get_pool(&pool_id).expect("pool not found");
println!("target: {}", pool.target_amount);

let (name, description, url) = client.get_pool_metadata(&pool_id);
println!("pool name: {}", name);

let seconds_left = client.get_pool_remaining_time(&pool_id)
    .expect("pool not found");
println!("seconds until deadline: {}", seconds_left);
```

### 5 — Paginate through contributions

```rust
// Fetch up to 20 contributions starting at offset 0.
let contributions = client
    .get_pool_contributions_paginated(&pool_id, &0u32, &20u32)
    .expect("pool not found");

for c in contributions.iter() {
    println!("contributor: {} amount: {}", c.contributor, c.amount);
}
```

### 6 — Sponsor closes the pool after disbursement

```rust
client.close_pool(&pool_id, &sponsor);

let closed = client.is_closed(&pool_id).expect("pool not found");
assert!(closed);
```

---

## Error Reference

Errors relevant to FundEdu flows (from `CrowdfundingError`):

| Code | Variant | When it occurs |
|---|---|---|
| 6 | `PoolNotFound` | `pool_id` does not exist |
| 11 | `InvalidPoolState` | Operation not allowed in current `PoolState` |
| 12 | `ContractPaused` | Contract-wide pause is active |
| 16 | `InvalidAmount` | Contribution amount violates `min_contribution` |
| 29 | `Unauthorized` | Caller lacks required privileges |
| 43 | `NoContributionToRefund` | Student address has no registered contribution |
| 45 | `PoolAlreadyClosed` | `close_pool` called on an already-closed pool |
| 49 | `UserBlacklisted` | Caller address is on the contract blacklist |

---

## Building and Testing

```bash
# From the repo root
cd contract

# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Run all tests (includes pool and contribution tests)
cargo test

# Run only FundEdu-relevant tests
cargo test test_create_pool
cargo test test_save_pool
cargo test verify_cause
```

All new FundEdu logic that touches the contract must have corresponding tests
in `contract/contract/test/` following the existing file-per-feature convention
(e.g. `fund_edu_test.rs`).

---

## Related Files

| Path | Purpose |
|---|---|
| `contract/contract/src/crowdfunding.rs` | Full contract implementation |
| `contract/contract/src/base/types.rs` | `PoolConfig`, `PoolState`, `StorageKey` |
| `contract/contract/src/base/errors.rs` | `CrowdfundingError` enum |
| `contract/contract/src/interfaces/crowdfunding.rs` | Public trait / ABI |
| `contract/contract/test/create_pool.rs` | Pool creation tests |
| `contract/contract/test/verify_cause.rs` | Cause verification tests |
| `contract/contract/test/crowdfunding_test.rs` | Integration tests |
