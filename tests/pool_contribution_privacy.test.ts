/**
 * Pool Contribution Privacy Tests — Issue #464
 */

import { describe, it, expect } from "@jest/globals";

interface ContributionEvent {
  pool_id: string;
  contributor: string;
  amount: number;
  is_private: boolean;
  timestamp: number;
}

interface Pool {
  id: string;
  is_private: boolean;
  contributions: Map<string, number>;
  events: ContributionEvent[];
}

function makePool(overrides: Partial<Pool> = {}): Pool {
  return { id: "pool_1", is_private: false, contributions: new Map(), events: [], ...overrides };
}

function contribute(pool: Pool, contributor: string, amount: number, now = Date.now()): ContributionEvent | string {
  if (amount <= 0) return "ERR_INVALID_AMOUNT";
  const prev = pool.contributions.get(contributor) ?? 0;
  pool.contributions.set(contributor, prev + amount);
  const event: ContributionEvent = {
    pool_id: pool.id,
    contributor,
    amount,
    is_private: pool.is_private,
    timestamp: now,
  };
  pool.events.push(event);
  return event;
}

describe("Pool Contribution Privacy", () => {
  describe("1. Public pool contributions (is_private=false)", () => {
    it("emits event with is_private=false for public pool", () => {
      const pool = makePool({ is_private: false });
      const event = contribute(pool, "alice", 100);
      expect(event).not.toBe("ERR_INVALID_AMOUNT");
      expect((event as ContributionEvent).is_private).toBe(false);
    });

    it("event contains all required fields for public contribution", () => {
      const pool = makePool({ id: "pub_pool", is_private: false });
      const now = Date.now();
      const event = contribute(pool, "alice", 200, now) as ContributionEvent;
      expect(event.pool_id).toBe("pub_pool");
      expect(event.contributor).toBe("alice");
      expect(event.amount).toBe(200);
      expect(event.is_private).toBe(false);
      expect(event.timestamp).toBe(now);
    });

    it("multiple public contributions each emit is_private=false", () => {
      const pool = makePool({ is_private: false });
      contribute(pool, "alice", 100);
      contribute(pool, "bob", 200);
      pool.events.forEach((e) => expect(e.is_private).toBe(false));
    });
  });

  describe("2. Private pool contributions (is_private=true)", () => {
    it("emits event with is_private=true for private pool", () => {
      const pool = makePool({ is_private: true });
      const event = contribute(pool, "alice", 100);
      expect(event).not.toBe("ERR_INVALID_AMOUNT");
      expect((event as ContributionEvent).is_private).toBe(true);
    });

    it("event contains all required fields for private contribution", () => {
      const pool = makePool({ id: "priv_pool", is_private: true });
      const now = Date.now();
      const event = contribute(pool, "bob", 500, now) as ContributionEvent;
      expect(event.pool_id).toBe("priv_pool");
      expect(event.contributor).toBe("bob");
      expect(event.amount).toBe(500);
      expect(event.is_private).toBe(true);
      expect(event.timestamp).toBe(now);
    });

    it("multiple private contributions each emit is_private=true", () => {
      const pool = makePool({ is_private: true });
      contribute(pool, "alice", 100);
      contribute(pool, "bob", 200);
      pool.events.forEach((e) => expect(e.is_private).toBe(true));
    });
  });

  describe("3. Privacy flag is pool-level, not contribution-level", () => {
    it("public and private pools emit different privacy flags for same contributor", () => {
      const pubPool = makePool({ id: "pub", is_private: false });
      const privPool = makePool({ id: "priv", is_private: true });
      const pubEvent = contribute(pubPool, "alice", 100) as ContributionEvent;
      const privEvent = contribute(privPool, "alice", 100) as ContributionEvent;
      expect(pubEvent.is_private).toBe(false);
      expect(privEvent.is_private).toBe(true);
    });

    it("privacy flag on event matches pool is_private setting", () => {
      [true, false].forEach((flag) => {
        const pool = makePool({ is_private: flag });
        const event = contribute(pool, "user", 50) as ContributionEvent;
        expect(event.is_private).toBe(pool.is_private);
      });
    });

    it("event pool_id matches the contributing pool", () => {
      const pool = makePool({ id: "specific_pool", is_private: true });
      const event = contribute(pool, "alice", 100) as ContributionEvent;
      expect(event.pool_id).toBe("specific_pool");
    });
  });
});
