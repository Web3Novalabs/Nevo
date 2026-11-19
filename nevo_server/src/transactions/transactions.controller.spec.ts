import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import request from 'supertest';
import { JwtService, JwtSignOptions } from '@nestjs/jwt';
import { TransactionsController } from './transactions.controller';
import { ContractService } from '../contract/contract.service';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { JwtStrategy } from '../auth/jwt.strategy';
import { JWT_SECRET_FALLBACK } from '../auth/jwt.config';
import { StellarError } from '../contract/stellar.error';

describe('TransactionsController', () => {
  let controller: TransactionsController;
  let contractService: { submitSignedXdr: jest.Mock };

  beforeEach(async () => {
    contractService = {
      submitSignedXdr: jest.fn().mockResolvedValue('tx-hash-12345'),
    };

    const module: TestingModule = await Test.createTestingModule({
      controllers: [TransactionsController],
      providers: [
        {
          provide: ContractService,
          useValue: contractService,
        },
      ],
    }).compile();

    controller = module.get<TransactionsController>(TransactionsController);
    jest.clearAllMocks();
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  describe('submit (POST /transactions/submit)', () => {
    it('submits signed XDR via ContractService and returns txHash', async () => {
      contractService.submitSignedXdr.mockResolvedValue('tx-hash-12345');

      const result = await controller.submit({ xdr: 'AAAA...signedXdr' });

      expect(contractService.submitSignedXdr).toHaveBeenCalledWith(
        'AAAA...signedXdr',
      );
      expect(result).toEqual({ txHash: 'tx-hash-12345' });
    });

    it('forwards the exact xdr string to ContractService.submitSignedXdr', async () => {
      const signedXdr = 'BBBB...differentXdr';
      contractService.submitSignedXdr.mockResolvedValue('different-hash');

      const result = await controller.submit({ xdr: signedXdr });

      expect(contractService.submitSignedXdr).toHaveBeenCalledTimes(1);
      expect(contractService.submitSignedXdr).toHaveBeenCalledWith(signedXdr);
      expect(result).toEqual({ txHash: 'different-hash' });
    });

    it('propagates StellarError thrown by ContractService (tx_bad_auth)', async () => {
      const error = new StellarError('tx_bad_auth');
      contractService.submitSignedXdr.mockRejectedValue(error);

      await expect(controller.submit({ xdr: 'bad-xdr' })).rejects.toThrow(
        StellarError,
      );
    });

    it('propagates StellarError thrown by ContractService (op_underfunded)', async () => {
      const error = new StellarError('op_underfunded');
      contractService.submitSignedXdr.mockRejectedValue(error);

      await expect(controller.submit({ xdr: 'another-xdr' })).rejects.toThrow(
        StellarError,
      );
    });

    it('propagates generic errors thrown by ContractService without suppression', async () => {
      const error = new Error('RPC connection refused');
      contractService.submitSignedXdr.mockRejectedValue(error);

      await expect(controller.submit({ xdr: 'any-xdr' })).rejects.toThrow(
        'RPC connection refused',
      );
    });

    it('does not call submitSignedXdr more than once per request', async () => {
      contractService.submitSignedXdr.mockResolvedValue('single-call-hash');

      await controller.submit({ xdr: 'once-only' });

      expect(contractService.submitSignedXdr).toHaveBeenCalledTimes(1);
    });
  });

  describe('JwtAuthGuard protection on submit route', () => {
    it('applies JwtAuthGuard to the submit method', () => {
      const metadata = Reflect.getMetadata(
        '__guards__',
        TransactionsController.prototype.submit,
      );
      expect(metadata).toBeDefined();
      expect(Array.isArray(metadata)).toBe(true);
      const hasJwtGuard = (metadata as unknown[]).some(
        (guard) => guard === JwtAuthGuard,
      );
      expect(hasJwtGuard).toBe(true);
    });
  });
});

// ---------------------------------------------------------------------------
// HTTP-level tests – spin up a real NestJS app to verify JwtAuthGuard rejects
// unauthenticated / invalid / expired requests to POST /transactions/submit
// ---------------------------------------------------------------------------
describe('TransactionsController (guard / HTTP)', () => {
  let app: INestApplication;
  let contractService: { submitSignedXdr: jest.Mock };

  const jwtSecret = process.env.JWT_SECRET ?? JWT_SECRET_FALLBACK;
  const signToken = (
    payload: object,
    opts?: JwtSignOptions,
    secret: string = jwtSecret,
  ) => new JwtService({ secret }).sign(payload, opts);

  beforeEach(async () => {
    contractService = {
      submitSignedXdr: jest.fn().mockResolvedValue('tx-hash-xyz'),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [TransactionsController],
      providers: [
        { provide: ContractService, useValue: contractService },
        JwtStrategy,
      ],
    }).compile();

    app = moduleRef.createNestApplication();
    await app.init();
  });

  afterEach(async () => {
    await app.close();
    jest.clearAllMocks();
  });

  it('returns 401 when no Authorization header is provided', async () => {
    await request(app.getHttpServer())
      .post('/transactions/submit')
      .send({ xdr: 'AAAA...xdr' })
      .expect(401);

    expect(contractService.submitSignedXdr).not.toHaveBeenCalled();
  });

  it('returns 401 when an invalid / malformed token is provided', async () => {
    await request(app.getHttpServer())
      .post('/transactions/submit')
      .set('Authorization', 'Bearer not-a-real-token')
      .send({ xdr: 'AAAA...xdr' })
      .expect(401);

    expect(contractService.submitSignedXdr).not.toHaveBeenCalled();
  });

  it('returns 401 when an already-expired JWT is provided', async () => {
    const expiredToken = signToken({ sub: 'GPUBKEY' }, { expiresIn: '-1s' });

    await request(app.getHttpServer())
      .post('/transactions/submit')
      .set('Authorization', `Bearer ${expiredToken}`)
      .send({ xdr: 'AAAA...xdr' })
      .expect(401);

    expect(contractService.submitSignedXdr).not.toHaveBeenCalled();
  });

  it('returns 401 when a JWT signed with the wrong secret is provided', async () => {
    const forgedToken = signToken(
      { sub: 'GPUBKEY' },
      undefined,
      'wrong-secret',
    );

    await request(app.getHttpServer())
      .post('/transactions/submit')
      .set('Authorization', `Bearer ${forgedToken}`)
      .send({ xdr: 'AAAA...xdr' })
      .expect(401);

    expect(contractService.submitSignedXdr).not.toHaveBeenCalled();
  });

  it('rejects expired/invalid JWTs with the same response as a missing token', async () => {
    const expiredToken = signToken({ sub: 'GPUBKEY' }, { expiresIn: '-1s' });
    const server = app.getHttpServer();

    const missing = await request(server)
      .post('/transactions/submit')
      .send({ xdr: 'AAAA...xdr' });
    const invalid = await request(server)
      .post('/transactions/submit')
      .set('Authorization', 'Bearer not-a-real-token')
      .send({ xdr: 'AAAA...xdr' });
    const expired = await request(server)
      .post('/transactions/submit')
      .set('Authorization', `Bearer ${expiredToken}`)
      .send({ xdr: 'AAAA...xdr' });

    expect(missing.status).toBe(401);
    expect(invalid.status).toBe(missing.status);
    expect(expired.status).toBe(missing.status);
    expect(invalid.body).toEqual(missing.body);
    expect(expired.body).toEqual(missing.body);
    expect(contractService.submitSignedXdr).not.toHaveBeenCalled();
  });

  it('returns 201 and calls submitSignedXdr when a valid JWT is provided', async () => {
    const token = signToken({ sub: 'GPUBKEY' });

    await request(app.getHttpServer())
      .post('/transactions/submit')
      .set('Authorization', `Bearer ${token}`)
      .send({ xdr: 'AAAA...valid-xdr' })
      .expect(201);

    expect(contractService.submitSignedXdr).toHaveBeenCalledWith(
      'AAAA...valid-xdr',
    );
  });
});
