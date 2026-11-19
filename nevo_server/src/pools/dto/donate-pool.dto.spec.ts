import { INestApplication, ValidationPipe } from '@nestjs/common';
import { Matches } from 'class-validator';
import { Test, TestingModule } from '@nestjs/testing';
import request from 'supertest';
import { ContractService } from '../../contract/contract.service';
import { PoolsController } from '../pools.controller';
import { PoolsService } from '../pools.service';

/**
 * Pins down the POST /pools/:id/donate body contract as it behaves under the
 * global ValidationPipe: which amounts are accepted and which are rejected.
 *
 * The route requires JWT auth (@UseGuards(JwtAuthGuard)), so we bypass the
 * guard by overriding it on the module level — the DTO validation runs before
 * the guard in NestJS's request lifecycle when the pipe is global, but because
 * guards run before the handler we need to mock the guard to reach the pipe.
 * We achieve this by providing a mock JwtAuthGuard that always allows requests.
 */
import { JwtAuthGuard } from '../../auth/jwt-auth.guard';

describe('DonatePoolDto (POST /pools/:id/donate body contract)', () => {
  let app: INestApplication;
  let buildDonateTransaction: jest.Mock;
  let findByContractId: jest.Mock;

  const POOL_ID = '1';

  beforeEach(async () => {
    buildDonateTransaction = jest
      .fn()
      .mockReturnValue('unsigned-xdr-string');
    findByContractId = jest
      .fn()
      .mockResolvedValue({ id: POOL_ID, creatorWallet: 'GWALLET' });

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [PoolsController],
      providers: [
        {
          provide: PoolsService,
          useValue: {
            findByContractId,
            create: jest.fn(),
            findAll: jest.fn(),
            findOneMerged: jest.fn(),
            updateMeta: jest.fn(),
            buildWithdrawTx: jest.fn(),
            buildClosePoolTx: jest.fn(),
          },
        },
        {
          provide: ContractService,
          useValue: { buildDonateTransaction },
        },
      ],
    })
      .overrideGuard(JwtAuthGuard)
      .useValue({
        canActivate: (ctx: import('@nestjs/common').ExecutionContext) => {
          const req = ctx.switchToHttp().getRequest();
          req.user = { sub: 'user-id', publicKey: 'GPUBLICKEY' };
          return true;
        },
      })
      .compile();

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

  it('accepts a valid numeric amount string and reaches the service', async () => {
    await request(app.getHttpServer())
      .post(`/pools/${POOL_ID}/donate`)
      .send({ amount: '10000000' })
      .expect(201);

    expect(buildDonateTransaction).toHaveBeenCalled();
  });

  it('rejects a missing amount with 400', async () => {
    await request(app.getHttpServer())
      .post(`/pools/${POOL_ID}/donate`)
      .send({})
      .expect(400);

    expect(buildDonateTransaction).not.toHaveBeenCalled();
  });

  it('rejects a non-numeric amount string with 400', async () => {
    await request(app.getHttpServer())
      .post(`/pools/${POOL_ID}/donate`)
      .send({ amount: 'not-a-number' })
      .expect(400);

    expect(buildDonateTransaction).not.toHaveBeenCalled();
  });

  it('3. rejects zero amount with 400 (Out-of-range)', async () => {
    await request(app.getHttpServer())
      .post(`/pools/${POOL_ID}/donate`)
      .send({ amount: '0' })
      .expect(400);

    expect(buildDonateTransaction).not.toHaveBeenCalled();
  });

  it('3. rejects negative amount with 400 (Out-of-range)', async () => {
    await request(app.getHttpServer())
      .post(`/pools/${POOL_ID}/donate`)
      .send({ amount: '-10' })
      .expect(400);

    expect(buildDonateTransaction).not.toHaveBeenCalled();
  });

  it('1. rejects an empty amount string with 400 (Null/empty parameters handled)', async () => {
    await request(app.getHttpServer())
      .post(`/pools/${POOL_ID}/donate`)
      .send({ amount: '' })
      .expect(400);

    expect(buildDonateTransaction).not.toHaveBeenCalled();
  });

  it('5. rejects unknown extra fields under forbidNonWhitelisted with 400 (Parameter combinations validated)', async () => {
    await request(app.getHttpServer())
      .post(`/pools/${POOL_ID}/donate`)
      .send({ amount: '10000000', extra: 'field' })
      .expect(400);

    expect(buildDonateTransaction).not.toHaveBeenCalled();
  });
});
