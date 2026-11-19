import { getDonorCount } from '@/lib/getDonorCount';
import type { Pool } from '@/src/store/poolsStore';

function createPool(id: string, raised = 0): Pool {
  return {
    id,
    title: 'Test pool',
    description: 'A test pool',
    category: 'Community',
    status: 'Active',
    target: 1000,
    raised,
    imageColor: '#22c55e',
  };
}

describe('getDonorCount', () => {
  it.each([
    ['1', 42],
    ['2', 87],
    ['3', 31],
    ['4', 15],
    ['5', 5],
  ])(
    'returns the expected count for hardcoded pool ID %s',
    (id, expectedCount) => {
      expect(getDonorCount(createPool(id))).toBe(expectedCount);
    }
  );

  it('uses the fallback formula for an unrecognised pool ID', () => {
    const pool = createPool('custom-pool-id', 250);

    expect(getDonorCount(pool)).toBe(19);
  });
});
