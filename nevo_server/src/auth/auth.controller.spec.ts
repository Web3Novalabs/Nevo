import { Test, TestingModule } from '@nestjs/testing';
import {
  ExecutionContext,
  HttpException,
  HttpStatus,
  INestApplication,
  UnauthorizedException,
  ValidationPipe,
} from '@nestjs/common';
import { APP_GUARD } from '@nestjs/core';
import { ThrottlerGuard, ThrottlerModule } from '@nestjs/throttler';
import request from 'supertest';
import * as StellarSdk from '@stellar/stellar-sdk';

jest.mock('@nestjs/schedule', () => ({
  Cron: () => () => {},
  CronExpression: {},
  ScheduleModule: {},
  SchedulerRegistry: jest.fn(),
}));
import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { NonceService } from './nonce.service';
import { JwtService } from '@nestjs/jwt';
import { UsersService } from '../users/users.service';
import { JwtAuthGuard } from './jwt-auth.guard';

// Stand-in for the passport JWT guard: any bearer token authenticates as the
// test wallet, a missing header is rejected with 401.
const fakeJwtGuard = (publicKey: string) => ({
  canActivate: (context: ExecutionContext) => {
    const req = context.switchToHttp().getRequest<{
      headers: Record<string, string | undefined>;
      user?: { publicKey: string };
    }>();
    if (!req.headers.authorization?.startsWith('Bearer ')) {
      throw new UnauthorizedException();
    }
    req.user = { publicKey };
    return true;
  },
});

describe('AuthController throttle limits', () => {
  let app: INestApplication;
  let nonceService: NonceService;
  let authService: AuthService;

  const keypair = StellarSdk.Keypair.random();
  const publicKey = keypair.publicKey();

  beforeEach(async () => {
    const moduleRef: TestingModule = await Test.createTestingModule({
      imports: [
        ThrottlerModule.forRoot([
          {
            ttl: 60000,
            limit: 10,
          },
        ]),
      ],
      controllers: [AuthController],
      providers: [
        AuthService,
        {
          provide: APP_GUARD,
          useClass: ThrottlerGuard,
        },
        {
          provide: NonceService,
          useValue: {
            generateNonce: jest.fn().mockResolvedValue('nonce-123'),
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
    app.getHttpAdapter().getInstance().set('trust proxy', 1);
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

  it('GET /auth/challenge rejects the 11th request within 60s (limit 10)', async () => {
    const ip = '10.0.0.1';
    for (let i = 0; i < 10; i++) {
      await request(app.getHttpServer())
        .get('/auth/challenge')
        .query({ publicKey })
        .set('X-Forwarded-For', ip)
        .expect(200);
    }

    await request(app.getHttpServer())
      .get('/auth/challenge')
      .query({ publicKey })
      .set('X-Forwarded-For', ip)
      .expect(429);
  });

  it('POST /auth/verify rejects the 6th request within 60s (limit 5)', async () => {
    const ip = '10.0.0.2';
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

    for (let i = 0; i < 5; i++) {
      await request(app.getHttpServer())
        .post('/auth/verify')
        .set('X-Forwarded-For', ip)
        .send({ publicKey, signature, message: nonce })
        .expect(201);
    }

    await request(app.getHttpServer())
      .post('/auth/verify')
      .set('X-Forwarded-For', ip)
      .send({ publicKey, signature, message: nonce })
      .expect(429);
  });

  it('scopes throttle counters per IP', async () => {
    const ipA = '10.0.0.3';
    const ipB = '10.0.0.4';

    for (let i = 0; i < 10; i++) {
      await request(app.getHttpServer())
        .get('/auth/challenge')
        .query({ publicKey })
        .set('X-Forwarded-For', ipA)
        .expect(200);
    }

    await request(app.getHttpServer())
      .get('/auth/challenge')
      .query({ publicKey })
      .set('X-Forwarded-For', ipA)
      .expect(429);

    // A different IP must have its own counter and not be throttled.
    await request(app.getHttpServer())
      .get('/auth/challenge')
      .query({ publicKey })
      .set('X-Forwarded-For', ipB)
      .expect(200);
  });
});

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
    })
      .overrideGuard(JwtAuthGuard)
      .useValue(fakeJwtGuard(publicKey))
      .compile();

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

  describe('GET /auth/challenge publicKey validation', () => {
    const badChecksum =
      publicKey.slice(0, -1) + (publicKey.endsWith('A') ? 'B' : 'A');

    it.each([
      ['too short', 'GABC123'],
      ['lowercase', publicKey.toLowerCase()],
      ['oversized', 'G' + 'A'.repeat(5000)],
      ['not a G key', 'S' + publicKey.slice(1)],
      ['bad checksum', badChecksum],
    ])(
      'rejects a %s key with 400 and stores no nonce',
      async (_label, invalidKey) => {
        await request(app.getHttpServer())
          .get('/auth/challenge')
          .query({ publicKey: invalidKey })
          .expect(400);

        expect(nonceService.generateNonce).not.toHaveBeenCalled();
      },
    );

    it('returns 429 when the key has too many outstanding challenges', async () => {
      jest
        .spyOn(nonceService, 'generateNonce')
        .mockRejectedValue(
          new HttpException('Too many', HttpStatus.TOO_MANY_REQUESTS),
        );

      await request(app.getHttpServer())
        .get('/auth/challenge')
        .query({ publicKey })
        .expect(429);
    });
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

  describe('POST /auth/logout', () => {
    it('returns 401 without a token', async () => {
      const logoutSpy = jest.spyOn(authService, 'logout');

      await request(app.getHttpServer()).post('/auth/logout').expect(401);

      expect(logoutSpy).not.toHaveBeenCalled();
    });

    it('returns 200 with a valid token and calls authService.logout with the JWT publicKey', async () => {
      const logoutSpy = jest
        .spyOn(authService, 'logout')
        .mockResolvedValue(undefined);

      await request(app.getHttpServer())
        .post('/auth/logout')
        .set('Authorization', `Bearer valid-token`)
        .expect(200);

      expect(logoutSpy).toHaveBeenCalledTimes(1);
      expect(logoutSpy).toHaveBeenCalledWith(publicKey);
    });
  });

  describe('GET /auth/challenge', () => {
    it('delegates to NonceService.generateNonce', async () => {
      const nonceServiceSpy = jest
        .spyOn(nonceService, 'generateNonce')
        .mockResolvedValue('nonce-service-generated-nonce');

      const response = await request(app.getHttpServer())
        .get('/auth/challenge')
        .query({ publicKey })
        .expect(200);

      expect(response.body).toEqual({ nonce: 'nonce-service-generated-nonce' });
      expect(nonceServiceSpy).toHaveBeenCalledWith(publicKey);
    });

    it('no longer exposes the removed AuthService.generateChallenge (Issue #1490)', () => {
      expect(
        (authService as unknown as Record<string, unknown>).generateChallenge,
      ).toBeUndefined();
    });
  });
});
