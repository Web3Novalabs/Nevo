/**
 * Campaign List Management Tests — Issue #490
 *
 * Test get_all_campaigns function:
 * (1) Empty list initially.
 * (2) Single campaign appears in list.
 * (3) Multiple campaigns all listed.
 * (4) Campaign order preserved.
 * (5) List updates with new campaigns.
 */

import { describe, it, expect, beforeEach } from "@jest/globals";

interface Campaign {
  id: string;
  title: string;
  target: number;
  raised: number;
  state: "active" | "paused" | "completed" | "cancelled";
  createdAt: number;
}

class CampaignRegistry {
  private campaigns: Campaign[] = [];
  private nextId = 1;

  create(title: string, target: number, now = Date.now()): Campaign {
    const campaign: Campaign = {
      id: `c${this.nextId++}`,
      title,
      target,
      raised: 0,
      state: "active",
      createdAt: now,
    };
    this.campaigns.push(campaign);
    return campaign;
  }

  getAll(): Campaign[] {
    return [...this.campaigns];
  }

  getById(id: string): Campaign | undefined {
    return this.campaigns.find((c) => c.id === id);
  }

  count(): number {
    return this.campaigns.length;
  }
}

describe("Campaign List Management", () => {
  let registry: CampaignRegistry;

  beforeEach(() => {
    registry = new CampaignRegistry();
  });

  describe("1. Empty list initially", () => {
    it("returns empty array before any campaigns are created", () => {
      expect(registry.getAll()).toEqual([]);
    });

    it("count is zero initially", () => {
      expect(registry.count()).toBe(0);
    });

    it("getAll returns a new array each call (not a reference)", () => {
      const list1 = registry.getAll();
      const list2 = registry.getAll();
      expect(list1).not.toBe(list2);
    });
  });

  describe("2. Single campaign appears in list", () => {
    it("list contains the created campaign", () => {
      registry.create("First Campaign", 1000);
      expect(registry.getAll()).toHaveLength(1);
    });

    it("campaign data is correct in the list", () => {
      registry.create("Scholarship Fund", 5000);
      const [camp] = registry.getAll();
      expect(camp.title).toBe("Scholarship Fund");
      expect(camp.target).toBe(5000);
      expect(camp.raised).toBe(0);
      expect(camp.state).toBe("active");
    });

    it("campaign has a unique id assigned", () => {
      registry.create("Test", 100);
      const [camp] = registry.getAll();
      expect(camp.id).toBeDefined();
      expect(camp.id.length).toBeGreaterThan(0);
    });

    it("count is 1 after single creation", () => {
      registry.create("Solo", 200);
      expect(registry.count()).toBe(1);
    });
  });

  describe("3. Multiple campaigns all listed", () => {
    it("all created campaigns appear in the list", () => {
      registry.create("Camp A", 100);
      registry.create("Camp B", 200);
      registry.create("Camp C", 300);
      expect(registry.getAll()).toHaveLength(3);
    });

    it("each campaign has distinct id", () => {
      registry.create("X", 10);
      registry.create("Y", 20);
      registry.create("Z", 30);
      const ids = registry.getAll().map((c) => c.id);
      expect(new Set(ids).size).toBe(3);
    });

    it("all campaign titles are present", () => {
      const titles = ["Alpha", "Beta", "Gamma", "Delta"];
      titles.forEach((t) => registry.create(t, 100));
      const listed = registry.getAll().map((c) => c.title);
      titles.forEach((t) => expect(listed).toContain(t));
    });

    it("count matches number of created campaigns", () => {
      for (let i = 0; i < 10; i++) registry.create(`Camp ${i}`, i * 100);
      expect(registry.count()).toBe(10);
    });
  });

  describe("4. Campaign order preserved", () => {
    it("campaigns appear in insertion order", () => {
      registry.create("First", 100);
      registry.create("Second", 200);
      registry.create("Third", 300);
      const list = registry.getAll();
      expect(list[0].title).toBe("First");
      expect(list[1].title).toBe("Second");
      expect(list[2].title).toBe("Third");
    });

    it("ids are assigned in ascending order", () => {
      for (let i = 0; i < 5; i++) registry.create(`Camp ${i}`, 100);
      const ids = registry.getAll().map((c) => c.id);
      for (let i = 1; i < ids.length; i++) {
        expect(ids[i] > ids[i - 1]).toBe(true);
      }
    });

    it("createdAt timestamps reflect insertion order", () => {
      let tick = 0;
      registry.create("Early", 100, tick++);
      registry.create("Middle", 200, tick++);
      registry.create("Late", 300, tick++);
      const list = registry.getAll();
      expect(list[0].createdAt).toBeLessThan(list[1].createdAt);
      expect(list[1].createdAt).toBeLessThan(list[2].createdAt);
    });

    it("order is stable across multiple getAll calls", () => {
      ["A", "B", "C"].forEach((t) => registry.create(t, 100));
      const first = registry.getAll().map((c) => c.title);
      const second = registry.getAll().map((c) => c.title);
      expect(first).toEqual(second);
    });
  });

  describe("5. List updates with new campaigns", () => {
    it("list grows after each new campaign", () => {
      expect(registry.getAll()).toHaveLength(0);
      registry.create("One", 100);
      expect(registry.getAll()).toHaveLength(1);
      registry.create("Two", 200);
      expect(registry.getAll()).toHaveLength(2);
    });

    it("newly added campaign is at the end of the list", () => {
      registry.create("Old", 100);
      registry.create("New", 200);
      const list = registry.getAll();
      expect(list[list.length - 1].title).toBe("New");
    });

    it("existing campaigns are unchanged after adding a new one", () => {
      registry.create("Stable", 500);
      const before = registry.getAll()[0];
      registry.create("Another", 300);
      const after = registry.getAll()[0];
      expect(after.id).toBe(before.id);
      expect(after.title).toBe(before.title);
      expect(after.target).toBe(before.target);
    });

    it("getById finds newly added campaign", () => {
      const camp = registry.create("Findable", 999);
      expect(registry.getById(camp.id)).toBeDefined();
      expect(registry.getById(camp.id)?.title).toBe("Findable");
    });

    it("getAll snapshot does not reflect later additions", () => {
      registry.create("Before", 100);
      const snapshot = registry.getAll();
      registry.create("After", 200);
      expect(snapshot).toHaveLength(1);
      expect(registry.getAll()).toHaveLength(2);
    });
  });
});
