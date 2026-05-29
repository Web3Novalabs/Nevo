/**
 * Edge Case Combination Tests — Issue #513
 */

import { describe, it, expect, beforeEach } from "@jest/globals";

const STATES = {
  ACTIVE: "active", PAUSED: "paused", COMPLETED: "completed",
  CANCELLED: "cancelled", REFUNDING: "refunding", FINALIZED: "finalized",
} as const;
type State = typeof STATES[keyof typeof STATES];

interface Campaign {
  id: string; state: State; deadline: number; raised: number;
  target: number; minContrib: number; maxContrib: number;
  contributions: Map<string, number>;
}
interface Pool {
  id: string; state: State; contributions: Map<string, number>; totalLocked: number;
}

function makeCampaign(overrides: Partial<Campaign> = {}): Campaign {
  return { id: "camp_1", state: STATES.ACTIVE, deadline: Date.now() + 86_400_000,
    raised: 0, target: 10_000, minContrib: 10, maxContrib: 1_000,
    contributions: new Map(), ...overrides };
}

function makePool(overrides: Partial<Pool> = {}): Pool {
  return { id: "pool_1", state: STATES.ACTIVE, contributions: new Map(), totalLocked: 0, ...overrides };
}

function contribute(entity: Campaign | Pool, user: string, amount: number, now = Date.now()): string | null {
  if (entity.state === STATES.PAUSED)  return "ERR_PAUSED";
  if (entity.state !== STATES.ACTIVE)  return "ERR_NOT_ACTIVE";
  if ("deadline" in entity && now >= entity.deadline) return "ERR_DEADLINE_PASSED";
  if ("minContrib" in entity && amount < entity.minContrib) return "ERR_BELOW_MIN";
  if ("maxContrib" in entity && amount > entity.maxContrib) return "ERR_ABOVE_MAX";
  const prev = entity.contributions.get(user) ?? 0;
  entity.contributions.set(user, prev + amount);
  if ("raised" in entity) entity.raised += amount;
  if ("totalLocked" in entity) (entity as Pool).totalLocked += amount;
  return null;
}

function emergencyWithdraw(entity: Campaign | Pool, user: string): number | string {
  if (entity.state !== STATES.PAUSED) return "ERR_NOT_PAUSED";
  const amount = entity.contributions.get(user) ?? 0;
  if (amount === 0) return "ERR_NO_BALANCE";
  entity.contributions.delete(user);
  if ("raised" in entity) entity.raised -= amount;
  if ("totalLocked" in entity) (entity as Pool).totalLocked -= amount;
  return amount;
}

function checkCompletion(campaign: Campaign, now = Date.now()): State {
  if (now < campaign.deadline) return campaign.state;
  return campaign.raised >= campaign.target ? STATES.COMPLETED : STATES.CANCELLED;
}

function initiateRefund(pool: Pool): string | null {
  if (pool.state === STATES.REFUNDING) return "ERR_ALREADY_REFUNDING";
  if (pool.state === STATES.FINALIZED) return "ERR_FINALIZED";
  pool.state = STATES.REFUNDING; return null;
}

function processRefund(pool: Pool, user: string): number | string {
  if (pool.state !== STATES.REFUNDING) return "ERR_NOT_REFUNDING";
  const amount = pool.contributions.get(user) ?? 0;
  if (amount === 0) return "ERR_NO_BALANCE";
  pool.contributions.delete(user);
  pool.totalLocked -= amount;
  return amount;
}

