import { DataSource } from 'typeorm';
import { AdvisoryLockService } from '../common/advisory-lock.service';
import { CreateNoncesTable1800000000001 } from '../migrations/1800000000001-CreateNoncesTable';
import {
  createSchemaDataSource,
  dropSchemaAndDestroy,
} from '../testing/test-database';
import { Nonce } from './nonce.entity';
import { NonceService } from './nonce.service';

describe('Nonce consumption in Postgres', () => {
  const schema = 'spec_nonce_consumption';
  const owner = 'wallet-owner';
  let dataSource: DataSource;
  let service: NonceService;

  beforeAll(async () => {
    dataSource = await createSchemaDataSource(schema, { entities: [Nonce] });
    const runner = dataSource.createQueryRunner();
    try {
      await new CreateNoncesTable1800000000001().up(runner);
    } finally {
      await runner.release();
    }
    service = new NonceService(
      dataSource.getRepository(Nonce),
      new AdvisoryLockService(dataSource),
    );
  });

  afterAll(async () => {
    await dropSchemaAndDestroy(dataSource, schema);
  });

  it('allows exactly one concurrent claim on an unused nonce', async () => {
    const nonce = await service.generateNonce(owner);
    const claims = await Promise.all(
      Array.from({ length: 12 }, () => service.consumeNonce(nonce, owner)),
    );
    expect(claims.filter(Boolean)).toHaveLength(1);
    await expect(service.consumeNonce(nonce, owner)).resolves.toBe(false);
  });

  it('binds a claim to the original wallet', async () => {
    const nonce = await service.generateNonce(owner);
    await expect(service.consumeNonce(nonce, 'another-wallet')).resolves.toBe(
      false,
    );
    await expect(service.consumeNonce(nonce, owner)).resolves.toBe(true);
  });

  it('rejects expired and unknown nonces', async () => {
    const nonce = await service.generateNonce(owner);
    await dataSource.query(
      "UPDATE nonces SET expires_at = LOCALTIMESTAMP - INTERVAL '1 second' WHERE nonce = $1",
      [nonce],
    );
    await expect(service.consumeNonce(nonce, owner)).resolves.toBe(false);
    await expect(service.consumeNonce('unknown-nonce', owner)).resolves.toBe(
      false,
    );
  });
});
