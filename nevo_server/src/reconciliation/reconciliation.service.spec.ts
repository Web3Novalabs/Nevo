import { ConfigService } from '@nestjs/config';
import { SchedulerRegistry } from '@nestjs/schedule';
import { AdvisoryLockService } from '../common/advisory-lock.service';
import { ContractService } from '../contract/contract.service';
import { Pool, PoolStatus } from '../pools/pool.entity';
import { ReconciliationService } from './reconciliation.service';

const I128_MAX = 170141183460469231731687303715884105727n;

const makePool = (n: number, raised: string): Pool =>
  ({
    id: `00000000-0000-0000-0000-${String(n).padStart(12, '0')}`,
    contractPoolId: String(n),
    status: PoolStatus.Active,
    raised,
  }) as Pool;

describe('ReconciliationService', () => {
  let pools: Pool[];
  let update: jest.Mock;
  let contract: { fetchTotalRaisedOnChain: jest.Mock };
  let lock: { runExclusive: jest.Mock };
  let scheduler: {
    addInterval: jest.Mock;
    deleteInterval: jest.Mock;
    doesExist: jest.Mock;
  };
  let env: Record<string, string>;
  let batchRequests: number[];
  let service: ReconciliationService;

  const buildRepo = () => ({
    update,
    createQueryBuilder: jest.fn(() => {
      let afterId: string | null = null;
      let take = 0;
      const qb = {
        where: () => qb,
        orderBy: () => qb,
        take: (n: number) => {
          take = n;
          return qb;
        },
        andWhere: (_sql: string, params: { afterId: string }) => {
          afterId = params.afterId;
          return qb;
        },
        getMany: () => {
          const rows = pools
            .filter((p) => !afterId || p.id > afterId)
            .slice(0, take);
          batchRequests.push(rows.length);
          return Promise.resolve(rows);
        },
      };
      return qb;
    }),
  });

  beforeEach(() => {
    pools = [];
    batchRequests = [];
    env = {};
    update = jest.fn().mockResolvedValue({ affected: 1 });
    contract = { fetchTotalRaisedOnChain: jest.fn() };
    lock = {
      runExclusive: jest.fn(
        (_k: number, _j: string, cb: () => Promise<unknown>) => cb(),
      ),
    };
    scheduler = {
      addInterval: jest.fn(),
      deleteInterval: jest.fn(),
      doesExist: jest.fn().mockReturnValue(true),
    };
    service = new ReconciliationService(
      buildRepo() as never,
      contract as unknown as ContractService,
      lock as unknown as AdvisoryLockService,
      { get: (k: string) => env[k] } as unknown as ConfigService,
      scheduler as unknown as SchedulerRegistry,
    );
    jest.spyOn(service['logger'], 'warn').mockImplementation();
    jest.spyOn(service['logger'], 'log').mockImplementation();
    jest.spyOn(service['logger'], 'error').mockImplementation();
  });

  afterEach(() => jest.restoreAllMocks());

  it('leaves matching pools untouched', async () => {
    pools = [makePool(1, '500')];
    contract.fetchTotalRaisedOnChain.mockResolvedValue(500n);

    const result = await service.reconcile('manual');

    expect(result).toMatchObject({
      checked: 1,
      matched: 1,
      drifted: 0,
      corrected: 0,
    });
    expect(update).not.toHaveBeenCalled();
  });

  it('corrects positive drift (database behind chain)', async () => {
    pools = [makePool(1, '100')];
    contract.fetchTotalRaisedOnChain.mockResolvedValue(250n);

    const result = await service.reconcile();

    expect(update).toHaveBeenCalledWith(
      { id: pools[0].id, raised: '100' },
      { raised: '250' },
    );
    expect(result.corrected).toBe(1);
    expect(result.drifts[0]).toMatchObject({
      dbRaised: '100',
      onChainRaised: '250',
      delta: '150',
    });
  });

  it('corrects negative drift (database ahead of chain, e.g. refunds)', async () => {
    pools = [makePool(1, '900')];
    contract.fetchTotalRaisedOnChain.mockResolvedValue(400n);

    const result = await service.reconcile();

    expect(update).toHaveBeenCalledWith(
      { id: pools[0].id, raised: '900' },
      { raised: '400' },
    );
    expect(result.drifts[0].delta).toBe('-500');
  });

  it('handles amounts above Number.MAX_SAFE_INTEGER exactly', async () => {
    pools = [makePool(1, '9007199254740993')];
    contract.fetchTotalRaisedOnChain.mockResolvedValue(I128_MAX);

    await service.reconcile();

    expect(update).toHaveBeenCalledWith(
      { id: pools[0].id, raised: '9007199254740993' },
      { raised: I128_MAX.toString() },
    );
  });

  it('is idempotent: a second run after correction changes nothing', async () => {
    pools = [makePool(1, '100')];
    contract.fetchTotalRaisedOnChain.mockResolvedValue(250n);
    update.mockImplementation(
      (where: { id: string }, set: { raised: string }) => {
        pools.find((p) => p.id === where.id)!.raised = set.raised;
        return Promise.resolve({ affected: 1 });
      },
    );

    await service.reconcile();
    const second = await service.reconcile();

    expect(update).toHaveBeenCalledTimes(1);
    expect(second).toMatchObject({ matched: 1, drifted: 0 });
  });

  it('does not touch the database when the RPC call fails', async () => {
    pools = [makePool(1, '100'), makePool(2, '100')];
    contract.fetchTotalRaisedOnChain
      .mockRejectedValueOnce(new Error('rpc down'))
      .mockResolvedValueOnce(300n);

    const result = await service.reconcile();

    expect(result).toMatchObject({ rpcErrors: 1, corrected: 1, checked: 2 });
    expect(update).toHaveBeenCalledTimes(1);
    expect((update.mock.calls as [{ id: string }][])[0][0].id).toBe(
      pools[1].id,
    );
  });

  it('keeps going when one database update fails', async () => {
    pools = [makePool(1, '1'), makePool(2, '1')];
    contract.fetchTotalRaisedOnChain.mockResolvedValue(9n);
    update
      .mockRejectedValueOnce(new Error('deadlock'))
      .mockResolvedValueOnce({ affected: 1 });

    const result = await service.reconcile();

    expect(result).toMatchObject({ dbErrors: 1, corrected: 1, drifted: 2 });
  });

  it('counts a conditional update that matched no row as a conflict', async () => {
    pools = [makePool(1, '1')];
    contract.fetchTotalRaisedOnChain.mockResolvedValue(9n);
    update.mockResolvedValue({ affected: 0 });

    const result = await service.reconcile();

    expect(result).toMatchObject({ conflicts: 1, corrected: 0 });
  });

  it('skips pools whose contract id is not a valid on-chain id', async () => {
    pools = [{ ...makePool(1, '1'), contractPoolId: 'abc' }];

    const result = await service.reconcile();

    expect(contract.fetchTotalRaisedOnChain).not.toHaveBeenCalled();
    expect(result.rpcErrors).toBe(1);
  });

  it('processes pools in batches of RECONCILIATION_BATCH_SIZE', async () => {
    env.RECONCILIATION_BATCH_SIZE = '2';
    pools = [1, 2, 3, 4, 5].map((n) => makePool(n, '7'));
    contract.fetchTotalRaisedOnChain.mockResolvedValue(7n);

    const result = await service.reconcile();

    expect(batchRequests).toEqual([2, 2, 1]);
    expect(result.checked).toBe(5);
    expect(contract.fetchTotalRaisedOnChain).toHaveBeenCalledTimes(5);
  });

  it('skips when another instance holds the advisory lock', async () => {
    lock.runExclusive.mockResolvedValue(undefined);

    const result = await service.reconcile();

    expect(result.status).toBe('skipped');
    expect(contract.fetchTotalRaisedOnChain).not.toHaveBeenCalled();
  });

  it('does not overlap runs within one process', async () => {
    pools = [makePool(1, '1')];
    let release!: (v: bigint) => void;
    contract.fetchTotalRaisedOnChain.mockReturnValue(
      new Promise<bigint>((resolve) => (release = resolve)),
    );

    const first = service.reconcile();
    await new Promise((r) => setImmediate(r));
    const second = await service.reconcile('manual');
    release(1n);

    expect(second.status).toBe('skipped');
    expect((await first).status).toBe('completed');
    expect(lock.runExclusive).toHaveBeenCalledTimes(1);
  });

  it('accumulates metrics across runs', async () => {
    pools = [makePool(1, '1')];
    contract.fetchTotalRaisedOnChain.mockResolvedValue(5n);

    await service.reconcile();

    expect(service.getStats()).toMatchObject({
      runs: 1,
      checked: 1,
      driftDetected: 1,
      driftCorrected: 1,
    });
    expect(service.getStats().lastRun?.drifts).toHaveLength(1);
  });

  describe('scheduling', () => {
    const registeredHandle = () =>
      (scheduler.addInterval.mock.calls as [string, NodeJS.Timeout][])[0][1];

    afterEach(() => jest.useRealTimers());

    it('registers an interval from RECONCILIATION_INTERVAL_MS and runs it', async () => {
      jest.useFakeTimers();
      env.RECONCILIATION_INTERVAL_MS = '1000';
      const spy = jest
        .spyOn(service, 'reconcile')
        .mockResolvedValue({} as never);

      service.onModuleInit();
      expect(scheduler.addInterval).toHaveBeenCalledWith(
        'poolRaisedReconciliation',
        expect.anything(),
      );
      await jest.advanceTimersByTimeAsync(2100);

      expect(spy).toHaveBeenCalledTimes(2);
      expect(spy).toHaveBeenCalledWith('scheduled');
      service.onModuleDestroy();
      expect(scheduler.deleteInterval).toHaveBeenCalled();
      clearInterval(registeredHandle());
    });

    it('falls back to the default interval for invalid values', () => {
      env.RECONCILIATION_INTERVAL_MS = 'nope';
      const setSpy = jest.spyOn(global, 'setInterval');

      service.onModuleInit();

      expect(setSpy).toHaveBeenCalledWith(expect.any(Function), 300000);
      clearInterval(registeredHandle());
    });

    it('does not schedule when RECONCILIATION_ENABLED=false', () => {
      env.RECONCILIATION_ENABLED = 'false';

      service.onModuleInit();

      expect(scheduler.addInterval).not.toHaveBeenCalled();
    });
  });
});
