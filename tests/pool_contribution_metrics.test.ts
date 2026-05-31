/**
 * Pool Contribution Metrics Tracking Tests — Issue #463
 */

import { describe, it, expect } from "@jest/globals";

interface PoolMetrics {
  contributor_count: number;
  total_raised: number;
  last_donation_at: number;
  contributors: Set<string>;
}

function makeMetrics(): PoolMetrics {
  return { contributor_count: 0, total_raised: 0, last_donation_at: 0, contributors: new Set() };
}

function recordContribution(metrics: PoolMetrics, contributor: string, amount: number, now: number): void {
  if (!metrics.contributors.has(contributor)) {
    metrics.contributors.add(contributor);
    metrics.contributor_count++;
  }
  metrics.total_raised += amount;
  metrics.last_donation_at = now;
}

describe("Pool contribution metrics tracking", () => {
  it("first contribution increments contributor_count to 1", () => {
    const m = makeMetrics();
    recordContribution(m, "alice", 100, 1000);
    expect(m.contributor_count).toBe(1);
  });

  it("same contributor's second contribution keeps count at 1", () => {
    const m = makeMetrics();
    recordContribution(m, "alice", 100, 1000);
    recordContribution(m, "alice", 200, 2000);
    expect(m.contributor_count).toBe(1);
  });

  it("different contributor increments count to 2", () => {
    const m = makeMetrics();
    recordContribution(m, "alice", 100, 1000);
    recordContribution(m, "bob", 200, 2000);
    expect(m.contributor_count).toBe(2);
  });

  it("total_raised updates correctly with multiple contributions", () => {
    const m = makeMetrics();
    recordContribution(m, "alice", 100, 1000);
    recordContribution(m, "alice", 200, 2000);
    recordContribution(m, "bob", 300, 3000);
    expect(m.total_raised).toBe(600);
  });

  it("last_donation_at timestamp updates properly", () => {
    const m = makeMetrics();
    recordContribution(m, "alice", 100, 1000);
    expect(m.last_donation_at).toBe(1000);
    recordContribution(m, "bob", 200, 5000);
    expect(m.last_donation_at).toBe(5000);
  });
});
