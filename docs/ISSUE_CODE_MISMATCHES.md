# Issue/Code Mismatches — contracts/hello-world

While working a batch of contributor test-writing issues against
`nevo_contract/contracts/hello-world`, several GitHub issues turned out to
describe behavior, functions, or error types that don't match the contract
code as it actually exists. This document records those gaps so maintainers
can reconcile issue descriptions with the real code before assigning further
test-writing or implementation work against them — several of these issues
would currently lead a contributor to write tests against fictional behavior
if taken at face value.

Findings below were re-verified directly against source at the time of
writing (branch state noted per issue, since some fixes exist on unmerged
feature branches).

## Issue #1059: contribution metrics tracking

**What the issue describes:** Pool contribution metrics — specifically a
per-donor contribution counter and a `last_donation_at` timestamp field —
should be tracked accurately per pool.

**What actually exists in code:** `donate()`
([lib.rs:389-431](../nevo_contract/contracts/hello-world/src/lib.rs#L389-L431))
and `donate_with_token()`
([lib.rs:1204-1265](../nevo_contract/contracts/hello-world/src/lib.rs#L1204-L1265))
both unconditionally increment a `"d_count"` storage counter on every call,
with no check for whether the donor has contributed before. `get_donor_count()`
([lib.rs:553-564](../nevo_contract/contracts/hello-world/src/lib.rs#L553-L564))
reads this same `"d_count"` key. Confusingly, `donate_with_token()` also
separately tracks a properly-deduplicated unique-donor count via a
`(pool_id, "donor", &donor)` existence check, but that correct counter isn't
what `get_donor_count()` reads. The `Pool` struct
([lib.rs:210-217](../nevo_contract/contracts/hello-world/src/lib.rs#L210-L217))
has exactly six fields (`sponsor, goal, collected, is_closed, state,
application_deadline`) — no `last_donation_at` field exists anywhere in the
struct or storage.

**Gap:** the donor count reported by `get_donor_count()` double-counts (and
keeps counting) repeat donations from the same address, rather than tracking
unique contributors as the name implies — the correct dedup logic exists in
the code but isn't wired to the getter. `last_donation_at` doesn't exist at
all; any test or implementation work assuming it's readable will fail
immediately.

## Issue #1068: refund state validation

**What the issue describes:** `refund_donation` should validate pool state
(e.g. rejecting a refund on an already-disbursed pool) with a dedicated
error, in addition to deadline/grace-period checks.

**What actually exists in code:** Re-checked on both `main` and `feat/#1080`
(the most plausible place a fix would land) — **not fixed on either**. On
`feat/#1080`, `lib.rs` contains two separate `pub fn refund_donation`
definitions back to back (one taking multi-line doc comments and a partial
signature, immediately followed by a second full definition), and the
surviving function body has `.get::<_, Pool>(&pool_id).expect("Pool not
found"); .unwrap_or_else(|| env.panic_with_error(ContractError::PoolNotFound));`
as two chained statements (the second is a dangling method call, not valid
syntax) plus `panic!("PoolNotExpired"); env.panic_with_error(...)` — a
`panic!` immediately followed by unreachable code. This is genuinely
corrupted source, not just a design gap; `cargo build` would not succeed
against this function as written. Eligibility for refund is gated purely by
`deadline == 0 || current_ledger <= deadline || current_ledger < deadline +
REFUND_GRACE_PERIOD_LEDGERS` — there is no reference to `pool.state` or
`is_closed` anywhere in the refund path. `PoolAlreadyDisbursed` does not
exist in `ContractError` on either branch (confirmed via grep — the enum has
14 variants on `main`, 17 on `feat/#1080` after #1080 added
`InvalidPoolName`/`InvalidPoolTarget`/`InvalidPoolDeadline`, none named
`PoolAlreadyDisbursed` or otherwise pool-state-related for refunds).

**Gap:** this issue can't be worked as a normal test-writing task at all —
the target function has duplicated/syntactically-broken code that needs a
real fix (not just tests) before anything meaningful can be asserted against
it. Once fixed, the state-validation behavior and error variant described
still don't exist and would need to be implemented, not just tested.

## Issue #1080: pool creation parameter validation

**What the issue describes:** `save_pool` should reject an empty pool name,
a zero target amount, and a past deadline, each with a specific named error.

**What actually exists in code:** There is no `save_pool` function anywhere
in the contract — pool creation is via `create_pool`
([lib.rs:296-349](../nevo_contract/contracts/hello-world/src/lib.rs#L296-L349)).
Re-checked current state: on `main`, `create_pool` still performs none of
this validation (only a description-length check exists). **On
`feat/#1080` specifically, this has since been implemented**: `create_pool`
now checks `title.len() == 0` → `ContractError::InvalidPoolName` (15),
`goal == 0` → `ContractError::InvalidPoolTarget` (16), and
`application_deadline <= env.ledger().timestamp()` →
`ContractError::InvalidPoolDeadline` (17)
([lib.rs:313-323 on feat/#1080](../nevo_contract/contracts/hello-world/src/lib.rs#L313-L323)).

**Gap:** the function name in the issue (`save_pool`) never matched and
should be corrected to `create_pool` regardless of branch. The validation
behavior itself is a real gap only on `main` — it's already resolved on the
unmerged `feat/#1080` branch, so this issue should be closed or re-scoped
once that branch lands, not worked again from scratch.

## Issue #1082: campaign creation validation edge cases

**What the issue describes:** A maximum title length, a maximum goal value,
and a duplicate-campaign-ID rejection should all be enforced by
`create_campaign`.

**What actually exists in code:** There is no `create_campaign` function;
the same `create_pool` applies. No maximum title length is enforced
anywhere — the only length-related constants in the file are
`MAX_DESCRIPTION_LENGTH = 500`, `MAX_URL_LENGTH = 256`, and
`MAX_IMAGE_HASH_LENGTH = 64`
([lib.rs:42-44](../nevo_contract/contracts/hello-world/src/lib.rs#L42-L44)),
none of which bound `title`. No maximum goal value is enforced — `goal:
u128` is stored as-is with no upper-bound check anywhere in `create_pool`.
Pool IDs are purely auto-incrementing
(`pool_id = pool_count + 1`,
[lib.rs:308-316](../nevo_contract/contracts/hello-world/src/lib.rs#L308-L316))
and are never caller-supplied, on both `main` and `feat/#1080`.

**Gap:** three of the issue's five scenarios describe caps or ID collisions
that cannot occur through any normal call path given the current data
model — "maximum title length succeeds," "maximum goal value succeeds,"
and "duplicate ID fails" all need either a real cap/uniqueness check
implemented first, or the issue re-scoped to describe testing large-but-valid
values instead of enforced maxima.

## Issue #1079: pool multi-signature configuration

**What the issue describes:** Pool creation should support a
`required_signatures` count and a `signers` list, with validation for empty
signer lists, zero required-signature counts, and mismatched
signer/threshold parameters.

**What actually exists in code:** No multi-signature concept exists
anywhere in the contract. A full-file grep for `required_signatures`,
`signers`, `multi_sig`, `multisig`, and `multi-sig` across `lib.rs`, `test.rs`,
`test_issues.rs`, and `test_register_school.rs` returns zero matches. The
`Pool` struct
([lib.rs:210-217](../nevo_contract/contracts/hello-world/src/lib.rs#L210-L217))
models sponsorship as a single `Address`, with no signer list or threshold
field. `create_pool`'s parameters are `creator, title, description, goal,
application_deadline` only. `ContractError` has no signature-related variant.

**Gap:** this isn't a partial mismatch like the others — multi-signature
pool configuration hasn't been built at all, on any branch checked. There is
nothing in the current data model to validate or write tests against; this
issue describes a feature that needs to be designed and implemented before
any test-writing work can be assigned against it.
