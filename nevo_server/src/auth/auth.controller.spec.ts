import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import { APP_GUARD } from '@nestjs/core';
import { ThrottlerGuard, ThrottlerModule } from '@nestjs/throttler';
import request from 'supertest';
import * as StellarSdk from '@stellar/stellar-sdk';
import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { NonceService } from './nonce.service';
import { JwtService } from '@nestjs/jwt';
import { UsersService } from '../users/users.service';

describe('AuthController (challenge/verify)', () => {
  let app: INestApplication;
  let nonceService: NonceService;
  let authService: AuthService;

  const keypair = StellarSdk.Keypair.random();
  const publicKey = keypair.publicKey();

  beforeEach(async () => {
    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [AuthController],
      providers: [
        AuthService,
        {
          provide: NonceService,
          useValue: {
            generateNonce: jest.fn(),
            findAndValidateNonce: jest.fn(),
            markNonceAsUsed: jest.fn(),
          },
        },
        {
          provide: UsersService,
          useValue: {
            findOrCreate: jest.fn().mockResolvedValue({
              id: 'user-id',
              publicKey,
              displayName: null,
              createdAt: new Date(),
              updatedAt: new Date(),
            }),
          },
        },
        {
          provide: JwtService,
          useValue: {
            sign: jest.fn().mockReturnValue('access-token'),
          },
        },
      ],
    }).compile();

    nonceService = moduleRef.get<NonceService>(NonceService);
    authService = moduleRef.get<AuthService>(AuthService);

    app = moduleRef.createNestApplication();
    // Mirror the global pipe configured in main.ts so these tests exercise the
    // same validation the real server applies.
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

  it('GET /auth/challenge with valid publicKey returns nonce object', async () => {
    jest.spyOn(nonceService, 'generateNonce').mockResolvedValue('nonce-123');

    await request(app.getHttpServer())
      .get('/auth/challenge')
      .query({ publicKey })
      .expect(200)
      .expect(({ body }) => {
        expect(body).toEqual({ nonce: 'nonce-123' });
      });

    expect(nonceService.generateNonce).toHaveBeenCalledWith(publicKey);
  });

  it('GET /auth/challenge without publicKey returns 400', async () => {
    await request(app.getHttpServer()).get('/auth/challenge').expect(400);
  });

  it('POST /auth/verify with valid signature returns accessToken', async () => {
    const nonce = 'test-nonce';
    const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

    jest.spyOn(authService as any, 'verifySignature').mockReturnValue(true);
    jest.spyOn(nonceService, 'findAndValidateNonce').mockResolvedValue({
      id: 'nonce-id',
      nonce,
      publicKey,
      createdAt: new Date(),
      expiresAt: new Date(Date.now() + 10000),
      used: false,
    });
    jest.spyOn(nonceService, 'markNonceAsUsed').mockResolvedValue();

    await request(app.getHttpServer())
      .post('/auth/verify')
      .send({ publicKey, signature, message: nonce })
      .expect(201)
      .expect(({ body }) => {
        expect(body).toMatchObject({
          accessToken: 'access-token',
          user: expect.objectContaining({ publicKey }),
        });
      });
  });

  it('POST /auth/verify with invalid signature returns 401', async () => {
    const nonce = 'test-nonce';
    const invalidSignature = '00'.repeat(64);

    jest.spyOn(authService as any, 'verifySignature').mockReturnValue(false);

    await request(app.getHttpServer())
      .post('/auth/verify')
      .send({ publicKey, signature: invalidSignature, message: nonce })
      .expect(401);
  });

  it('POST /auth/verify with unknown nonce returns 401', async () => {
    const nonce = 'unknown-nonce';
    const signature = keypair.sign(Buffer.from(nonce)).toString('hex');

    jest.spyOn(authService as any, 'verifySignature').mockReturnValue(true);
    jest.spyOn(nonceService, 'findAndValidateNonce').mockResolvedValue(null);

    await request(app.getHttpServer())
      .post('/auth/verify')
      .send({ publicKey, signature, message: nonce })
      .expect(401);
  });

  describe('POST /auth/verify body validation', () => {
    // Guards against the regression where VerifyDto was an interface: interfaces
    // are erased at runtime, so the global ValidationPipe skipped this body and
    // let malformed or extra fields through to the service untouched.
    const validSignature = (nonce: string) =>
      keypair.sign(Buffer.from(nonce)).toString('hex');

    const expectRejected = async (body: Record<string, unknown>) => {
      const verifySpy = jest.spyOn(authService, 'verify');
      await request(app.getHttpServer())
        .post('/auth/verify')
        .send(body)
        .expect(400);
      // The request must not reach the service at all.
      expect(verifySpy).not.toHaveBeenCalled();
    };

    it('rejects an empty body', async () => {
      await expectRejected({});
    });

    it('rejects a body missing publicKey', async () => {
      await expectRejected({
        signature: validSignature('n'),
        message: 'n',
      });
    });

    it('rejects a body missing signature', async () => {
      await expectRejected({ publicKey, message: 'n' });
    });

    it('rejects a body missing message', async () => {
      await expectRejected({ publicKey, signature: validSignature('n') });
    });

    it('rejects non-string fields', async () => {
      await expectRejected({ publicKey, signature: 12345, message: 'n' });
    });

    it('rejects an empty-string message', async () => {
      await expectRejected({
        publicKey,
        signature: validSignature('n'),
        message: '',
      });
    });

    it('rejects a malformed Stellar public key', async () => {
      await expectRejected({
        publicKey: 'not-a-stellar-key',
        signature: validSignature('n'),
        message: 'n',
      });
    });

    it('rejects a non-hex signature', async () => {
      await expectRejected({
        publicKey,
        signature: 'zzzz-not-hex',
        message: 'n',
      });
    });

    it('rejects unknown extra fields (forbidNonWhitelisted)', async () => {
      await expectRejected({
        publicKey,
        signature: validSignature('n'),
        message: 'n',
        isAdmin: true,
      });
    });
  });
});
