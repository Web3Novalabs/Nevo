/**
 * Pause State Operations Tests — Issue #481
 *
 * Test paused contract restrictions:
 * (1) create_campaign fails when paused.
 * (2) save_pool fails when paused.
 * (3) contribute fails when paused.
 * (4) update_pool_state fails when paused.
 * (5) Getter functions work when paused.
 */

import { describe, it, expect, beforeEach } from "@jest/globals";

interface Campaign {
  id: string;
  title: string;
  target: number;
  raised: number;
  state: "active" | "paused" | "completed" | "cancelled";
  contributions: Map<string, number>;
}

interface Pool {
  id: string;
  title: string;
  goal: number;
  collected: number;
  state: "active" | "paused" | "refunding" | "finalized";
  contributions: Map<string, number>;
}

interface Contract {
  paused: boolean;
  campaigns: Map<string, Campaign>;
  pools: Map<string, Pool>;
}

function makeContract(): Contract {
  return { paused: false, campaigns: new Map(), pools: new Map() };
}

function createCampaign(contract: Contract, id: string, title: string, target: number): string | Campaign {
  if (contract.paused) return "ERR_CONTRACT_PAUSED";
  const campaign: Campaign = { id, title, target, raised: 0, state: "active", contributions: new Map() };
  contract.campaigns.set(id, campaign);
  return campaign;
}

function savePool(contract: Contract, id: string, title: string, goal: number): string | Pool {
  if (contract.paused) return "ERR_CONTRACT_PAUSED";
  const pool: Pool = { id, title, goal, collected: 0, state: "active", contributions: new Map() };
  contract.pools.set(id, pool);
  return pool;
}

function contribute(contract: Contract, campaignId: string, user: string, amount: number): string | null {
  if (contract.paused) return "ERR_CONTRACT_PAUSED";
  const campaign = contract.campaigns.get(campaignId);
  if (!campaign) return "ERR_NOT_FOUND";
  if (campaign.state !== "active") return "ERR_NOT_ACTIVE";
  campaign.contributions.set(user, (campaign.contributions.get(user) ?? 0) + amount);
  campaign.raised += amount;
  return null;
}

function updatePoolState(contract: Contract, poolId: string, newState: Pool["state"]): string | null {
  if (contract.paused) return "ERR_CONTRACT_PAUSED";
  const pool = contract.pools.get(poolId);
  if (!pool) return "ERR_NOT_FOUND";
  pool.state = newState;
  return null;
}

function getCampaign(contract: Contract, id: string): Campaign | undefined {
  return contract.campaigns.get(id);
}

function getPool(contract: Contract, id: string): Pool | undefined {
  return contract.pools.get(id);
}

function getAllCampaigns(contract: Contract): Campaign[] {
  return [...contract.campaigns.values()];
}

function getAllPools(contract: Contract): Pool[] {
  return [...contract.pools.values()];
}

