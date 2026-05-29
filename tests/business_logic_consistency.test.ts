/**
 * Business Logic Consistency Tests — Issue #516
 */

import { describe, it, expect } from "@jest/globals";

interface Campaign {
  id: string;
  state: "active" | "paused" | "completed" | "cancelled" | "finalized";
  target: number; raised: number; deadline: number;
  minContrib: number; maxContrib: number; feeRateBps: number;
  contributions: Map<string, number>;
}
interface Pool {
  id: string; state: "active" | "paused" | "refunding" | "finalized";
  yieldRateBps: number; lockPeriodMs: number;
  contributions: Map<string, { amount: number; lockedAt: number }>;
  totalLocked: number;
}

function calculateFee(amount: number, feeRateBps: number): number {
  return Math.floor((amount * feeRateBps) / 10_000);
}
function netAfterFee(amount: number, feeRateBps: number): number {
  return amount - calculateFee(amount, feeRateBps);
}
function calculateYield(amount: number, yieldRateBps: number): number {
  return Math.floor((amount * yieldRateBps) / 10_000);
}
function validateContribution(camp: Campaign, user: string, amount: number, now = Date.now()): string | null {
  if (camp.state !== "active")      return "ERR_NOT_ACTIVE";
  if (now >= camp.deadline)         return "ERR_DEADLINE_PASSED";
  if (amount < camp.minContrib)     return "ERR_BELOW_MIN";
  if (amount > camp.maxContrib)     return "ERR_ABOVE_MAX";
  return null;
}
function applyContribution(camp: Campaign, user: string, amount: number): void {
  camp.contributions.set(user, (camp.contributions.get(user) ?? 0) + amount);
  camp.raised += amount;
}
function poolDeposit(pool: Pool, user: string, amount: number, now = Date.now()): string | null {
  if (pool.state !== "active") return "ERR_NOT_ACTIVE";
  if (amount <= 0)              return "ERR_INVALID_AMOUNT";
  pool.contributions.set(user, { amount, lockedAt: now });
  pool.totalLocked += amount; return null;
}
function poolWithdraw(pool: Pool, user: string, now = Date.now()): number | string {
  const entry = pool.contributions.get(user);
  if (!entry) return "ERR_NO_DEPOSIT";
  if (now < entry.lockedAt + pool.lockPeriodMs) return "ERR_STILL_LOCKED";
  const yieldEarned = calculateYield(entry.amount, pool.yieldRateBps);
  pool.contributions.delete(user);
  pool.totalLocked -= entry.amount;
  return entry.amount + yieldEarned;
}
function transition(camp: Campaign, action: "pause"|"resume"|"cancel"|"finalize"): string | null {
  const { state } = camp;
  if (action === "pause"    && state === "active")    { camp.state = "paused";    return null; }
  if (action === "resume"   && state === "paused")    { camp.state = "active";    return null; }
  if (action === "cancel"   && (state === "active" || state === "paused")) { camp.state = "cancelled"; return null; }
  if (action === "finalize" && state === "completed") { camp.state = "finalized"; return null; }
  return `ERR_INVALID_TRANSITION:${state}→${action}`;
}

function makeCamp(o: Partial<Campaign> = {}): Campaign {
  return { id: "c1", state: "active", target: 10_000, raised: 0,
    deadline: Date.now() + 86_400_000, minContrib: 10, maxContrib: 1_000,
    feeRateBps: 250, contributions: new Map(), ...o };
}
function makePool(o: Partial<Pool> = {}): Pool {
  return { id: "p1", state: "active", yieldRateBps: 500,
    lockPeriodMs: 7 * 86_400_000, contributions: new Map(), totalLocked: 0, ...o };
}