describe("Edge Case Combinations", () => {
  describe("1. Paused contract + emergency withdrawal", () => {
    it("blocks new contributions when paused", () => {
      const camp = makeCampaign({ state: STATES.PAUSED });
      expect(contribute(camp, "alice", 100)).toBe("ERR_PAUSED");
    });
    it("allows emergency withdrawal when paused", () => {
      const camp = makeCampaign();
      contribute(camp, "alice", 500);
      camp.state = STATES.PAUSED;
      expect(emergencyWithdraw(camp, "alice")).toBe(500);
      expect(camp.contributions.has("alice")).toBe(false);
    });
    it("emergency withdrawal fails when not paused", () => {
      const camp = makeCampaign();
      contribute(camp, "alice", 500);
      expect(emergencyWithdraw(camp, "alice")).toBe("ERR_NOT_PAUSED");
    });
    it("emergency withdrawal returns error for non-contributor", () => {
      const camp = makeCampaign({ state: STATES.PAUSED });
      expect(emergencyWithdraw(camp, "stranger")).toBe("ERR_NO_BALANCE");
    });
    it("multiple users can emergency-withdraw independently when paused", () => {
      const camp = makeCampaign();
      contribute(camp, "alice", 300);
      contribute(camp, "bob", 700);
      camp.state = STATES.PAUSED;
      expect(emergencyWithdraw(camp, "alice")).toBe(300);
      expect(emergencyWithdraw(camp, "bob")).toBe(700);
      expect(camp.raised).toBe(0);
    });
  });

  describe("2. Campaign completion at exact deadline", () => {
    it("accepts contribution 1ms before deadline", () => {
      const deadline = Date.now() + 1000;
      expect(contribute(makeCampaign({ deadline }), "alice", 100, deadline - 1)).toBeNull();
    });
    it("rejects contribution at exact deadline moment", () => {
      const deadline = Date.now() + 1000;
      expect(contribute(makeCampaign({ deadline }), "alice", 100, deadline)).toBe("ERR_DEADLINE_PASSED");
    });
    it("marks COMPLETED when raised >= target at deadline", () => {
      const deadline = Date.now() + 1000;
      const camp = makeCampaign({ deadline, target: 500 });
      contribute(camp, "alice", 500, deadline - 1);
      camp.state = checkCompletion(camp, deadline + 1);
      expect(camp.state).toBe(STATES.COMPLETED);
    });
    it("marks CANCELLED when raised < target at deadline", () => {
      const deadline = Date.now() + 1000;
      const camp = makeCampaign({ deadline, target: 1000 });
      contribute(camp, "alice", 499, deadline - 1);
      camp.state = checkCompletion(camp, deadline + 1);
      expect(camp.state).toBe(STATES.CANCELLED);
    });
    it("exact target met at exact deadline → COMPLETED", () => {
      const deadline = Date.now() + 1000;
      const camp = makeCampaign({ deadline, target: 500 });
      contribute(camp, "alice", 500, deadline - 1);
      camp.state = checkCompletion(camp, deadline);
      expect(camp.state).toBe(STATES.COMPLETED);
    });
  });

  describe("3. Pool refund during state transition", () => {
    it("initiates refund from ACTIVE state", () => {
      const pool = makePool();
      expect(initiateRefund(pool)).toBeNull();
      expect(pool.state).toBe(STATES.REFUNDING);
    });
    it("blocks second refund initiation", () => {
      expect(initiateRefund(makePool({ state: STATES.REFUNDING }))).toBe("ERR_ALREADY_REFUNDING");
    });
    it("blocks refund initiation on FINALIZED pool", () => {
      expect(initiateRefund(makePool({ state: STATES.FINALIZED }))).toBe("ERR_FINALIZED");
    });
    it("processes user refund correctly during REFUNDING state", () => {
      const pool = makePool();
      contribute(pool, "alice", 400);
      initiateRefund(pool);
      expect(processRefund(pool, "alice")).toBe(400);
      expect(pool.totalLocked).toBe(0);
    });
    it("blocks contribution during REFUNDING state", () => {
      expect(contribute(makePool({ state: STATES.REFUNDING }), "alice", 100)).toBe("ERR_NOT_ACTIVE");
    });
    it("user cannot claim refund twice", () => {
      const pool = makePool();
      contribute(pool, "alice", 200);
      initiateRefund(pool);
      processRefund(pool, "alice");
      expect(processRefund(pool, "alice")).toBe("ERR_NO_BALANCE");
    });
  });

  describe("4. Multiple edge conditions together", () => {
    it("paused + at deadline + partial fill → emergency withdraw succeeds", () => {
      const deadline = Date.now() - 1;
      const camp = makeCampaign({ deadline, target: 1000, state: STATES.PAUSED });
      camp.contributions.set("alice", 499); camp.raised = 499;
      expect(emergencyWithdraw(camp, "alice")).toBe(499);
      expect(camp.raised).toBe(0);
    });
    it("min and max contribution bounds are both enforced simultaneously", () => {
      const camp = makeCampaign({ minContrib: 50, maxContrib: 200 });
      expect(contribute(camp, "u1", 49)).toBe("ERR_BELOW_MIN");
      expect(contribute(camp, "u1", 201)).toBe("ERR_ABOVE_MAX");
      expect(contribute(camp, "u1", 100)).toBeNull();
    });
    it("pool: contribute → pause → emergency withdraw → unpause → re-contribute", () => {
      const pool = makePool();
      contribute(pool, "alice", 300);
      pool.state = STATES.PAUSED;
      emergencyWithdraw(pool, "alice");
      pool.state = STATES.ACTIVE;
      expect(contribute(pool, "alice", 300)).toBeNull();
      expect(pool.totalLocked).toBe(300);
    });
    it("campaign: multiple users hit boundary conditions independently", () => {
      const camp = makeCampaign({ minContrib: 10, maxContrib: 500, target: 1000 });
      expect(contribute(camp, "u1", 10)).toBeNull();
      expect(contribute(camp, "u2", 500)).toBeNull();
      expect(contribute(camp, "u3", 9)).toBe("ERR_BELOW_MIN");
      expect(contribute(camp, "u4", 501)).toBe("ERR_ABOVE_MAX");
      expect(camp.raised).toBe(510);
    });
  });

  describe("5. Boundary condition interactions", () => {
    it("campaign target - 1: should be CANCELLED at deadline", () => {
      const deadline = Date.now() + 500;
      const camp = makeCampaign({ deadline, target: 1000 });
      contribute(camp, "u1", 999, deadline - 1);
      camp.state = checkCompletion(camp, deadline + 1);
      expect(camp.state).toBe(STATES.CANCELLED);
    });
    it("campaign target exactly: should be COMPLETED at deadline", () => {
      const deadline = Date.now() + 500;
      const camp = makeCampaign({ deadline, target: 1000 });
      contribute(camp, "u1", 1000, deadline - 1);
      camp.state = checkCompletion(camp, deadline + 1);
      expect(camp.state).toBe(STATES.COMPLETED);
    });
    it("campaign target + 1: still COMPLETED (over-funded)", () => {
      const deadline = Date.now() + 500;
      const camp = makeCampaign({ deadline, target: 1000, maxContrib: 5000 });
      contribute(camp, "u1", 1001, deadline - 1);
      camp.state = checkCompletion(camp, deadline + 1);
      expect(camp.state).toBe(STATES.COMPLETED);
    });
    it("zero totalLocked after all refunds processed", () => {
      const pool = makePool();
      ["a","b","c"].forEach((u) => contribute(pool, u, 100));
      initiateRefund(pool);
      ["a","b","c"].forEach((u) => processRefund(pool, u));
      expect(pool.totalLocked).toBe(0);
      expect(pool.contributions.size).toBe(0);
    });
    it("single-wei contribution at minimum boundary is accepted", () => {
      expect(contribute(makeCampaign({ minContrib: 1 }), "u1", 1)).toBeNull();
    });
  });
});
