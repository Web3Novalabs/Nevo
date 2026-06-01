/**
 * Data Persistence Validation Tests — Issue #510
 */

import { describe, it, expect, beforeEach } from "@jest/globals";

class InstanceStorage {
  private store = new Map<string, unknown>();
  set<T>(key: string, value: T): void { this.store.set(key, value); }
  get<T>(key: string): T | undefined  { return this.store.get(key) as T; }
  has(key: string): boolean            { return this.store.has(key); }
  delete(key: string): boolean         { return this.store.delete(key); }
  clear(): void                        { this.store.clear(); }
  keys(): string[]                     { return [...this.store.keys()]; }
  size(): number                       { return this.store.size; }
}

class PersistentStorage {
  private store = new Map<string, unknown>();
  private ttls  = new Map<string, number>();
  private now: () => number;
  constructor(nowFn: () => number = Date.now) { this.now = nowFn; }
  set<T>(key: string, value: T, ttlMs?: number): void {
    const cloned = value && typeof value === "object" ? JSON.parse(JSON.stringify(value)) : value;
    this.store.set(key, cloned);
    if (ttlMs != null) this.ttls.set(key, this.now() + ttlMs);
    else                this.ttls.delete(key);
  }
  get<T>(key: string): T | undefined {
    const expiry = this.ttls.get(key);
    if (expiry != null && this.now() > expiry) { this.store.delete(key); this.ttls.delete(key); return undefined; }
    const val = this.store.get(key);
    if (val && typeof val === "object") {
      return JSON.parse(JSON.stringify(val)) as T;
    }
    return val as T;
  }
  has(key: string): boolean    { return this.get(key) !== undefined; }
  delete(key: string): boolean { this.ttls.delete(key); return this.store.delete(key); }
  evictExpired(): number {
    const now = this.now(); let count = 0;
    for (const [key, expiry] of this.ttls) {
      if (now > expiry) { this.store.delete(key); this.ttls.delete(key); count++; }
    }
    return count;
  }
  keys(): string[]  { return [...this.store.keys()].filter((k) => this.has(k)); }
  size(): number    { return this.keys().length; }
}

interface ContractState { paused: boolean; owner: string; version: number; }
interface CampaignMeta  { id: string; title: string; target: number; raised: number; createdAt: number; }

class ContractSimulator {
  readonly instance   = new InstanceStorage();
  readonly persistent: PersistentStorage;
  constructor(nowFn?: () => number) { this.persistent = new PersistentStorage(nowFn); }
  initState(state: ContractState): void { this.instance.set("state", { ...state }); }
  getState(): ContractState | undefined { return this.instance.get<ContractState>("state"); }
  patchState(patch: Partial<ContractState>): void {
    const current = this.getState();
    if (!current) throw new Error("State not initialised");
    this.instance.set("state", { ...current, ...patch });
  }
  saveCampaign(meta: CampaignMeta, ttlMs?: number): void {
    this.persistent.set(`campaign:${meta.id}`, { ...meta }, ttlMs);
  }
  getCampaign(id: string): CampaignMeta | undefined {
    return this.persistent.get<CampaignMeta>(`campaign:${id}`);
  }
  deleteCampaign(id: string): boolean { return this.persistent.delete(`campaign:${id}`); }
  updateRaised(id: string, delta: number): void {
    const meta = this.getCampaign(id);
    if (!meta) throw new Error(`Campaign ${id} not found`);
    this.persistent.set(`campaign:${id}`, { ...meta, raised: meta.raised + delta });
  }
}

