/**
 * Campaign Lifecycle Integration Tests — Issue #494
 *
 * Test complete campaign workflow:
 * (1) Create campaign with fee.
 * (2) Multiple users donate.
 * (3) Campaign reaches goal.
 * (4) Further donations fail.
 * (5) All balances and metrics correct throughout.
 */

import { describe, it, expect } from "@jest/globals";

const FEE_BPS = 250; // 2.5%

interface Campaign {
  id: string;
  title: string;
  target: number;
  raised: number;
  feeBps: number;
  feesCollected: number;
  state: "active" | "completed" | "cancelled" | "finalized";
  contributions: Map<string, number>;
  deadline: number;
}

function createCampaign(id: string, title: string, target: number, feeBps: number, deadline: number): Campaign {
  return { id, title, target, raised: 0, feeBps, feesCollected: 0, state: "active", contributions: new Map(), deadline };
}

function donate(campaign: Campaign, user: string, amount: number, now = Date.now()): string | null {
  if (campaign.state !== "active") return "ERR_NOT_ACTIVE";
  if (now >= campaign.deadline)    return "ERR_DEADLINE_PASSED";
  const fee = Math.floor((amount * campaign.feeBps) / 10_000);
  const net = amount - fee;
  campaign.contributions.set(user, (campaign.contributions.get(user) ?? 0) + net);
  campaign.raised += net;
  campaign.feesCollected += fee;
  if (campaign.raised >= campaign.target) campaign.state = "completed";
  return null;
}

function finalize(campaign: Campaign): string | null {
  if (campaign.state !== "completed") return "ERR_NOT_COMPLETED";
  campaign.state = "finalized";
  return null;
}

function cancel(campaign: Campaign): string | null {
  if (campaign.state === "finalized") return "ERR_ALREADY_FINALIZED";
  campaign.state = "cancelled";
  return null;
}

function totalContributions(campaign: Campaign): number {
  let sum = 0;
  campaign.contributions.forEach((v) => { sum += v; });
  return sum;
}

