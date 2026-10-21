## Summary

- Handle duplicate donation transaction hashes idempotently and preserve nullable, empty, and long memos.
- Cover donation sort validation for pool and authenticated donor routes.
- Test the asset-length migration against PostgreSQL, including the 12-character boundary and case preservation.
- Run the focused donation tests in the server CI job with PostgreSQL.

## Validation

- `TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/nevo_test npm test -- --runInBand donations` — passed (39 tests).
- `npm run build` — passed.
- The full backend test suite has unrelated existing failures in `contract.service.spec.ts` and suites loading `@nestjs/schedule`.

Closes Web3Novalabs/Nevo#1515
Closes Web3Novalabs/Nevo#1514
Closes Web3Novalabs/Nevo#1513
Closes Web3Novalabs/Nevo#1512