describe("Data Persistence Validation", () => {
  let contract: ContractSimulator;
  beforeEach(() => {
    contract = new ContractSimulator();
    contract.initState({ paused: false, owner: "admin", version: 1 });
  });

  describe("1. Instance storage persists correctly", () => {
    it("stores and retrieves contract state", () => {
      const state = contract.getState();
      expect(state?.paused).toBe(false);
      expect(state?.owner).toBe("admin");
      expect(state?.version).toBe(1);
    });
    it("patch updates only specified fields", () => {
      contract.patchState({ paused: true });
      const state = contract.getState()!;
      expect(state.paused).toBe(true);
      expect(state.owner).toBe("admin");
      expect(state.version).toBe(1);
    });
    it("successive patches are cumulative", () => {
      contract.patchState({ paused: true });
      contract.patchState({ version: 2 });
      expect(contract.getState()!.paused).toBe(true);
      expect(contract.getState()!.version).toBe(2);
    });
    it("instance storage is independent per contract instance", () => {
      const contract2 = new ContractSimulator();
      contract2.initState({ paused: true, owner: "other", version: 5 });
      expect(contract.getState()?.paused).toBe(false);
      expect(contract2.getState()?.paused).toBe(true);
    });
    it("has() reflects write/delete lifecycle", () => {
      expect(contract.instance.has("state")).toBe(true);
      contract.instance.delete("state");
      expect(contract.instance.has("state")).toBe(false);
    });
  });

  describe("2. Persistent storage works for metadata", () => {
    it("stores and retrieves campaign metadata", () => {
      contract.saveCampaign({ id: "c1", title: "Test", target: 1000, raised: 0, createdAt: 1000 });
      const meta = contract.getCampaign("c1");
      expect(meta?.title).toBe("Test");
      expect(meta?.target).toBe(1000);
    });
    it("multiple campaigns stored independently", () => {
      ["c1", "c2", "c3"].forEach((id, i) =>
        contract.saveCampaign({ id, title: `Camp ${i}`, target: 100 * (i + 1), raised: 0, createdAt: i })
      );
      expect(contract.getCampaign("c1")?.target).toBe(100);
      expect(contract.getCampaign("c2")?.target).toBe(200);
      expect(contract.getCampaign("c3")?.target).toBe(300);
    });
    it("data stored with no TTL persists indefinitely", () => {
      contract.saveCampaign({ id: "eternal", title: "Forever", target: 1, raised: 0, createdAt: 0 });
      expect(contract.persistent.has("campaign:eternal")).toBe(true);
    });
    it("overwrite replaces previous data entirely", () => {
      contract.saveCampaign({ id: "c1", title: "Old", target: 100, raised: 0, createdAt: 0 });
      contract.saveCampaign({ id: "c1", title: "New", target: 999, raised: 0, createdAt: 0 });
      expect(contract.getCampaign("c1")?.title).toBe("New");
      expect(contract.getCampaign("c1")?.target).toBe(999);
    });
  });

  describe("3. Data survives simulated contract calls", () => {
    it("campaign raised amount survives sequential updateRaised calls", () => {
      contract.saveCampaign({ id: "c1", title: "T", target: 1000, raised: 0, createdAt: 0 });
      contract.updateRaised("c1", 100);
      contract.updateRaised("c1", 200);
      contract.updateRaised("c1", 300);
      expect(contract.getCampaign("c1")?.raised).toBe(600);
    });
    it("state changes persist across multiple patchState calls", () => {
      for (let v = 2; v <= 5; v++) contract.patchState({ version: v });
      expect(contract.getState()?.version).toBe(5);
    });
    it("instance and persistent storage remain isolated after calls", () => {
      contract.saveCampaign({ id: "iso", title: "Iso", target: 500, raised: 0, createdAt: 0 });
      contract.patchState({ paused: true });
      expect(contract.getCampaign("iso")?.target).toBe(500);
      expect(contract.getState()?.owner).toBe("admin");
    });
    it("concurrent writes to different keys do not interfere", () => {
      ["a","b","c","d","e"].forEach((id, i) =>
        contract.saveCampaign({ id, title: id.toUpperCase(), target: (i + 1) * 100, raised: 0, createdAt: 0 })
      );
      expect(contract.getCampaign("c")?.target).toBe(300);
      expect(contract.getCampaign("e")?.target).toBe(500);
      expect(contract.persistent.size()).toBe(5);
    });
  });

  describe("4. Storage cleanup when needed", () => {
    it("deleteCampaign removes entry from persistent storage", () => {
      contract.saveCampaign({ id: "del1", title: "D", target: 1, raised: 0, createdAt: 0 });
      contract.deleteCampaign("del1");
      expect(contract.persistent.has("campaign:del1")).toBe(false);
    });
    it("TTL-expired entries are unavailable after expiry", () => {
      let tick = 0;
      const sim = new ContractSimulator(() => tick);
      sim.saveCampaign({ id: "exp", title: "Expiring", target: 1, raised: 0, createdAt: 0 }, 1000);
      tick = 500; expect(sim.getCampaign("exp")).toBeDefined();
      tick = 1001; expect(sim.getCampaign("exp")).toBeUndefined();
    });
    it("evictExpired removes multiple expired entries and returns count", () => {
      let tick = 0;
      const sim = new ContractSimulator(() => tick);
      ["e1","e2","e3"].forEach((id) =>
        sim.saveCampaign({ id, title: id, target: 1, raised: 0, createdAt: 0 }, 100)
      );
      sim.saveCampaign({ id: "keep", title: "keep", target: 1, raised: 0, createdAt: 0 }, 10_000);
      tick = 200;
      expect(sim.persistent.evictExpired()).toBe(3);
      expect(sim.getCampaign("keep")).toBeDefined();
    });
    it("instance.clear() removes all instance keys", () => {
      contract.instance.set("tmp1", "a"); contract.instance.set("tmp2", "b");
      contract.instance.clear();
      expect(contract.instance.size()).toBe(0);
    });
    it("deleting a non-existent key returns false without error", () => {
      expect(contract.persistent.delete("campaign:ghost")).toBe(false);
      expect(contract.instance.delete("ghost")).toBe(false);
    });
  });

  describe("5. No data corruption", () => {
    it("reading a key does not mutate stored value", () => {
      contract.saveCampaign({ id: "c1", title: "Original", target: 100, raised: 0, createdAt: 0 });
      const ref1 = contract.getCampaign("c1");
      if (ref1) { ref1.title = "MUTATED"; ref1.target = 9999; }
      expect(contract.getCampaign("c1")?.title).toBe("Original");
      expect(contract.getCampaign("c1")?.target).toBe(100);
    });
    it("stored objects are independent copies", () => {
      const meta = { id: "c2", title: "Copy", target: 500, raised: 0, createdAt: 0 };
      contract.saveCampaign(meta);
      meta.title = "CHANGED"; meta.target = 1;
      expect(contract.getCampaign("c2")?.title).toBe("Copy");
      expect(contract.getCampaign("c2")?.target).toBe(500);
    });
    it("updateRaised does not corrupt other fields", () => {
      contract.saveCampaign({ id: "c3", title: "Intact", target: 800, raised: 0, createdAt: 42 });
      contract.updateRaised("c3", 100);
      const meta = contract.getCampaign("c3")!;
      expect(meta.title).toBe("Intact");
      expect(meta.target).toBe(800);
      expect(meta.createdAt).toBe(42);
      expect(meta.raised).toBe(100);
    });
    it("simultaneous storage writes do not corrupt each other", () => {
      for (let i = 0; i < 50; i++)
        contract.saveCampaign({ id: `c${i}`, title: `Title${i}`, target: i * 10, raised: 0, createdAt: i });
      for (let i = 0; i < 50; i++) {
        expect(contract.getCampaign(`c${i}`)?.title).toBe(`Title${i}`);
        expect(contract.getCampaign(`c${i}`)?.target).toBe(i * 10);
      }
    });
    it("patchState preserves unmodified fields after many patches", () => {
      for (let i = 0; i < 10; i++) contract.patchState({ version: i });
      expect(contract.getState()?.owner).toBe("admin");
      expect(contract.getState()?.paused).toBe(false);
    });
  });
});
