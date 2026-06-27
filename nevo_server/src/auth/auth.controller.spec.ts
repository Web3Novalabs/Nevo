import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
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
});