describe("Business Logic Consistency", () => {
  describe("1. Campaign rules consistently applied", () => {
    it("validates contribution amount against min/max for every user", () => {
      const camp = makeCamp({ minContrib: 100, maxContrib: 500 });
      const cases: [string, number, string | null][] = [
        ["u1", 99, "ERR_BELOW_MIN"], ["u2", 100, null], ["u3", 500, null], ["u4", 501, "ERR_ABOVE_MAX"],
      ];
      cases.forEach(([user, amount, expected]) => expect(validateContribution(camp, user, amount)).toBe(expected));
    });
    it("raised amount grows correctly for sequential contributions", () => {
      const camp = makeCamp();
      applyContribution(camp, "a", 200);
      applyContribution(camp, "b", 300);
      applyContribution(camp, "c", 500);
      expect(camp.raised).toBe(1000);
    });
    it("same user can contribute multiple times (accumulates)", () => {
      const camp = makeCamp();
      applyContribution(camp, "alice", 100);
      applyContribution(camp, "alice", 200);
      expect(camp.contributions.get("alice")).toBe(300);
    });
    it("campaign rules apply equally regardless of contribution order", () => {
      const camp1 = makeCamp(); const camp2 = makeCamp();
      [100, 200, 300].forEach((a) => applyContribution(camp1, "u", a));
      [300, 200, 100].forEach((a) => applyContribution(camp2, "u", a));
      expect(camp1.raised).toBe(camp2.raised);
      expect(camp1.contributions.get("u")).toBe(camp2.contributions.get("u"));
    });
  });

  describe("2. Pool rules enforced uniformly", () => {
    it("rejects deposits in non-active states", () => {
      (["paused", "refunding", "finalized"] as const).forEach((state) =>
        expect(poolDeposit(makePool({ state }), "u1", 100)).toBe("ERR_NOT_ACTIVE")
      );
    });
    it("rejects zero and negative deposit amounts", () => {
      const pool = makePool();
      expect(poolDeposit(pool, "u1", 0)).toBe("ERR_INVALID_AMOUNT");
      expect(poolDeposit(pool, "u1", -1)).toBe("ERR_INVALID_AMOUNT");
    });
    it("totalLocked reflects all deposits accurately", () => {
      const pool = makePool();
      poolDeposit(pool, "a", 400); poolDeposit(pool, "b", 600);
      expect(pool.totalLocked).toBe(1000);
    });
    it("withdrawal blocked before lock period elapses", () => {
      const now = Date.now();
      const pool = makePool({ lockPeriodMs: 1_000 });
      poolDeposit(pool, "alice", 500, now);
      expect(poolWithdraw(pool, "alice", now + 999)).toBe("ERR_STILL_LOCKED");
    });
    it("withdrawal succeeds exactly at lock period boundary", () => {
      const now = Date.now();
      const pool = makePool({ lockPeriodMs: 1_000, yieldRateBps: 0 });
      poolDeposit(pool, "alice", 500, now);
      expect(poolWithdraw(pool, "alice", now + 1_000)).toBe(500);
    });
  });

  describe("3. Fee calculations accurate", () => {
    it("2.5% fee on round numbers is correct", () => {
      expect(calculateFee(1_000, 250)).toBe(25);
      expect(calculateFee(10_000, 250)).toBe(250);
    });
    it("fee is zero for 0 bps", () => {
      expect(calculateFee(1_000, 0)).toBe(0);
    });
    it("fee floors fractional results", () => {
      expect(calculateFee(99, 100)).toBe(0);
      expect(calculateFee(101, 100)).toBe(1);
    });
    it("net after fee + fee == original amount (within floor rounding)", () => {
      const amounts = [100, 999, 1_234, 50_000];
      const bps = [100, 250, 300, 500];
      amounts.forEach((a) => bps.forEach((b) => {
        const fee = calculateFee(a, b); const net = netAfterFee(a, b);
        expect(net + fee).toBeLessThanOrEqual(a);
        expect(a - (net + fee)).toBeLessThanOrEqual(1);
      }));
    });
    it("5% yield on pool deposit is calculated correctly", () => {
      expect(calculateYield(1_000, 500)).toBe(50);
      expect(calculateYield(200, 500)).toBe(10);
    });
    it("yield + principal equals expected withdrawal amount", () => {
      const now = Date.now();
      const pool = makePool({ yieldRateBps: 500, lockPeriodMs: 0 });
      poolDeposit(pool, "alice", 1_000, now);
      expect(poolWithdraw(pool, "alice", now + 1)).toBe(1_050);
    });
  });

  describe("4. Time-based rules work correctly", () => {
    it("contribution accepted at any point before deadline", () => {
      const camp = makeCamp({ deadline: Date.now() + 1_000_000 });
      expect(validateContribution(camp, "u", 100, Date.now())).toBeNull();
    });
    it("contribution rejected at deadline (boundary)", () => {
      const dl = Date.now() + 100;
      expect(validateContribution(makeCamp({ deadline: dl }), "u", 100, dl)).toBe("ERR_DEADLINE_PASSED");
    });
    it("contribution rejected after deadline", () => {
      expect(validateContribution(makeCamp({ deadline: Date.now() - 1 }), "u", 100)).toBe("ERR_DEADLINE_PASSED");
    });
    it("pool lock period enforced with ms precision", () => {
      const now = 1_000_000;
      const pool = makePool({ lockPeriodMs: 500, yieldRateBps: 0 });
      poolDeposit(pool, "u", 100, now);
      expect(poolWithdraw(pool, "u", now + 499)).toBe("ERR_STILL_LOCKED");
      expect(poolWithdraw(pool, "u", now + 500)).toBe(100);
    });
    it("time-based rules independent of contribution amount", () => {
      const camp = makeCamp({ deadline: Date.now() - 1 });
      [10, 100, 500, 1000].forEach((a) =>
        expect(validateContribution(camp, "u", a)).toBe("ERR_DEADLINE_PASSED")
      );
    });
  });

  describe("5. State machine logic sound", () => {
    it("active → paused → active (resume)", () => {
      const camp = makeCamp();
      expect(transition(camp, "pause")).toBeNull(); expect(camp.state).toBe("paused");
      expect(transition(camp, "resume")).toBeNull(); expect(camp.state).toBe("active");
    });
    it("active → cancelled", () => {
      const camp = makeCamp();
      expect(transition(camp, "cancel")).toBeNull(); expect(camp.state).toBe("cancelled");
    });
    it("paused → cancelled", () => {
      const camp = makeCamp({ state: "paused" });
      expect(transition(camp, "cancel")).toBeNull(); expect(camp.state).toBe("cancelled");
    });
    it("completed → finalized", () => {
      const camp = makeCamp({ state: "completed" });
      expect(transition(camp, "finalize")).toBeNull(); expect(camp.state).toBe("finalized");
    });
    it("invalid transitions return error strings", () => {
      expect(transition(makeCamp({ state: "cancelled" }), "pause")).toMatch(/ERR_INVALID_TRANSITION/);
      expect(transition(makeCamp({ state: "finalized" }), "resume")).toMatch(/ERR_INVALID_TRANSITION/);
      expect(transition(makeCamp({ state: "active" }), "finalize")).toMatch(/ERR_INVALID_TRANSITION/);
    });
    it("state machine is deterministic", () => {
      const run = () => {
        const c = makeCamp();
        transition(c, "pause"); transition(c, "resume"); transition(c, "cancel");
        return c.state;
      };
      expect(run()).toBe("cancelled"); expect(run()).toBe("cancelled");
    });
    it("terminal states block all further transitions", () => {
      const terminals: Array<Campaign["state"]> = ["cancelled", "finalized"];
      const actions = ["pause", "resume", "cancel", "finalize"] as const;
      terminals.forEach((state) =>
        actions.forEach((action) =>
          expect(transition(makeCamp({ state }), action)).toMatch(/ERR_INVALID_TRANSITION/)
        )
      );
    });
  });
});