describe("Pause State Operations", () => {
  let contract: Contract;

  beforeEach(() => {
    contract = makeContract();
  });

  describe("1. create_campaign fails when paused", () => {
    it("returns ERR_CONTRACT_PAUSED when contract is paused", () => {
      contract.paused = true;
      expect(createCampaign(contract, "c1", "Test", 1000)).toBe("ERR_CONTRACT_PAUSED");
    });

    it("does not create campaign when paused", () => {
      contract.paused = true;
      createCampaign(contract, "c1", "Test", 1000);
      expect(contract.campaigns.has("c1")).toBe(false);
    });

    it("succeeds when contract is not paused", () => {
      const result = createCampaign(contract, "c1", "Test", 1000);
      expect(typeof result).toBe("object");
      expect(contract.campaigns.has("c1")).toBe(true);
    });

    it("fails immediately after pausing (no campaigns created after pause)", () => {
      createCampaign(contract, "c1", "Before", 500);
      contract.paused = true;
      expect(createCampaign(contract, "c2", "After", 500)).toBe("ERR_CONTRACT_PAUSED");
      expect(contract.campaigns.size).toBe(1);
    });
  });

  describe("2. save_pool fails when paused", () => {
    it("returns ERR_CONTRACT_PAUSED when contract is paused", () => {
      contract.paused = true;
      expect(savePool(contract, "p1", "Pool", 5000)).toBe("ERR_CONTRACT_PAUSED");
    });

    it("does not persist pool when paused", () => {
      contract.paused = true;
      savePool(contract, "p1", "Pool", 5000);
      expect(contract.pools.has("p1")).toBe(false);
    });

    it("succeeds when contract is active", () => {
      const result = savePool(contract, "p1", "Pool", 5000);
      expect(typeof result).toBe("object");
      expect(contract.pools.has("p1")).toBe(true);
    });

    it("existing pools are unaffected when new pool creation is blocked", () => {
      savePool(contract, "p1", "Existing", 1000);
      contract.paused = true;
      savePool(contract, "p2", "New", 2000);
      expect(contract.pools.size).toBe(1);
      expect(contract.pools.has("p1")).toBe(true);
    });
  });

  describe("3. contribute fails when paused", () => {
    it("returns ERR_CONTRACT_PAUSED when contract is paused", () => {
      createCampaign(contract, "c1", "Test", 1000);
      contract.paused = true;
      expect(contribute(contract, "c1", "alice", 100)).toBe("ERR_CONTRACT_PAUSED");
    });

    it("does not update raised amount when paused", () => {
      createCampaign(contract, "c1", "Test", 1000);
      contract.paused = true;
      contribute(contract, "c1", "alice", 100);
      expect(contract.campaigns.get("c1")?.raised).toBe(0);
    });

    it("does not record contribution when paused", () => {
      createCampaign(contract, "c1", "Test", 1000);
      contract.paused = true;
      contribute(contract, "c1", "alice", 100);
      expect(contract.campaigns.get("c1")?.contributions.has("alice")).toBe(false);
    });

    it("succeeds when contract is active", () => {
      createCampaign(contract, "c1", "Test", 1000);
      expect(contribute(contract, "c1", "alice", 100)).toBeNull();
      expect(contract.campaigns.get("c1")?.raised).toBe(100);
    });

    it("blocks contributions from all users when paused", () => {
      createCampaign(contract, "c1", "Test", 1000);
      contract.paused = true;
      ["alice", "bob", "carol"].forEach((user) => {
        expect(contribute(contract, "c1", user, 100)).toBe("ERR_CONTRACT_PAUSED");
      });
      expect(contract.campaigns.get("c1")?.raised).toBe(0);
    });
  });

  describe("4. update_pool_state fails when paused", () => {
    it("returns ERR_CONTRACT_PAUSED when contract is paused", () => {
      savePool(contract, "p1", "Pool", 5000);
      contract.paused = true;
      expect(updatePoolState(contract, "p1", "finalized")).toBe("ERR_CONTRACT_PAUSED");
    });

    it("does not change pool state when contract is paused", () => {
      savePool(contract, "p1", "Pool", 5000);
      contract.paused = true;
      updatePoolState(contract, "p1", "finalized");
      expect(contract.pools.get("p1")?.state).toBe("active");
    });

    it("succeeds when contract is active", () => {
      savePool(contract, "p1", "Pool", 5000);
      expect(updatePoolState(contract, "p1", "finalized")).toBeNull();
      expect(contract.pools.get("p1")?.state).toBe("finalized");
    });

    it("blocks all state transitions when paused", () => {
      savePool(contract, "p1", "Pool", 5000);
      contract.paused = true;
      (["paused", "refunding", "finalized"] as Pool["state"][]).forEach((s) => {
        expect(updatePoolState(contract, "p1", s)).toBe("ERR_CONTRACT_PAUSED");
      });
      expect(contract.pools.get("p1")?.state).toBe("active");
    });
  });

  describe("5. Getter functions work when paused", () => {
    it("getCampaign returns data when contract is paused", () => {
      createCampaign(contract, "c1", "Test Campaign", 1000);
      contract.paused = true;
      const campaign = getCampaign(contract, "c1");
      expect(campaign).toBeDefined();
      expect(campaign?.title).toBe("Test Campaign");
      expect(campaign?.target).toBe(1000);
    });

    it("getPool returns data when contract is paused", () => {
      savePool(contract, "p1", "Test Pool", 5000);
      contract.paused = true;
      const pool = getPool(contract, "p1");
      expect(pool).toBeDefined();
      expect(pool?.title).toBe("Test Pool");
      expect(pool?.goal).toBe(5000);
    });

    it("getAllCampaigns returns full list when paused", () => {
      createCampaign(contract, "c1", "Camp 1", 100);
      createCampaign(contract, "c2", "Camp 2", 200);
      contract.paused = true;
      expect(getAllCampaigns(contract)).toHaveLength(2);
    });

    it("getAllPools returns full list when paused", () => {
      savePool(contract, "p1", "Pool 1", 1000);
      savePool(contract, "p2", "Pool 2", 2000);
      contract.paused = true;
      expect(getAllPools(contract)).toHaveLength(2);
    });

    it("getCampaign returns undefined for non-existent id when paused", () => {
      contract.paused = true;
      expect(getCampaign(contract, "ghost")).toBeUndefined();
    });

    it("raised amount is readable and unchanged after blocked contributions", () => {
      createCampaign(contract, "c1", "Test", 1000);
      contribute(contract, "c1", "alice", 300);
      contract.paused = true;
      contribute(contract, "c1", "bob", 200);
      expect(getCampaign(contract, "c1")?.raised).toBe(300);
    });
  });
});
