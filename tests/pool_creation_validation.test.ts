/**
 * Pool Creation Parameter Validation Tests — Issue #484
 *
 * Test save_pool parameter validation:
 * (1) Empty name fails with InvalidPoolName.
 * (2) Zero target_amount fails with InvalidPoolTarget.
 * (3) Past deadline fails with InvalidPoolDeadline.
 * (4) Valid parameters succeed.
 * (5) Pool ID generation works.
 */

import { describe, it, expect, beforeEach } from "@jest/globals";

export interface Pool {
  id: string;
  name: string;
  target_amount: number;
  deadline: number;
  state: "active" | "paused" | "refunding" | "finalized" | "closed";
}

export interface Contract {
  pools: Map<string, Pool>;
  nextPoolId: number;
}

export function makeContract(): Contract {
  return {
    pools: new Map(),
    nextPoolId: 1,
  };
}

/**
 * Saves a pool to the contract, validating input parameters.
 *
 * @param contract The contract state object.
 * @param name The pool's name.
 * @param target_amount The funding target amount.
 * @param deadline The pool's deadline timestamp (ms).
 * @param now The current timestamp to validate against (defaults to Date.now()).
 * @returns The created Pool object or an error string.
 */
export function save_pool(
  contract: Contract,
  name: string,
  target_amount: number,
  deadline: number,
  now: number = Date.now()
): Pool | string {
  // 1. Empty name check (including whitespace-only names)
  if (!name || name.trim() === "") {
    return "InvalidPoolName";
  }

  // 2. Zero or negative target amount check
  if (target_amount <= 0) {
    return "InvalidPoolTarget";
  }

  // 3. Past or present deadline check (deadline must be in the future)
  if (deadline <= now) {
    return "InvalidPoolDeadline";
  }

  // 5. Pool ID generation
  const id = `pool_${contract.nextPoolId++}`;

  // 4. Valid parameters succeed
  const pool: Pool = {
    id,
    name,
    target_amount,
    deadline,
    state: "active",
  };

  contract.pools.set(id, pool);
  return pool;
}

describe("Pool Creation Parameter Validation (Issue #484)", () => {
  let contract: Contract;
  let currentTimestamp: number;

  beforeEach(() => {
    contract = makeContract();
    currentTimestamp = 1717196400000; // Simulated current time (e.g. 2024-06-01T00:00:00Z)
  });

  describe("1. Empty name validation", () => {
    it("fails with InvalidPoolName when name is empty string", () => {
      const result = save_pool(contract, "", 5000, currentTimestamp + 10000, currentTimestamp);
      expect(result).toBe("InvalidPoolName");
    });

    it("fails with InvalidPoolName when name is only whitespace", () => {
      const result = save_pool(contract, "   ", 5000, currentTimestamp + 10000, currentTimestamp);
      expect(result).toBe("InvalidPoolName");
    });

    it("does not store any pool in the contract when name is invalid", () => {
      save_pool(contract, "", 5000, currentTimestamp + 10000, currentTimestamp);
      expect(contract.pools.size).toBe(0);
    });
  });

  describe("2. Zero target amount validation", () => {
    it("fails with InvalidPoolTarget when target_amount is exactly zero", () => {
      const result = save_pool(contract, "Scholarship Pool", 0, currentTimestamp + 10000, currentTimestamp);
      expect(result).toBe("InvalidPoolTarget");
    });

    it("fails with InvalidPoolTarget when target_amount is negative", () => {
      const result = save_pool(contract, "Scholarship Pool", -100, currentTimestamp + 10000, currentTimestamp);
      expect(result).toBe("InvalidPoolTarget");
    });

    it("does not store any pool in the contract when target_amount is invalid", () => {
      save_pool(contract, "Scholarship Pool", 0, currentTimestamp + 10000, currentTimestamp);
      expect(contract.pools.size).toBe(0);
    });
  });

  describe("3. Past deadline validation", () => {
    it("fails with InvalidPoolDeadline when deadline is in the past", () => {
      const result = save_pool(contract, "Scholarship Pool", 5000, currentTimestamp - 1000, currentTimestamp);
      expect(result).toBe("InvalidPoolDeadline");
    });

    it("fails with InvalidPoolDeadline when deadline is exactly equal to current time", () => {
      const result = save_pool(contract, "Scholarship Pool", 5000, currentTimestamp, currentTimestamp);
      expect(result).toBe("InvalidPoolDeadline");
    });

    it("does not store any pool in the contract when deadline is invalid", () => {
      save_pool(contract, "Scholarship Pool", 5000, currentTimestamp - 1000, currentTimestamp);
      expect(contract.pools.size).toBe(0);
    });
  });

  describe("4. Valid parameters validation", () => {
    it("succeeds and returns the pool object when all parameters are valid", () => {
      const result = save_pool(contract, "Scholarship Pool", 5000, currentTimestamp + 10000, currentTimestamp);
      expect(typeof result).toBe("object");
      
      const pool = result as Pool;
      expect(pool.name).toBe("Scholarship Pool");
      expect(pool.target_amount).toBe(5000);
      expect(pool.deadline).toBe(currentTimestamp + 10000);
      expect(pool.state).toBe("active");
    });

    it("persists the created pool inside the contract storage", () => {
      const result = save_pool(contract, "Scholarship Pool", 5000, currentTimestamp + 10000, currentTimestamp);
      const pool = result as Pool;
      
      expect(contract.pools.has(pool.id)).toBe(true);
      expect(contract.pools.get(pool.id)).toEqual(pool);
    });
  });

  describe("5. Pool ID generation", () => {
    it("generates sequential, unique pool IDs", () => {
      const pool1 = save_pool(contract, "Pool One", 1000, currentTimestamp + 10000, currentTimestamp) as Pool;
      const pool2 = save_pool(contract, "Pool Two", 2000, currentTimestamp + 10000, currentTimestamp) as Pool;
      const pool3 = save_pool(contract, "Pool Three", 3000, currentTimestamp + 10000, currentTimestamp) as Pool;

      expect(pool1.id).toBe("pool_1");
      expect(pool2.id).toBe("pool_2");
      expect(pool3.id).toBe("pool_3");
    });

    it("increments the nextPoolId tracker in the contract state", () => {
      expect(contract.nextPoolId).toBe(1);
      save_pool(contract, "Pool One", 1000, currentTimestamp + 10000, currentTimestamp);
      expect(contract.nextPoolId).toBe(2);
      save_pool(contract, "Pool Two", 2000, currentTimestamp + 10000, currentTimestamp);
      expect(contract.nextPoolId).toBe(3);
    });
  });
});
