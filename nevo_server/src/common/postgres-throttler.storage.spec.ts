import {
  Controller,
  Get,
  Global,
  INestApplication,
  Module,
} from '@nestjs/common';
import { APP_GUARD } from '@nestjs/core';
import { NestExpressApplication } from '@nestjs/platform-express';
import { Test } from '@nestjs/testing';
import { Throttle, ThrottlerGuard } from '@nestjs/throttler';
import request from 'supertest';
import type { App } from 'supertest/types';
import { DataSource } from 'typeorm';
import { parseTrustProxy } from '../config/environment';
import { CreateThrottleBuckets1800000000006 } from '../migrations/1800000000006-CreateThrottleBuckets';
import {
  createSchemaDataSource,
  dropSchemaAndDestroy,
} from '../testing/test-database';
import { PostgresThrottlerStorage } from './postgres-throttler.storage';
import { ThrottlingModule } from './throttling.module';

@Controller()
class ExampleController {
  @Get()
  @Throttle({ default: { limit: 1, ttl: 60_000 } })
  read(): string {
    return 'ok';
  }
}

describe('Postgres throttling', () => {
  const schema = 'spec_postgres_throttler';
  let dataSource: DataSource;
  let apps: INestApplication<App>[] = [];

  async function startApp(
    trustProxy: string,
  ): Promise<NestExpressApplication<App>> {
    @Global()
    @Module({
      providers: [{ provide: DataSource, useValue: dataSource }],
      exports: [DataSource],
    })
    class TestDatabaseModule {}

    const moduleRef = await Test.createTestingModule({
      imports: [TestDatabaseModule, ThrottlingModule],
      controllers: [ExampleController],
      providers: [{ provide: APP_GUARD, useClass: ThrottlerGuard }],
    }).compile();

    const app = moduleRef.createNestApplication<NestExpressApplication<App>>();
    apps.push(app);
    app.set('trust proxy', parseTrustProxy(trustProxy));
    await app.init();
    return app;
  }

  beforeAll(async () => {
    dataSource = await createSchemaDataSource(schema);
    const runner = dataSource.createQueryRunner();
    try {
      await new CreateThrottleBuckets1800000000006().up(runner);
    } finally {
      await runner.release();
    }
  });

  afterEach(async () => {
    await Promise.all(apps.map((app) => app.close()));
    apps = [];
    await dataSource.query('TRUNCATE throttle_buckets');
  });

  afterAll(async () => {
    await dropSchemaAndDestroy(dataSource, schema);
  });

  it('shares a client bucket across app instances and gives other clients their own buckets', async () => {
    const first = request((await startApp('loopback')).getHttpServer());
    const second = request((await startApp('loopback')).getHttpServer());

    await first.get('/').set('X-Forwarded-For', '192.0.2.10').expect(200);
    await second.get('/').set('X-Forwarded-For', '192.0.2.10').expect(429);
    await second.get('/').set('X-Forwarded-For', '192.0.2.20').expect(200);
  });

  it('uses the connecting IP when proxy trust is disabled', async () => {
    const client = request((await startApp('false')).getHttpServer());

    await client.get('/').set('X-Forwarded-For', '192.0.2.10').expect(200);
    await client.get('/').set('X-Forwarded-For', '192.0.2.20').expect(429);
  });

  it('counts simultaneous increments and resets a bucket after its block expires', async () => {
    const storage = new PostgresThrottlerStorage(dataSource);
    const records = await Promise.all(
      Array.from({ length: 6 }, () =>
        storage.increment('shared', 60_000, 3, 60_000, 'default'),
      ),
    );
    expect(records.filter(({ isBlocked }) => !isBlocked)).toHaveLength(3);

    await dataSource.query(
      "UPDATE throttle_buckets SET blocked_until = statement_timestamp() - INTERVAL '1 second'",
    );
    await expect(
      storage.increment('shared', 60_000, 3, 60_000, 'default'),
    ).resolves.toMatchObject({
      totalHits: 1,
      isBlocked: false,
    });
  });
});
