import type { Pool } from '@/src/store/poolsStore';

/**
 * Approximates the number of donors for a pool.
 *
 * TEMPORARY: this is a frontend-only approximation until a real donor-count
 * API exists. The contract exposes `get_donor_count` on-chain (see
 * `ContractService.getDonorCountOnChain`), but nothing surfaces it to the
 * frontend yet — replace this function once that endpoint is available.
 *
 * Pool IDs '1'–'5' are the seeded demo pools created with the app, so their
 * donor counts are hardcoded instead of estimated to keep the demo data
 * stable and realistic.
 *
 * Every other pool falls back to an estimate derived from `pool.raised`:
 * `Math.floor((raised * 7.3) / 100) + 1` assumes each donor contributes
 * roughly 7.3% of the raised total (~13.7 units) on average, so the donor
 * count is approximated as 7.3% of the raised amount. The `+ 1` guarantees
 * a pool that has raised anything is never shown with zero donors.
 */
export function getDonorCount(pool: Pool): number {
  if (pool.id === '1') return 42;
  if (pool.id === '2') return 87;
  if (pool.id === '3') return 31;
  if (pool.id === '4') return 15;
  if (pool.id === '5') return 5;
  return Math.floor((pool.raised * 7.3) / 100) + 1;
}
