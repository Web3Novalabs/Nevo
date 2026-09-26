import { INestApplication, ValidationPipe } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import request from 'supertest';
import { ContractService } from '../../contract/contract.service';
import { PoolsController } from '../pools.controller';
import { PoolsService } from '../pools.service';

/**
 * Pins down the POST /pools body contract as it behaves under the global
 * ValidationPipe: which payloads are accepted, which are rejected, and that
 * unknown fields are forbidden.
 */
describe('CreatePoolDto (POST /pools body contract)', () => {
  let app: INestApplication;
  let create: jest.Mock;

  /** A minimal fully-valid payload that should always pass validation. */
  const validBody = {
    contractPoolId: 'pool-abc-123',
    creatorWallet: 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCW7M',
    title: 'Clean Water Initiative',
    description: 'Providing clean drinking water to rural communities.',
    goal: '1000000000',
  };

  beforeEach(async () => {
    create = jest.fn().mockResolvedValue({ id: 1, ...validBody });

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [PoolsController],
      providers: [
        {
          provide: PoolsService,
          useValue: {
            create,
            findAll: jest.fn(),
            findOneMerged: jest.fn(),
            updateMeta: jest.fn(),
            findByContractId: jest.fn(),
            buildWithdrawTx: jest.fn(),
            buildClosePoolTx: jest.fn(),
          },
        },
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

  it('accepts a fully valid payload and reaches the service', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send(validBody)
      .expect(201);

    expect(create).toHaveBeenCalledWith(validBody);
  });

  it('accepts an optional category and imageUrl alongside the required fields', async () => {
    const bodyWithOptionals = {
      ...validBody,
      category: 'health',
      imageUrl: 'https://example.com/image.png',
    };

    await request(app.getHttpServer())
      .post('/pools')
      .send(bodyWithOptionals)
      .expect(201);

    expect(create).toHaveBeenCalledWith(bodyWithOptionals);
  });

  it('rejects a missing required field (contractPoolId) with 400', async () => {
    const { contractPoolId: _omitted, ...bodyWithoutId } = validBody;
    await request(app.getHttpServer())
      .post('/pools')
      .send(bodyWithoutId)
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('rejects a missing required field (creatorWallet) with 400', async () => {
    const { creatorWallet: _omitted, ...bodyWithoutWallet } = validBody;
    await request(app.getHttpServer())
      .post('/pools')
      .send(bodyWithoutWallet)
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('rejects a malformed creatorWallet with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, creatorWallet: 'not-a-stellar-key' })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('rejects a title that exceeds 100 characters with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, title: 'A'.repeat(101) })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('accepts a title exactly at the 100-character limit', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, title: 'A'.repeat(100) })
      .expect(201);
  });

  it('rejects a description that exceeds 1000 characters with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, description: 'B'.repeat(1001) })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('accepts a description exactly at the 1000-character limit', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, description: 'B'.repeat(1000) })
      .expect(201);
  });

  it('rejects a category that exceeds 100 characters with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, category: 'C'.repeat(101) })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('accepts a category exactly at the 100-character limit', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, category: 'C'.repeat(100) })
      .expect(201);
  });

  it('rejects an invalid imageUrl string with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, imageUrl: 'not-a-url' })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('rejects a non-numeric goal string with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, goal: 'not-a-number' })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('rejects a zero goal with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, goal: '0' })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('rejects a negative goal with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, goal: '-1' })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('rejects unknown extra fields under forbidNonWhitelisted with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, extraField: 'surprise' })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('rejects a non-URL imageUrl with 400', async () => {
    await request(app.getHttpServer())
      .post('/pools')
      .send({ ...validBody, imageUrl: 'not a url' })
      .expect(400);

    expect(create).not.toHaveBeenCalled();
  });

  it('accepts a valid https imageUrl alongside the required fields', async () => {
    const body = { ...validBody, imageUrl: 'https://example.com/pool.png' };
    await request(app.getHttpServer()).post('/pools').send(body).expect(201);

    expect(create).toHaveBeenCalledWith(body);
  });

  it('accepts a null imageUrl (nullable optional field)', async () => {
    const body = { ...validBody, imageUrl: null };
    await request(app.getHttpServer()).post('/pools').send(body).expect(201);

    expect(create).toHaveBeenCalledWith(body);
  });
});