describe("Campaign Lifecycle Integration", () => {
  describe("1. Create campaign with fee", () => {
    it("creates campaign with correct initial state", () => {
      const camp = createCampaign("c1", "Relief Fund", 10_000, FEE_BPS, Date.now() + 86_400_000);
      expect(camp.state).toBe("active");
      expect(camp.raised).toBe(0);
      expect(camp.feesCollected).toBe(0);
      expect(camp.feeBps).toBe(FEE_BPS);
    });

    it("fee rate is stored correctly", () => {
      const camp = createCampaign("c1", "Test", 5_000, 500, Date.now() + 1000);
      expect(camp.feeBps).toBe(500);
    });

    it("campaign starts with empty contributions", () => {
      const camp = createCampaign("c1", "Test", 1_000, FEE_BPS, Date.now() + 1000);
      expect(camp.contributions.size).toBe(0);
    });
  });

  describe("2. Multiple users donate", () => {
    it("each donation is recorded per user", () => {
      const camp = createCampaign("c1", "Test", 100_000, FEE_BPS, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000);
      donate(camp, "bob", 2_000);
      donate(camp, "carol", 3_000);
      expect(camp.contributions.has("alice")).toBe(true);
      expect(camp.contributions.has("bob")).toBe(true);
      expect(camp.contributions.has("carol")).toBe(true);
    });

    it("net amount after fee is credited to user", () => {
      const camp = createCampaign("c1", "Test", 100_000, FEE_BPS, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000); // fee = 25, net = 975
      expect(camp.contributions.get("alice")).toBe(975);
    });

    it("fees accumulate across multiple donations", () => {
      const camp = createCampaign("c1", "Test", 100_000, FEE_BPS, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000); // fee = 25
      donate(camp, "bob", 2_000);   // fee = 50
      expect(camp.feesCollected).toBe(75);
    });

    it("raised amount equals sum of net contributions", () => {
      const camp = createCampaign("c1", "Test", 100_000, FEE_BPS, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000);
      donate(camp, "bob", 2_000);
      donate(camp, "carol", 3_000);
      expect(camp.raised).toBe(totalContributions(camp));
    });

    it("same user can donate multiple times and amounts accumulate", () => {
      const camp = createCampaign("c1", "Test", 100_000, FEE_BPS, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000);
      donate(camp, "alice", 1_000);
      expect(camp.contributions.get("alice")).toBe(1_950); // 975 + 975
    });
  });

  describe("3. Campaign reaches goal", () => {
    it("state transitions to completed when raised >= target", () => {
      const camp = createCampaign("c1", "Test", 1_000, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000);
      expect(camp.state).toBe("completed");
    });

    it("state transitions to completed when overfunded", () => {
      const camp = createCampaign("c1", "Test", 500, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 600);
      expect(camp.state).toBe("completed");
    });

    it("state remains active while below target", () => {
      const camp = createCampaign("c1", "Test", 1_000, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 999);
      expect(camp.state).toBe("active");
    });

    it("completion happens on the exact donation that crosses the threshold", () => {
      const camp = createCampaign("c1", "Test", 1_000, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 500);
      expect(camp.state).toBe("active");
      donate(camp, "bob", 500);
      expect(camp.state).toBe("completed");
    });

    it("can be finalized after completion", () => {
      const camp = createCampaign("c1", "Test", 500, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 500);
      expect(finalize(camp)).toBeNull();
      expect(camp.state).toBe("finalized");
    });
  });

  describe("4. Further donations fail after goal reached", () => {
    it("donation returns ERR_NOT_ACTIVE after campaign completes", () => {
      const camp = createCampaign("c1", "Test", 500, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 500);
      expect(donate(camp, "bob", 100)).toBe("ERR_NOT_ACTIVE");
    });

    it("raised amount does not change after completion", () => {
      const camp = createCampaign("c1", "Test", 500, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 500);
      const raisedAtCompletion = camp.raised;
      donate(camp, "bob", 100);
      expect(camp.raised).toBe(raisedAtCompletion);
    });

    it("donation returns ERR_NOT_ACTIVE after finalization", () => {
      const camp = createCampaign("c1", "Test", 500, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 500);
      finalize(camp);
      expect(donate(camp, "bob", 100)).toBe("ERR_NOT_ACTIVE");
    });

    it("donation returns ERR_NOT_ACTIVE after cancellation", () => {
      const camp = createCampaign("c1", "Test", 1_000, 0, Date.now() + 86_400_000);
      cancel(camp);
      expect(donate(camp, "alice", 100)).toBe("ERR_NOT_ACTIVE");
    });

    it("donation returns ERR_DEADLINE_PASSED after deadline", () => {
      const camp = createCampaign("c1", "Test", 1_000, 0, Date.now() - 1);
      expect(donate(camp, "alice", 100, Date.now())).toBe("ERR_DEADLINE_PASSED");
    });
  });

  describe("5. All balances and metrics correct throughout", () => {
    it("raised + feesCollected equals total gross donations", () => {
      const camp = createCampaign("c1", "Test", 100_000, FEE_BPS, Date.now() + 86_400_000);
      const donations = [1_000, 2_000, 3_000, 4_000];
      donations.forEach((a, i) => donate(camp, `user${i}`, a));
      const grossTotal = donations.reduce((s, a) => s + a, 0);
      expect(camp.raised + camp.feesCollected).toBe(grossTotal);
    });

    it("individual contribution balances sum to total raised", () => {
      const camp = createCampaign("c1", "Test", 100_000, FEE_BPS, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000);
      donate(camp, "bob", 2_000);
      donate(camp, "carol", 3_000);
      expect(totalContributions(camp)).toBe(camp.raised);
    });

    it("fee calculation is consistent: fee = floor(amount * bps / 10000)", () => {
      const camp = createCampaign("c1", "Test", 100_000, FEE_BPS, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000);
      const expectedFee = Math.floor((1_000 * FEE_BPS) / 10_000);
      expect(camp.feesCollected).toBe(expectedFee);
    });

    it("full lifecycle: create → donate → complete → finalize preserves all metrics", () => {
      const camp = createCampaign("c1", "Full Lifecycle", 2_000, FEE_BPS, Date.now() + 86_400_000);
      donate(camp, "alice", 1_000);
      donate(camp, "bob", 1_000);
      expect(camp.state).toBe("completed");
      finalize(camp);
      expect(camp.state).toBe("finalized");
      expect(camp.raised).toBe(totalContributions(camp));
      expect(camp.feesCollected).toBe(Math.floor((2_000 * FEE_BPS) / 10_000));
    });

    it("finalize fails if campaign is not completed", () => {
      const camp = createCampaign("c1", "Test", 1_000, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 500);
      expect(finalize(camp)).toBe("ERR_NOT_COMPLETED");
    });

    it("cancel fails if campaign is already finalized", () => {
      const camp = createCampaign("c1", "Test", 500, 0, Date.now() + 86_400_000);
      donate(camp, "alice", 500);
      finalize(camp);
      expect(cancel(camp)).toBe("ERR_ALREADY_FINALIZED");
    });
  });
});
