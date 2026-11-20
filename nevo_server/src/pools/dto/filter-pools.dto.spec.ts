import { INestApplication, ValidationPipe } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import request from 'supertest';
import { ContractService } from '../../contract/contract.service';
import { PoolStatus } from '../pool.entity';
import { PoolsController } from '../pools.controller';
import { PoolsService } from '../pools.service';

/**
 * Pins down the GET /pools query contract as it behaves under the global
 * ValidationPipe: which filters reach the service, what they are coerced to,
 * and what is rejected outright.
 */
describe('FilterPoolsDto (GET /pools query contract)', () => {
  let app: INestApplication;
  let findAll: jest.Mock;

  beforeEach(async () => {
    findAll = jest
      .fn()
      .mockResolvedValue({ data: [], total: 0, page: 1, limit: 20 });

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [PoolsController],
      providers: [
        { provide: PoolsService, useValue: { findAll } },
        { provide: ContractService, useValue: {} },
      ],
    }).compile();

    app = moduleRef.createNestApplication();
    // Same configuration as main.ts.
    app.useGlobalPipes(
      new ValidationPipe({
        whitelist: true,
        forbidNonWhitelisted: true,
        transform: true,
      }),
    );
    await app.init();
  });

  afterEach(async () => {
    await app.close();
    jest.clearAllMocks();
  });

  it('validates the creator and search length', async () => {
    const creator = 'GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF';
    await request(app.getHttpServer())
      .get('/pools')
      .query({ creator })
      .expect(200);
    expect(findAll).toHaveBeenCalledWith(expect.objectContaining({ creator }));
    await request(app.getHttpServer())
      .get('/pools')
      .query({ creator: 'invalid' })
      .expect(400);
    await request(app.getHttpServer())
      .get('/pools')
      .query({ search: 'a'.repeat(201) })
      .expect(400);
  });

  it('applies the declared defaults when no query parameters are given', async () => {
    await request(app.getHttpServer()).get('/pools').expect(200);
    expect(findAll).toHaveBeenCalledWith({ page: 1, limit: 20 });
  });

  it('passes every documented filter through to the service', async () => {
    await request(app.getHttpServer())
      .get('/pools')
      .query({
        page: '2',
        limit: '25',
        search: 'clean water',
        category: 'health',
        status: PoolStatus.Active,
      })
      .expect(200);

    expect(findAll).toHaveBeenCalledWith({
      page: 2,
      limit: 25,
      search: 'clean water',
      category: 'health',
      status: PoolStatus.Active,
    });
  });

  it('coerces numeric query strings to numbers', async () => {
    await request(app.getHttpServer())
      .get('/pools')
      .query({ page: '3', limit: '50' })
      .expect(200);

    // toHaveBeenCalledWith compares strictly, so 3 here would not match '3'.
    expect(findAll).toHaveBeenCalledWith(
      expect.objectContaining({ page: 3, limit: 50 }),
    );
  });

  it('rejects a non-numeric page with 400', async () => {
    await request(app.getHttpServer())
      .get('/pools')
      .query({ page: 'abc' })
      .expect(400);
    expect(findAll).not.toHaveBeenCalled();
  });

  it('rejects page below the declared minimum with 400', async () => {
    await request(app.getHttpServer())
      .get('/pools')
      .query({ page: '0' })
      .expect(400);
    expect(findAll).not.toHaveBeenCalled();
  });

  it('rejects limit above the declared maximum with 400', async () => {
    await request(app.getHttpServer())
      .get('/pools')
      .query({ limit: '101' })
      .expect(400);
    expect(findAll).not.toHaveBeenCalled();
  });

  it('rejects a status outside the PoolStatus enum with 400', async () => {
    await request(app.getHttpServer())
      .get('/pools')
      .query({ status: 'archived' })
      .expect(400);
    expect(findAll).not.toHaveBeenCalled();
  });

  it('rejects unknown query parameters with 400', async () => {
    await request(app.getHttpServer())
      .get('/pools')
      .query({ sortBy: 'largest' })
      .expect(400);
    expect(findAll).not.toHaveBeenCalled();
  });
});
