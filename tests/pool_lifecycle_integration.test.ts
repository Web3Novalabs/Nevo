/**
 * Pool Lifecycle Integration Tests — Issue #495
 *
 * Test complete pool workflow:
 * (1) Create pool with metadata.
 * (2) Multiple contributions.
 * (3) State transitions.
 * (4) Refund scenario.
 * (5) Pool closure.
 * (6) All state changes tracked correctly.
 */

import { describe, it, expect } from "@jest/globals";

interface PoolMetadata {
  title: string;
  description: string;
  imageHash?: string;
}

interface Contribution {
  amount: number;
  contributedAt: number;
}

interface Pool {
  id: string;
  metadata: PoolMetadata;
  goal: number;
  collected: number;
  state: "active" | "paused" | "refunding" | "closed" | "finalized";
  contributions: Map<string, Contribution>;
  stateHistory: Array<{ state: Pool["state"]; at: number }>;
}

function createPool(id: string, metadata: PoolMetadata, goal: number, now = Date.now()): Pool {
  const pool: Pool = {
    id, metadata, goal, collected: 0,
    state: "active",
    contributions: new Map(),
    stateHistory: [{ state: "active", at: now }],
  };
  return pool;
}

function contribute(pool: Pool, user: string, amount: number, now = Date.now()): string | null {
  if (pool.state !== "active") return "ERR_NOT_ACTIVE";
  if (amount <= 0)              return "ERR_INVALID_AMOUNT";
  const prev = pool.contributions.get(user);
  pool.contributions.set(user, { amount: (prev?.amount ?? 0) + amount, contributedAt: now });
  pool.collected += amount;
  return null;
}

function transitionState(pool: Pool, newState: Pool["state"], now = Date.now()): string | null {
  const { state } = pool;
  const allowed: Record<Pool["state"], Pool["state"][]> = {
    active:    ["paused", "refunding", "closed"],
    paused:    ["active", "refunding", "closed"],
    refunding: ["closed"],
    closed:    ["finalized"],
    finalized: [],
  };
  if (!allowed[state].includes(newState)) return `ERR_INVALID_TRANSITION:${state}→${newState}`;
  pool.state = newState;
  pool.stateHistory.push({ state: newState, at: now });
  return null;
}

function processRefund(pool: Pool, user: string): number | string {
  if (pool.state !== "refunding") return "ERR_NOT_REFUNDING";
  const entry = pool.contributions.get(user);
  if (!entry || entry.amount === 0) return "ERR_NO_BALANCE";
  const refundAmount = entry.amount;
  pool.contributions.delete(user);
  pool.collected -= refundAmount;
  return refundAmount;
}

