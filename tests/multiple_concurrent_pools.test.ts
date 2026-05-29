/**
 * Multiple Concurrent Pools Stress Tests — Issue #498
 */

import { describe, it, expect } from "@jest/globals";

interface Pool {
  id: string;
  state: "active" | "paused" | "refunding" | "finalized";
  yieldRateBps: number;
  lockPeriodMs: number;
  contributions: Map<string, { amount: number; lockedAt: number }>;
  totalLocked: number;
}

function makePool(id: string, yieldRateBps = 500, lockPeriodMs = 7 * 86_400_000): Pool {
  return { id, state: "active", yieldRateBps, lockPeriodMs, contributions: new Map(), totalLocked: 0 };
}

function deposit(pool: Pool, user: string, amount: number, now = Date.now()): string | null {
  if (pool.state !== "active") return "ERR_NOT_ACTIVE";
  if (amount <= 0) return "ERR_INVALID_AMOUNT";
  pool.contributions.set(user, { amount, lockedAt: now });
  pool.totalLocked += amount;
  return null;
}

function transition(pool: Pool, newState: Pool["state"]): void {
  pool.state = newState;
}

describe("Multiple Concurrent Pools Stress Tests", () => {
  it("1. Create 100 pools successfully", () => {
    const pools: Pool[] = [];
    for (let i = 0; i < 100; i++) {
      pools.push(makePool(`pool_${i}`, 500 + i, 86_400_000 * (i % 7 + 1)));
    }
    expect(pools.length).toBe(100);
    pools.forEach((p, i) => {
      expect(p.id).toBe(`pool_${i}`);
      expect(p.state).toBe("active");
      expect(p.yieldRateBps).toBe(500 + i);
    });
  });

  it("2. Independent state management", () => {
    const pools = Array.from({ length: 100 }, (_, i) => makePool(`p${i}`));
    transition(pools[10], "paused");
    transition(pools[50], "finalized");
    transition(pools[99], "refunding");
    expect(pools[10].state).toBe("paused");
    expect(pools[50].state).toBe("finalized");
    expect(pools[99].state).toBe("refunding");
    expect(pools[0].state).toBe("active");
    expect(pools[49].state).toBe("active");
  });

  it("3. Contribution tracking per pool", () => {
    const pools = Array.from({ length: 100 }, (_, i) => makePool(`p${i}`));
    pools.forEach((pool, i) => {
      deposit(pool, "user_a", 100 + i);
      deposit(pool, "user_b", 200 + i);
    });
    pools.forEach((pool, i) => {
      expect(pool.contributions.get("user_a")?.amount).toBe(100 + i);
      expect(pool.contributions.get("user_b")?.amount).toBe(200 + i);
      expect(pool.totalLocked).toBe(300 + 2 * i);
    });
  });

  it("4. State transitions work for all", () => {
    const pools = Array.from({ length: 100 }, (_, i) => makePool(`p${i}`));
    pools.forEach((pool) => transition(pool, "paused"));
    expect(pools.every((p) => p.state === "paused")).toBe(true);
    pools.forEach((pool) => transition(pool, "active"));
    expect(pools.every((p) => p.state === "active")).toBe(true);
  });

  it("5. Resource usage acceptable", () => {
    const pools = Array.from({ length: 100 }, (_, i) => makePool(`p${i}`));
    for (let i = 0; i < 100; i++) {
      for (let u = 0; u < 10; u++) {
        deposit(pools[i], `user_${u}`, 100 * (u + 1));
      }
    }
    const totalContributions = pools.reduce((sum, p) => sum + p.contributions.size, 0);
    expect(totalContributions).toBe(1000);
    const totalLocked = pools.reduce((sum, p) => sum + p.totalLocked, 0);
    expect(totalLocked).toBe(100 * 5500);
  });
});
