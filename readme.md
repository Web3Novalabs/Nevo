## Handsoff notes

<!-- handsoff-issue-1120 -->
- #1120: [P3] Document that the configured creation fee is not yet enforced

<!-- handsoff-issue-1124 -->
- #1124: [P3] Expand the doc comment on donate_with_token with a Panics section
<!-- handsoff-issue-1121 -->
- #1121: [P3] Expand the doc comment on apply_to_pool with a Panics section

<!-- handsoff-issue-1123 -->
- #1123: [P3] Expand the doc comment on setup_application_milestones with a Panics section

<!-- handsoff-issue-1125 -->
- #1125: [P3] Expand the doc comment on close_pool with a Panics section

<!-- handsoff-issue-1303 -->
- #1303: Add tests for event emission validation

# Nevo

[![CI](https://github.com/Web3Novalabs/Nevo/actions/workflows/ci.yml/badge.svg)](https://github.com/Web3Novalabs/Nevo/actions/workflows/ci.yml)

**Nevo** is an open-source donation platform built on Stellar, designed to make fundraising transparent, secure, and efficient through blockchain technology.

## Overview

Nevo enables individuals, communities, and organizations to create and manage donation pools on-chain. By leveraging Stellar’s fast transactions and low fees, Nevo provides a seamless way to collect, track, and distribute funds with full transparency.

## Features

### Open Donation Pools

* Create public fundraising pools for any cause
* Define goals, descriptions, and metadata
* Transparent tracking of contributions on-chain

### Collect Donations

* Anyone can contribute to open pools
* All transactions are recorded and verifiable
* Minimal fees thanks to Stellar infrastructure

### Pool Management

* Pool creators can monitor progress in real time
* Clear visibility into contributors and amounts

### Close Pools

* Creators can close pools once goals are met or campaigns end
* Prevents further contributions after closure

### Withdraw Funds

* Secure withdrawal mechanism for pool owners
* Funds are accessible according to predefined rules

## Why Nevo?

### Transparency

All transactions and pool states are recorded on-chain, ensuring:

* Public auditability
* Trustless donation flows
* Reduced risk of misuse

### Security

* Built using smart contracts on Stellar
* No centralized custody of funds
* Defined access controls for withdrawals and pool management

### Efficiency

* Very low transaction fees
* Fast settlement times
* Scalable for global use

### Open Source

* Fully open and community-driven
* Designed for extensibility and collaboration
* Encourages contributions and independent audits

## How It Works

1. **Create a Pool**
   A user initializes a donation pool with details such as title, description, and goal.

2. **Receive Donations**
   Contributors send funds directly to the pool.

3. **Track Progress**
   All activity is visible on-chain, including contributions and pool status.

4. **Close the Pool**
   The pool creator closes the campaign when appropriate.

5. **Withdraw Funds**
   The creator securely withdraws collected funds.

## License

This project is open-source and available under the MIT License.

## Vision

Nevo aims to redefine how donations are managed by making them:

* Trustless
* Transparent
* Accessible to anyone

## Documentation

- [Pools page details](./docs/pools-page.md) — browse pools, search, filter categories, and review campaign progress.
- [Backend server (nevo_server)](./nevo_server/README.md) — env vars (see `.env.example`), local PostgreSQL setup, and `migration:generate`/`migration:run`/`migration:revert` scripts.

## Run locally with Docker

With Docker and Compose v2 installed, run from the repository root:

```bash
docker compose up
```

Use `docker compose up --build` after changing either app or its Dockerfile. Compose starts PostgreSQL 16, runs the server migrations on startup, and then starts the frontend. The defaults target Stellar testnet.

- Frontend: http://localhost:3002
- API: http://localhost:3001
- Swagger UI: http://localhost:3001/api-docs

Set `NEVO_CONTRACT_ID` before starting to point both apps at your deployed testnet contract. The defaults use ports that rarely conflict; override `FRONTEND_PORT`, `API_PORT`, or `POSTGRES_PORT` if needed. `docker compose down -v` also removes the local database volume.

---

If you have ideas or want to collaborate, feel free to open an issue or submit a pull request.
