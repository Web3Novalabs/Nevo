<p align="center">
  <a href="http://nestjs.com/" target="blank"><img src="https://nestjs.com/img/logo-small.svg" width="120" alt="Nest Logo" /></a>
</p>

[circleci-image]: https://img.shields.io/circleci/build/github/nestjs/nest/master?token=abc123def456
[circleci-url]: https://circleci.com/gh/nestjs/nest

  <p align="center">A progressive <a href="http://nodejs.org" target="_blank">Node.js</a> framework for building efficient and scalable server-side applications.</p>
    <p align="center">
<a href="https://www.npmjs.com/~nestjscore" target="_blank"><img src="https://img.shields.io/npm/v/@nestjs/core.svg" alt="NPM Version" /></a>
<a href="https://www.npmjs.com/~nestjscore" target="_blank"><img src="https://img.shields.io/npm/l/@nestjs/core.svg" alt="Package License" /></a>
<a href="https://www.npmjs.com/~nestjscore" target="_blank"><img src="https://img.shields.io/npm/dm/@nestjs/common.svg" alt="NPM Downloads" /></a>
<a href="https://circleci.com/gh/nestjs/nest" target="_blank"><img src="https://img.shields.io/circleci/build/github/nestjs/nest/master" alt="CircleCI" /></a>
<a href="https://discord.gg/G7Qnnhy" target="_blank"><img src="https://img.shields.io/badge/discord-online-brightgreen.svg" alt="Discord"/></a>
<a href="https://opencollective.com/nest#backer" target="_blank"><img src="https://opencollective.com/nest/backers/badge.svg" alt="Backers on Open Collective" /></a>
<a href="https://opencollective.com/nest#sponsor" target="_blank"><img src="https://opencollective.com/nest/sponsors/badge.svg" alt="Sponsors on Open Collective" /></a>
  <a href="https://paypal.me/kamilmysliwiec" target="_blank"><img src="https://img.shields.io/badge/Donate-PayPal-ff3f59.svg" alt="Donate us"/></a>
    <a href="https://opencollective.com/nest#sponsor"  target="_blank"><img src="https://img.shields.io/badge/Support%20us-Open%20Collective-41B883.svg" alt="Support us"></a>
  <a href="https://twitter.com/nestframework" target="_blank"><img src="https://img.shields.io/twitter/follow/nestframework.svg?style=social&label=Follow" alt="Follow us on Twitter"></a>