describe("Pool Lifecycle Integration", () => {
  describe("1. Create pool with metadata", () => {
    it("creates pool with correct metadata", () => {
      const meta: PoolMetadata = { title: "Scholarship Pool", description: "Funding students", imageHash: "abc123" };
      const pool = createPool("p1", meta, 50_000);
      expect(pool.metadata.title).toBe("Scholarship Pool");
      expect(pool.metadata.description).toBe("Funding students");
      expect(pool.metadata.imageHash).toBe("abc123");
    });

    it("pool starts in active state with zero collected", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 10_000);
      expect(pool.state).toBe("active");
      expect(pool.collected).toBe(0);
    });

    it("pool goal is stored correctly", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 75_000);
      expect(pool.goal).toBe(75_000);
    });

    it("initial state is recorded in history", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000, 42);
      expect(pool.stateHistory).toHaveLength(1);
      expect(pool.stateHistory[0].state).toBe("active");
      expect(pool.stateHistory[0].at).toBe(42);
    });

    it("pool starts with no contributions", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      expect(pool.contributions.size).toBe(0);
    });
  });

  describe("2. Multiple contributions", () => {
    it("accepts contributions from multiple users", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      expect(contribute(pool, "alice", 1_000)).toBeNull();
      expect(contribute(pool, "bob", 2_000)).toBeNull();
      expect(contribute(pool, "carol", 3_000)).toBeNull();
      expect(pool.contributions.size).toBe(3);
    });

    it("collected amount reflects all contributions", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 1_000);
      contribute(pool, "bob", 2_000);
      contribute(pool, "carol", 3_000);
      expect(pool.collected).toBe(6_000);
    });

    it("same user contributions accumulate", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 500);
      contribute(pool, "alice", 500);
      expect(pool.contributions.get("alice")?.amount).toBe(1_000);
    });

    it("rejects zero and negative amounts", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      expect(contribute(pool, "alice", 0)).toBe("ERR_INVALID_AMOUNT");
      expect(contribute(pool, "alice", -100)).toBe("ERR_INVALID_AMOUNT");
      expect(pool.collected).toBe(0);
    });

    it("rejects contributions when pool is not active", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      transitionState(pool, "paused");
      expect(contribute(pool, "alice", 100)).toBe("ERR_NOT_ACTIVE");
    });
  });

  describe("3. State transitions", () => {
    it("active → paused is allowed", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      expect(transitionState(pool, "paused")).toBeNull();
      expect(pool.state).toBe("paused");
    });

    it("paused → active (resume) is allowed", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      transitionState(pool, "paused");
      expect(transitionState(pool, "active")).toBeNull();
      expect(pool.state).toBe("active");
    });

    it("active → refunding is allowed", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      expect(transitionState(pool, "refunding")).toBeNull();
      expect(pool.state).toBe("refunding");
    });

    it("active → closed is allowed", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      expect(transitionState(pool, "closed")).toBeNull();
      expect(pool.state).toBe("closed");
    });

    it("closed → finalized is allowed", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      transitionState(pool, "closed");
      expect(transitionState(pool, "finalized")).toBeNull();
      expect(pool.state).toBe("finalized");
    });

    it("finalized → any state is blocked", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      transitionState(pool, "closed");
      transitionState(pool, "finalized");
      (["active", "paused", "refunding", "closed"] as Pool["state"][]).forEach((s) => {
        expect(transitionState(pool, s)).toMatch(/ERR_INVALID_TRANSITION/);
      });
    });

    it("invalid transition returns error string", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      expect(transitionState(pool, "finalized")).toMatch(/ERR_INVALID_TRANSITION/);
    });
  });

  describe("4. Refund scenario", () => {
    it("users can claim refund when pool is in refunding state", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 1_000);
      transitionState(pool, "refunding");
      expect(processRefund(pool, "alice")).toBe(1_000);
    });

    it("refund removes user from contributions", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 1_000);
      transitionState(pool, "refunding");
      processRefund(pool, "alice");
      expect(pool.contributions.has("alice")).toBe(false);
    });

    it("collected decreases after refund", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 1_000);
      contribute(pool, "bob", 2_000);
      transitionState(pool, "refunding");
      processRefund(pool, "alice");
      expect(pool.collected).toBe(2_000);
    });

    it("all users can be refunded independently", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 500);
      contribute(pool, "bob", 700);
      contribute(pool, "carol", 300);
      transitionState(pool, "refunding");
      expect(processRefund(pool, "alice")).toBe(500);
      expect(processRefund(pool, "bob")).toBe(700);
      expect(processRefund(pool, "carol")).toBe(300);
      expect(pool.collected).toBe(0);
    });

    it("refund fails when pool is not in refunding state", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 1_000);
      expect(processRefund(pool, "alice")).toBe("ERR_NOT_REFUNDING");
    });

    it("refund fails for user with no balance", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      transitionState(pool, "refunding");
      expect(processRefund(pool, "ghost")).toBe("ERR_NO_BALANCE");
    });

    it("user cannot claim refund twice", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 1_000);
      transitionState(pool, "refunding");
      processRefund(pool, "alice");
      expect(processRefund(pool, "alice")).toBe("ERR_NO_BALANCE");
    });
  });

  describe("5. Pool closure", () => {
    it("pool can be closed from active state", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      expect(transitionState(pool, "closed")).toBeNull();
      expect(pool.state).toBe("closed");
    });

    it("contributions are blocked after closure", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      transitionState(pool, "closed");
      expect(contribute(pool, "alice", 100)).toBe("ERR_NOT_ACTIVE");
    });

    it("collected amount is preserved after closure", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 100_000);
      contribute(pool, "alice", 5_000);
      transitionState(pool, "closed");
      expect(pool.collected).toBe(5_000);
    });

    it("closed pool can be finalized", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000);
      transitionState(pool, "closed");
      transitionState(pool, "finalized");
      expect(pool.state).toBe("finalized");
    });
  });

  describe("6. All state changes tracked correctly", () => {
    it("each transition appends to state history", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000, 0);
      transitionState(pool, "paused", 1);
      transitionState(pool, "active", 2);
      transitionState(pool, "closed", 3);
      expect(pool.stateHistory).toHaveLength(4);
    });

    it("state history records correct states in order", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000, 0);
      transitionState(pool, "paused", 1);
      transitionState(pool, "active", 2);
      transitionState(pool, "closed", 3);
      const states = pool.stateHistory.map((h) => h.state);
      expect(states).toEqual(["active", "paused", "active", "closed"]);
    });

    it("state history timestamps are monotonically increasing", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000, 0);
      transitionState(pool, "paused", 10);
      transitionState(pool, "active", 20);
      transitionState(pool, "closed", 30);
      for (let i = 1; i < pool.stateHistory.length; i++) {
        expect(pool.stateHistory[i].at).toBeGreaterThan(pool.stateHistory[i - 1].at);
      }
    });

    it("failed transitions do not append to history", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000, 0);
      transitionState(pool, "finalized"); // invalid
      expect(pool.stateHistory).toHaveLength(1);
    });

    it("full lifecycle history is complete: active→paused→active→refunding→closed→finalized", () => {
      const pool = createPool("p1", { title: "T", description: "D" }, 1_000, 0);
      transitionState(pool, "paused", 1);
      transitionState(pool, "active", 2);
      transitionState(pool, "refunding", 3);
      transitionState(pool, "closed", 4);
      transitionState(pool, "finalized", 5);
      const states = pool.stateHistory.map((h) => h.state);
      expect(states).toEqual(["active", "paused", "active", "refunding", "closed", "finalized"]);
    });
  });
});
