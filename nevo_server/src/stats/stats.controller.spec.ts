import { INestApplication } from '@nestjs/common';
import { Test } from '@nestjs/testing';
import request from 'supertest';
import { StatsController } from './stats.controller';
import { StatsService } from './stats.service';

describe('StatsController (HTTP)', () => {
  let app: INestApplication;
  const stats = {
    totalRaised: [
      { asset: 'XLM', amount: '170141183460469231731687303715884105727' },
    ],
    activePools: 2,
    completedPools: 1,
    uniqueDonors: 5,
  };

  beforeAll(async () => {
    const moduleRef = await Test.createTestingModule({
      controllers: [StatsController],
      providers: [
        {
          provide: StatsService,
          useValue: { getStats: () => Promise.resolve(stats) },
        },
      ],
    }).compile();
    app = moduleRef.createNestApplication();
    await app.init();
  });

  afterAll(() => app.close());

  it('GET /stats is public and returns the aggregate metrics', async () => {
    const res = await request(app.getHttpServer() as never)
      .get('/stats')
      .expect(200);

    expect(res.body).toEqual(stats);
  });
});
