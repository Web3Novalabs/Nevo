import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import request from 'supertest';
import { JwtService, JwtSignOptions } from '@nestjs/jwt';
import { ContractController } from './contract.controller';
import { ContractService } from './contract.service';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { JwtStrategy } from '../auth/jwt.strategy';
import { JWT_SECRET_FALLBACK } from '../auth/jwt.config';
import { StellarError } from './stellar.error';

describe('ContractController', () => {
  let controller: ContractController;
  let contractService: { submitSignedXdr: jest.Mock };

  beforeEach(async () => {
    contractService = {
      submitSignedXdr: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      controllers: [ContractController],
      providers: [
        {
          provide: ContractService,
          useValue: contractService,
        },
      ],
    }).compile();

    controller = module.get<ContractController>(ContractController);
    jest.clearAllMocks();
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  describe('submitXdr (POST /contract/submit)', () => {
    it('forwards the xdr argument to ContractService.submitSignedXdr and returns txHash', async () => {
      const signedXdr = 'AAAA...valid-signed-xdr';
      contractService.submitSignedXdr.mockResolvedValue('tx-hash-abc123');

      const result = await controller.submitXdr({ xdr: signedXdr });

      expect(contractService.submitSignedXdr).toHaveBeenCalledTimes(1);
      expect(contractService.submitSignedXdr).toHaveBeenCalledWith(signedXdr);
      expect(result).toEqual({ txHash: 'tx-hash-abc123' });
    });

    it('propagates StellarError thrown by ContractService without swallowing it', async () => {
      const error = new StellarError('tx_bad_auth');
      contractService.submitSignedXdr.mockRejectedValue(error);

      await expect(controller.submitXdr({ xdr: 'bad-xdr' })).rejects.toThrow(
        StellarError,
      );

      expect(contractService.submitSignedXdr).toHaveBeenCalledWith('bad-xdr');
    });

    it('propagates generic errors thrown by ContractService', async () => {
      const error = new Error('network timeout');
      contractService.submitSignedXdr.mockRejectedValue(error);

      await expect(controller.submitXdr({ xdr: 'some-xdr' })).rejects.toThrow(
        'network timeout',
      );
    });
  });

  describe('JwtAuthGuard protection', () => {
    it('applies JwtAuthGuard to the submitXdr route handler', () => {
      // Verify the guard decorator is present on the submitXdr method via metadata
      const guards: unknown[] =
        Reflect.getMetadata('__guards__', controller.submitXdr) ?? [];
      const guardTypes = guards.map((g) =>
        typeof g === 'function'
          ? g
          : (g as { constructor: unknown }).constructor,
      );
      expect(guardTypes).toContain(JwtAuthGuard);
    });

    it('applies JwtAuthGuard at the method level of submitXdr', () => {
      const reflector = new Reflector();
      // The guard is registered as a NestJS metadata entry on the method
      const metadata = Reflect.getMetadata(
        '__guards__',
        ContractController.prototype.submitXdr,
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
// unauthenticated / invalid / expired requests to POST /contract/submit
// ---------------------------------------------------------------------------
describe('ContractController (guard / HTTP)', () => {
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
      submitSignedXdr: jest.fn().mockResolvedValue('tx-hash-abc'),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [ContractController],
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
      .post('/contract/submit')
      .send({ xdr: 'AAAA...xdr' })
      .expect(401);

    expect(contractService.submitSignedXdr).not.toHaveBeenCalled();
  });

  it('returns 401 when an invalid / malformed token is provided', async () => {
    await request(app.getHttpServer())
      .post('/contract/submit')
      .set('Authorization', 'Bearer not-a-real-token')
      .send({ xdr: 'AAAA...xdr' })
      .expect(401);

    expect(contractService.submitSignedXdr).not.toHaveBeenCalled();
  });

  it('returns 401 when an already-expired JWT is provided', async () => {
    const expiredToken = signToken({ sub: 'GPUBKEY' }, { expiresIn: '-1s' });

    await request(app.getHttpServer())
      .post('/contract/submit')
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
      .post('/contract/submit')
      .set('Authorization', `Bearer ${forgedToken}`)
      .send({ xdr: 'AAAA...xdr' })
      .expect(401);

    expect(contractService.submitSignedXdr).not.toHaveBeenCalled();
  });

  it('rejects expired/invalid JWTs with the same response as a missing token', async () => {
    const expiredToken = signToken({ sub: 'GPUBKEY' }, { expiresIn: '-1s' });
    const server = app.getHttpServer();

    const missing = await request(server)
      .post('/contract/submit')
      .send({ xdr: 'AAAA...xdr' });
    const invalid = await request(server)
      .post('/contract/submit')
      .set('Authorization', 'Bearer not-a-real-token')
      .send({ xdr: 'AAAA...xdr' });
    const expired = await request(server)
      .post('/contract/submit')
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
      .post('/contract/submit')
      .set('Authorization', `Bearer ${token}`)
      .send({ xdr: 'AAAA...valid-xdr' })
      .expect(201);

    expect(contractService.submitSignedXdr).toHaveBeenCalledWith(
      'AAAA...valid-xdr',
    );
  });
});
