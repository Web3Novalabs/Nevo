import { INestApplication, ValidationPipe } from '@nestjs/common';
import { Test, TestingModule } from '@nestjs/testing';
import request from 'supertest';
import { AuthController } from '../auth.controller';
import { AuthService } from '../auth.service';
import { NonceService } from '../nonce.service';

/**
 * Pins down the POST /auth/verify body contract as it behaves under the global
 * ValidationPipe: which payloads are accepted, which are rejected, and that
 * unknown fields are stripped / forbidden.
 */
describe('VerifyAuthDto (POST /auth/verify body contract)', () => {
  let app: INestApplication;
  let verify: jest.Mock;

  /** A minimal valid payload that should always pass validation. */
  const VALID_PUBLIC_KEY = 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCW7M';
  const VALID_SIGNATURE = 'deadbeef01234567';
  const VALID_MESSAGE = 'some-nonce-value';

  const validBody = {
    publicKey: VALID_PUBLIC_KEY,
    signature: VALID_SIGNATURE,
    message: VALID_MESSAGE,
  };

  beforeEach(async () => {
    verify = jest.fn().mockResolvedValue({ accessToken: 'jwt', user: {} });

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [AuthController],
      providers: [
        { provide: AuthService, useValue: { verify } },
        { provide: NonceService, useValue: { generateNonce: jest.fn() } },
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
      .post('/auth/verify')
      .send(validBody)
      .expect(201);

    expect(verify).toHaveBeenCalledWith(validBody);
  });

  it('rejects a malformed publicKey (not a Stellar G... key) with 400', async () => {
    await request(app.getHttpServer())
      .post('/auth/verify')
      .send({ ...validBody, publicKey: 'not-a-stellar-key' })
      .expect(400);

    expect(verify).not.toHaveBeenCalled();
  });

  it('rejects a publicKey that starts with G but is wrong length with 400', async () => {
    await request(app.getHttpServer())
      .post('/auth/verify')
      .send({ ...validBody, publicKey: 'GSHORT' })
      .expect(400);

    expect(verify).not.toHaveBeenCalled();
  });

  it('rejects a non-hex signature with 400', async () => {
    await request(app.getHttpServer())
      .post('/auth/verify')
      .send({ ...validBody, signature: 'not-hex-!@#$' })
      .expect(400);

    expect(verify).not.toHaveBeenCalled();
  });

  it('rejects an empty message with 400', async () => {
    await request(app.getHttpServer())
      .post('/auth/verify')
      .send({ ...validBody, message: '' })
      .expect(400);

    expect(verify).not.toHaveBeenCalled();
  });

  it('rejects a missing publicKey with 400', async () => {
    const { publicKey: _omitted, ...bodyWithoutKey } = validBody;
    await request(app.getHttpServer())
      .post('/auth/verify')
      .send(bodyWithoutKey)
      .expect(400);

    expect(verify).not.toHaveBeenCalled();
  });

  it('rejects a missing signature with 400', async () => {
    const { signature: _omitted, ...bodyWithoutSig } = validBody;
    await request(app.getHttpServer())
      .post('/auth/verify')
      .send(bodyWithoutSig)
      .expect(400);

    expect(verify).not.toHaveBeenCalled();
  });

  it('rejects unknown extra fields under forbidNonWhitelisted with 400', async () => {
    await request(app.getHttpServer())
      .post('/auth/verify')
      .send({ ...validBody, extraField: 'surprise' })
      .expect(400);

    expect(verify).not.toHaveBeenCalled();
  });
});