</p>
  <!--[![Backers on Open Collective](https://opencollective.com/nest/backers/badge.svg)](https://opencollective.com/nest#backer)
  [![Sponsors on Open Collective](https://opencollective.com/nest/sponsors/badge.svg)](https://opencollective.com/nest#sponsor)-->

## Description

[Nest](https://github.com/nestjs/nest) framework TypeScript starter repository.

## Project setup

```bash
$ npm install
```

## Compile and run the project

```bash
# development
$ npm run start

# watch mode
$ npm run start:dev

# production mode
$ npm run start:prod
```

## Database Migrations

The schema is managed with TypeORM migrations. `synchronize` is off in every
environment, so **the schema is only ever changed by running a migration** —
starting the server will not create or alter tables for you.

Migrations live in `src/migrations/` and run against the connection defined in
`src/data-source.ts`, which reads the same `DB_HOST` / `DB_PORT` / `DB_USER` /
`DB_PASSWORD` / `DB_NAME` variables as the app. Copy `.env.example` to `.env`
and fill it in before running any of these.

| Script | What it does |
| --- | --- |
| `npm run migration:generate -- src/migrations/<Name>` | Diffs your entities against the current database and writes a new migration file. Needs a reachable database. |
| `npm run migration:run` | Applies every migration that has not been applied yet, oldest first. |
| `npm run migration:revert` | Rolls back the single most recently applied migration. |

Note the `--` before the path on `migration:generate`: without it npm swallows
the argument instead of forwarding it to the TypeORM CLI.

### First-time setup

```bash
# 1. install dependencies
$ npm install

# 2. configure the database connection
$ cp .env.example .env   # then edit the DB_* variables

# 3. create an empty database matching that connection, e.g.
$ createdb nevo

# 4. apply all existing migrations
$ npm run migration:run

# 5. start the server
$ npm run start:dev
```

Step 4 is the one new contributors miss. If the app starts but every query
fails with `relation "pools" does not exist`, the migrations have not been run.

### Changing the schema

1. Edit the relevant entity (e.g. `src/pools/pool.entity.ts`).
2. Make sure your local database is already up to date, so the diff only
   contains your change:
   ```bash
   $ npm run migration:run
   ```
3. Generate the migration, passing a descriptive name:
   ```bash
   $ npm run migration:generate -- src/migrations/AddCategoryToPools
   ```
   TypeORM prefixes the file with a timestamp, giving
   `src/migrations/1782396384461-AddCategoryToPools.ts`.
4. **Read the generated SQL before committing it.** TypeORM infers intent from
   a schema diff and will happily emit a `DROP COLUMN` where you meant a
   rename, which is silent data loss in production.
5. Apply it locally and confirm the app still works:
   ```bash
   $ npm run migration:run
   ```
6. Commit the migration file alongside the entity change — never one without
   the other.

To undo a migration you have applied locally, `npm run migration:revert` steps
back exactly one migration; run it repeatedly to go further back. Once a
migration is merged and deployed, do not edit it — add a new migration that
corrects it, since anyone who already ran the old one will never re-run it.

### Monetary amounts

`pools.goal`, `pools.raised` and `donations.amount` are `NUMERIC(39,0)` so the full on-chain `i128` range (up to `170141183460469231731687303715884105727`) fits exactly. The `pg` driver returns `NUMERIC` as a string, and every monetary field stays a `string` in TypeScript, DTOs and JSON responses. Never pass these values through `Number`/`parseFloat`; use `BigInt` for arithmetic in code, or SQL `numeric` arithmetic in queries.

Migration `MonetaryAmountsToNumeric1800000000004` is reversible. `down` restores `bigint`/`bigint`/`varchar(255)` and refuses to run (leaving data untouched) if any pool `goal`/`raised` exceeds the `bigint` maximum.
## Pool reconciliation

The indexer maintains `pools.raised`, but missed events, duplicates or refunds can make it drift from the chain. `ReconciliationService` periodically compares `pools.raised` with the contract's `get_total_raised` for every **Active** pool and overwrites the database value when they differ (the chain is authoritative).

| Variable | Default | Meaning |
|---|---|---|
| `RECONCILIATION_INTERVAL_MS` | `300000` | Time between scheduled runs. Invalid values fall back to the default. |
| `RECONCILIATION_BATCH_SIZE` | `20` | Pools read from the chain concurrently per batch (bounds RPC load). |
| `RECONCILIATION_ENABLED` | `true` | `false` disables the schedule; the manual trigger still works. |
| `ADMIN_PUBLIC_KEYS` | _(empty)_ | Comma-separated Stellar public keys allowed to call `/admin/*`. Empty means nobody. |

Behaviour:
- Amounts are compared and written as exact integers (`BigInt`/strings), never floats.
- An RPC failure leaves that pool untouched (it is never treated as a zero total); other pools in the batch continue. A failed DB update affects only that pool.
- The update is conditional on the `raised` value that was read, so a concurrent indexer write is not clobbered; such conflicts are retried on the next run.
- Runs never overlap: an in-process flag plus a Postgres advisory lock (safe across instances). A skipped run reports `status: "skipped"`.
- Each discrepancy is logged at `warn` with pool id, contract pool id, database value, chain value and signed delta, followed by a per-run summary.
- Running it again on unchanged data changes nothing (idempotent).

Endpoints (JWT + admin required; non-admins get `403`):
- `POST /admin/reconciliation` runs a reconciliation now and returns the run summary.
- `GET /admin/reconciliation` returns cumulative counters (`driftDetected`, `driftCorrected`, `rpcErrors`, `dbErrors`, ...) and the last run, for scraping or alerting on `driftDetected`/`rpcErrors` growth.
## Platform stats

`GET /stats` (public) returns aggregate impact metrics computed from the database in a single `REPEATABLE READ` snapshot:

```json
{
  "totalRaised": [{ "asset": "XLM", "amount": "1500000000" }],
  "activePools": 12,
  "completedPools": 4,
  "uniqueDonors": 87
}
```

- `totalRaised`: `SUM(donations.amount)` per asset (exact `numeric` arithmetic in PostgreSQL), returned as integer **strings** in base units, ordered by asset code. Empty array when there are no donations.
- `activePools` / `completedPools`: pools with `PoolStatus.Active` / `PoolStatus.Completed`.
- `uniqueDonors`: `COUNT(DISTINCT donations.donor_wallet)`.
- Responses are cached in memory for `STATS_CACHE_TTL_MS` (default `30000`, `0` disables), so totals are never more than one TTL stale; concurrent cache misses share one query and failures are not cached.

## Run tests

```bash
# unit tests
$ npm run test

# e2e tests
$ npm run test:e2e

# test coverage
$ npm run test:cov

# lint (check only, never modifies files) / lint with auto-fix
$ npm run lint
$ npm run lint:fix
```

### CI expectations

Every push and pull request runs, in `nevo_server/`: `npm run lint`, the **entire**
unit suite with `npm test -- --runInBand --coverage` (no spec directories or
test names are filtered out), `npm run build`, and the OpenAPI export / API-type
check. A coverage summary is written to the job summary and `coverage-summary.json`
plus `lcov.info` are uploaded as the `server-coverage` artifact (also when tests fail).

Some specs talk to a real PostgreSQL. They read `TEST_DATABASE_URL`
(default `postgres://postgres:postgres@localhost:5432/nevo_test`) and each works
in its own schema or cleans up after itself, so a plain `nevo_test` database is
enough:

```bash
$ createdb nevo_test
$ TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/nevo_test npm test -- --runInBand
```

Jest compiles specs to CommonJS through `tsconfig.jest.json` (the application
itself builds with `nest build`); this is what lets specs use dynamic `import()`
and `jest.mock` without `--experimental-vm-modules`.

## Deployment

When you're ready to deploy your NestJS application to production, there are some key steps you can take to ensure it runs as efficiently as possible. Check out the [deployment documentation](https://docs.nestjs.com/deployment) for more information.

If you are looking for a cloud-based platform to deploy your NestJS application, check out [Mau](https://mau.nestjs.com), our official platform for deploying NestJS applications on AWS. Mau makes deployment straightforward and fast, requiring just a few simple steps:

```bash
$ npm install -g @nestjs/mau
$ mau deploy
```

With Mau, you can deploy your application in just a few clicks, allowing you to focus on building features rather than managing infrastructure.

## Resources

Check out a few resources that may come in handy when working with NestJS:

- Visit the [NestJS Documentation](https://docs.nestjs.com) to learn more about the framework.
- For questions and support, please visit our [Discord channel](https://discord.gg/G7Qnnhy).
- To dive deeper and get more hands-on experience, check out our official video [courses](https://courses.nestjs.com/).
- Deploy your application to AWS with the help of [NestJS Mau](https://mau.nestjs.com) in just a few clicks.
- Visualize your application graph and interact with the NestJS application in real-time using [NestJS Devtools](https://devtools.nestjs.com).
- Need help with your project (part-time to full-time)? Check out our official [enterprise support](https://enterprise.nestjs.com).
- To stay in the loop and get updates, follow us on [X](https://x.com/nestframework) and [LinkedIn](https://linkedin.com/company/nestjs).
- Looking for a job, or have a job to offer? Check out our official [Jobs board](https://jobs.nestjs.com).

## Support

Nest is an MIT-licensed open source project. It can grow thanks to the sponsors and support by the amazing backers. If you'd like to join them, please [read more here](https://docs.nestjs.com/support).

## Stay in touch

- Author - [Kamil Myśliwiec](https://twitter.com/kammysliwiec)
- Website - [https://nestjs.com](https://nestjs.com/)
- Twitter - [@nestframework](https://twitter.com/nestframework)

## License

Nest is [MIT licensed](https://github.com/nestjs/nest/blob/master/LICENSE).

//
