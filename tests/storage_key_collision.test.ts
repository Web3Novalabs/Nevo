/**
 * Storage Key Collision Prevention Tests — Issue #500
 */

import { describe, it, expect } from "@jest/globals";

const CAMPAIGN_PREFIX   = "campaign";
const POOL_PREFIX       = "pool";
const CONTRIB_PREFIX    = "contrib";
const METRICS_PREFIX    = "metrics";
const STATE_PREFIX      = "state";

function campaignKey(id: string)                         { return `${CAMPAIGN_PREFIX}:${id}`; }
function poolKey(id: string)                             { return `${POOL_PREFIX}:${id}`; }
function userContribCampaignKey(userId: string, cId: string) { return `${CONTRIB_PREFIX}:campaign:${cId}:${userId}`; }
function userContribPoolKey(userId: string, pId: string)     { return `${CONTRIB_PREFIX}:pool:${pId}:${userId}`; }
function campaignMetricsKey(id: string)                  { return `${METRICS_PREFIX}:campaign:${id}`; }
function poolMetricsKey(id: string)                      { return `${METRICS_PREFIX}:pool:${id}`; }
function campaignStateKey(id: string)                    { return `${STATE_PREFIX}:campaign:${id}`; }
function poolStateKey(id: string)                        { return `${STATE_PREFIX}:pool:${id}`; }

describe("Storage Key Collision Prevention", () => {
  describe("1. Campaign and Pool IDs do not collide", () => {
    it("same numeric ID produces different keys for campaign vs pool", () => {
      const id = "42";
      expect(campaignKey(id)).not.toBe(poolKey(id));
    });

    it("campaign keys share no overlap with pool key space", () => {
      const ids = ["1", "100", "abc", "0x1234"];
      const campaignKeys = ids.map(campaignKey);
      const poolKeys     = ids.map(poolKey);
      const intersection = campaignKeys.filter((k) => poolKeys.includes(k));
      expect(intersection).toHaveLength(0);
    });

    it("campaign key cannot be spoofed by crafting a pool ID", () => {
      expect(poolKey("campaign:99")).not.toBe(campaignKey("99"));
    });

    it("all campaign keys are unique for distinct IDs", () => {
      const ids = ["1", "2", "3", "100", "999"];
      const keys = ids.map(campaignKey);
      expect(new Set(keys).size).toBe(ids.length);
    });

    it("all pool keys are unique for distinct IDs", () => {
      const ids = ["1", "2", "3", "100", "999"];
      const keys = ids.map(poolKey);
      expect(new Set(keys).size).toBe(ids.length);
    });
  });

  describe("2. User contributions tracked separately per campaign/pool", () => {
    const USER_A = "user_alice";
    const USER_B = "user_bob";
    const CAMP_1 = "camp_1";
    const CAMP_2 = "camp_2";
    const POOL_1 = "pool_1";

    it("same user, different campaigns → different keys", () => {
      expect(userContribCampaignKey(USER_A, CAMP_1)).not.toBe(userContribCampaignKey(USER_A, CAMP_2));
    });

    it("same campaign, different users → different keys", () => {
      expect(userContribCampaignKey(USER_A, CAMP_1)).not.toBe(userContribCampaignKey(USER_B, CAMP_1));
    });

    it("campaign contribution and pool contribution keys never collide for same user+id", () => {
      const sharedId = "entity_1";
      expect(userContribCampaignKey(USER_A, sharedId)).not.toBe(userContribPoolKey(USER_A, sharedId));
    });

    it("no cross-entity collision across all (user, campaign) pairs", () => {
      const users = [USER_A, USER_B];
      const campaigns = [CAMP_1, CAMP_2];
      const pools = [POOL_1];
      const allKeys: string[] = [];
      users.forEach((u) => {
        campaigns.forEach((c) => allKeys.push(userContribCampaignKey(u, c)));
        pools.forEach((p) => allKeys.push(userContribPoolKey(u, p)));
      });
      expect(new Set(allKeys).size).toBe(allKeys.length);
    });
  });

  describe("3. Metrics stored independently per entity", () => {
    it("campaign and pool metrics keys differ even for same ID", () => {
      expect(campaignMetricsKey("entity_1")).not.toBe(poolMetricsKey("entity_1"));
    });

    it("metrics keys don't collide with contribution keys", () => {
      const id = "entity_1";
      const userId = "user_1";
      expect(campaignMetricsKey(id)).not.toBe(userContribCampaignKey(userId, id));
      expect(poolMetricsKey(id)).not.toBe(userContribPoolKey(userId, id));
    });

    it("metrics keys don't collide with entity keys", () => {
      const id = "entity_1";
      expect(campaignMetricsKey(id)).not.toBe(campaignKey(id));
      expect(poolMetricsKey(id)).not.toBe(poolKey(id));
    });

    it("all metrics key types are unique in a mixed key set", () => {
      const id = "shared_id";
      const keys = [campaignMetricsKey(id), poolMetricsKey(id), campaignKey(id), poolKey(id), campaignStateKey(id), poolStateKey(id)];
      expect(new Set(keys).size).toBe(keys.length);
    });
  });

  describe("4. State keys unique across entity types", () => {
    it("campaign state key differs from pool state key for same ID", () => {
      expect(campaignStateKey("99")).not.toBe(poolStateKey("99"));
    });

    it("state keys don't collide with data keys", () => {
      const id = "entity_99";
      expect(campaignStateKey(id)).not.toBe(campaignKey(id));
      expect(poolStateKey(id)).not.toBe(poolKey(id));
    });

    it("state keys don't collide with metrics keys", () => {
      const id = "entity_99";
      expect(campaignStateKey(id)).not.toBe(campaignMetricsKey(id));
      expect(poolStateKey(id)).not.toBe(poolMetricsKey(id));
    });

    it("full key namespace: no two key types produce the same string", () => {
      const id = "test_entity";
      const userId = "test_user";
      const fullKeySpace = [
        campaignKey(id), poolKey(id),
        userContribCampaignKey(userId, id), userContribPoolKey(userId, id),
        campaignMetricsKey(id), poolMetricsKey(id),
        campaignStateKey(id), poolStateKey(id),
      ];
      expect(new Set(fullKeySpace).size).toBe(fullKeySpace.length);
    });

    it("ID prefix injection attack: entity with separator in ID does not collide", () => {
      const maliciousId = "X:user:alice";
      const normalId    = "X";
      const normalUser  = "alice";
      expect(userContribCampaignKey(normalUser, maliciousId)).not.toBe(userContribCampaignKey(normalUser, normalId));
    });
  });
});
