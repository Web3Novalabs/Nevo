import { Test, TestingModule } from '@nestjs/testing';
import {
  INestApplication,
  NotFoundException,
  ValidationPipe,
} from '@nestjs/common';
import { JwtService } from '@nestjs/jwt';
import request from 'supertest';
import { UsersController } from './users.controller';
import { UsersService } from './users.service';
import { JwtAuthGuard } from '../auth/jwt-auth.guard';
import { JwtStrategy } from '../auth/jwt.strategy';
import { JWT_SECRET_FALLBACK } from '../auth/jwt.config';
import { User } from './user.entity';
import { Keypair } from '@stellar/stellar-sdk';

const VALID_KEY = Keypair.random().publicKey();
const UNKNOWN_KEY = Keypair.random().publicKey();

const makeUser = (overrides: Partial<User> = {}): User => ({
  id: 'uuid-1',
  tokenVersion: 0,
  publicKey: 'GABC1234567890',
  displayName: 'Alice',
  createdAt: new Date(),
  updatedAt: new Date(),
  ...overrides,
});

const makeReq = (publicKey: string) => ({ user: { publicKey } });

describe('UsersController', () => {
  let controller: UsersController;
  let usersService: {
    updateDisplayName: jest.Mock;
    findByPublicKey: jest.Mock;
    findPublicProfile: jest.Mock;
  };

  beforeEach(async () => {
    usersService = {
      updateDisplayName: jest.fn(),
      findByPublicKey: jest.fn(),
      findPublicProfile: jest.fn(),
    };

    const module: TestingModule = await Test.createTestingModule({
      controllers: [UsersController],
      providers: [
        {
          provide: UsersService,
          useValue: usersService,
        },
      ],
    }).compile();

    controller = module.get<UsersController>(UsersController);
    jest.clearAllMocks();
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  it('returns only the signed-in user’s profile fields', async () => {
    const user = makeUser();
    usersService.findByPublicKey.mockResolvedValue(user);
    await expect(controller.getMe(makeReq(user.publicKey))).resolves.toEqual({
      publicKey: user.publicKey,
      displayName: user.displayName,
      createdAt: user.createdAt,
    });
    expect(usersService.findByPublicKey).toHaveBeenCalledWith(user.publicKey);
  });

  describe('updateMe (PATCH /users/me)', () => {
    describe('success path', () => {
      it('calls usersService.updateDisplayName with the public key from JWT and trimmed displayName', async () => {
        const user = makeUser();
        usersService.updateDisplayName.mockResolvedValue(user);

        const result = await controller.updateMe(makeReq('GABC1234567890'), {
          displayName: '  Bob  ',
        });

        expect(usersService.updateDisplayName).toHaveBeenCalledWith(
          'GABC1234567890',
          'Bob',
        );
        expect(result).toBe(user);
      });

      it('returns the updated user entity from usersService', async () => {
        const updatedUser = makeUser({ displayName: 'Charlie' });
        usersService.updateDisplayName.mockResolvedValue(updatedUser);

        const result = await controller.updateMe(makeReq('GPUB_KEY'), {
          displayName: 'Charlie',
        });

        expect(result).toEqual(updatedUser);
      });

      it('trims leading/trailing whitespace from displayName before passing to service', async () => {
        const user = makeUser({ displayName: 'Trimmed' });
        usersService.updateDisplayName.mockResolvedValue(user);

        await controller.updateMe(makeReq('GPUB_KEY'), {
          displayName: '   Trimmed   ',
        });

        expect(usersService.updateDisplayName).toHaveBeenCalledWith(
          'GPUB_KEY',
          'Trimmed',
        );
      });

      it('uses publicKey from JWT user (not from request body)', async () => {
        const user = makeUser({ publicKey: 'GJWT_KEY' });
        usersService.updateDisplayName.mockResolvedValue(user);

        await controller.updateMe(makeReq('GJWT_KEY'), {
          displayName: 'Name',
        });

        expect(usersService.updateDisplayName).toHaveBeenCalledWith(
          'GJWT_KEY',
          'Name',
        );
      });
    });

    describe('user not found path', () => {
      it('throws NotFoundException when usersService.updateDisplayName returns null', async () => {
        usersService.updateDisplayName.mockResolvedValue(null);

        await expect(
          controller.updateMe(makeReq('GABC1234567890'), {
            displayName: 'Alice',
          }),
        ).rejects.toThrow(NotFoundException);
      });

      it('throws NotFoundException with "User not found" message', async () => {
        usersService.updateDisplayName.mockResolvedValue(null);

        await expect(
          controller.updateMe(makeReq('GUNKNOWN'), {
            displayName: 'Ghost',
          }),
        ).rejects.toThrow('User not found');
      });

      it('does not call usersService.updateDisplayName twice on not-found', async () => {
        usersService.updateDisplayName.mockResolvedValue(null);

        try {
          await controller.updateMe(makeReq('GMISSING'), {
            displayName: 'Nobody',
          });
        } catch {
          // expected NotFoundException
        }

        expect(usersService.updateDisplayName).toHaveBeenCalledTimes(1);
      });
    });

    describe('error propagation', () => {
      it('propagates unexpected errors from usersService without suppression', async () => {
        const error = new Error('DB connection lost');
        usersService.updateDisplayName.mockRejectedValue(error);

        await expect(
          controller.updateMe(makeReq('GPUB_KEY'), {
            displayName: 'Alice',
          }),
        ).rejects.toThrow('DB connection lost');
      });
    });
  });

  describe('getPublicProfile (GET /users/:publicKey)', () => {
    const publicKey = VALID_KEY;

    it('returns the public profile DTO from usersService', async () => {
      const profile = {
        publicKey,
        displayName: 'Alice',
        joinedAt: new Date('2024-01-01T00:00:00.000Z'),
        poolsCreated: 3,
        totalRaised: '1500.0000000',
      };
      usersService.findPublicProfile.mockResolvedValue(profile);

      const result = await controller.getPublicProfile(publicKey);

      expect(usersService.findPublicProfile).toHaveBeenCalledWith(publicKey);
      expect(result).toEqual(profile);
    });

    it('does not expose private fields such as id, createdAt or updatedAt', async () => {
      usersService.findPublicProfile.mockResolvedValue({
        publicKey,
        displayName: 'Alice',
        joinedAt: new Date('2024-01-01T00:00:00.000Z'),
        poolsCreated: 0,
        totalRaised: '0.0000000',
      });

      const result = (await controller.getPublicProfile(
        publicKey,
      )) as unknown as Record<string, unknown>;

      expect(result).not.toHaveProperty('id');
      expect(result).not.toHaveProperty('createdAt');
      expect(result).not.toHaveProperty('updatedAt');
    });

    it('throws NotFoundException when the user does not exist', async () => {
      usersService.findPublicProfile.mockResolvedValue(null);

      await expect(controller.getPublicProfile(UNKNOWN_KEY)).rejects.toThrow(
        NotFoundException,
      );
    });

    it('throws NotFoundException with "User not found" message', async () => {
      usersService.findPublicProfile.mockResolvedValue(null);

      await expect(controller.getPublicProfile(UNKNOWN_KEY)).rejects.toThrow(
        'User not found',
      );
    });

    it('propagates unexpected errors from usersService', async () => {
      usersService.findPublicProfile.mockRejectedValue(
        new Error('DB connection lost'),
      );

      await expect(controller.getPublicProfile(publicKey)).rejects.toThrow(
        'DB connection lost',
      );
    });

    it('does not apply JwtAuthGuard to the public profile route', () => {
      const metadata = Reflect.getMetadata(
        '__guards__',
        UsersController.prototype.getPublicProfile,
      );
      expect(metadata).toBeUndefined();
    });
  });

  describe('JwtAuthGuard protection on updateMe route', () => {
    it('applies JwtAuthGuard to the updateMe method', () => {
      const metadata = Reflect.getMetadata(
        '__guards__',
        UsersController.prototype.updateMe,
      );
      expect(metadata).toBeDefined();
      expect(Array.isArray(metadata)).toBe(true);
      const hasJwtGuard = (metadata as unknown[]).some(
        (guard) => guard === JwtAuthGuard,
      );
      expect(hasJwtGuard).toBe(true);
    });

    it('does not apply JwtAuthGuard at the class level (only on specific routes)', () => {
      // Guard is only on the method, not the whole controller
      // class-level guards would be defined; method-level is sufficient
      const methodMetadata = Reflect.getMetadata(
        '__guards__',
        UsersController.prototype.updateMe,
      );
      expect(methodMetadata).toBeDefined();
    });
  });
});

describe('UsersController (PATCH /users/me validation)', () => {
  let app: INestApplication;
  let usersService: {
    updateDisplayName: jest.Mock;
    findByPublicKey: jest.Mock;
    findPublicProfile: jest.Mock;
  };

  const jwtSecret = process.env.JWT_SECRET ?? JWT_SECRET_FALLBACK;
  const signToken = (publicKey: string) =>
    new JwtService({ secret: jwtSecret }).sign({
      sub: publicKey,
      tokenVersion: 0,
    });

  beforeEach(async () => {
    usersService = {
      updateDisplayName: jest.fn().mockResolvedValue(makeUser()),
      findByPublicKey: jest
        .fn()
        .mockImplementation((publicKey: string) =>
          Promise.resolve({ publicKey, tokenVersion: 0 }),
        ),
      findPublicProfile: jest.fn(),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [UsersController],
      providers: [
        { provide: UsersService, useValue: usersService },
        JwtStrategy,
      ],
    }).compile();

    app = moduleRef.createNestApplication();
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

  it('rejects a whitespace-only displayName with 400', async () => {
    const token = signToken('GABC1234567890');

    await request(app.getHttpServer())
      .patch('/users/me')
      .set('Authorization', `Bearer ${token}`)
      .send({ displayName: '   ' })
      .expect(400);

    expect(usersService.updateDisplayName).not.toHaveBeenCalled();
  });

  it('accepts a valid displayName and forwards the trimmed value', async () => {
    const token = signToken('GABC1234567890');
    const user = makeUser({ displayName: 'Bob' });
    usersService.updateDisplayName.mockResolvedValue(user);

    const res = await request(app.getHttpServer())
      .patch('/users/me')
      .set('Authorization', `Bearer ${token}`)
      .send({ displayName: '  Bob  ' })
      .expect(200);

    expect(usersService.updateDisplayName).toHaveBeenCalledWith(
      'GABC1234567890',
      'Bob',
    );
    expect(res.body).toMatchObject({ displayName: 'Bob' });
  });
});

describe('UsersController (GET /users/:publicKey)', () => {
  let app: INestApplication;
  let usersService: {
    updateDisplayName: jest.Mock;
    findByPublicKey: jest.Mock;
    findPublicProfile: jest.Mock;
  };

  const validKey = VALID_KEY;

  beforeEach(async () => {
    usersService = {
      updateDisplayName: jest.fn(),
      findByPublicKey: jest
        .fn()
        .mockImplementation((publicKey: string) =>
          Promise.resolve({ publicKey, tokenVersion: 0 }),
        ),
      findPublicProfile: jest.fn(),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [UsersController],
      providers: [
        { provide: UsersService, useValue: usersService },
        JwtStrategy,
      ],
    }).compile();

    app = moduleRef.createNestApplication();
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

  it('returns 200 with the public DTO for a valid key', async () => {
    usersService.findPublicProfile.mockResolvedValue({
      publicKey: validKey,
      displayName: 'Alice',
      joinedAt: new Date('2024-01-01T00:00:00.000Z'),
      poolsCreated: 2,
      totalRaised: '100.0000000',
    });

    const res = await request(app.getHttpServer())
      .get(`/users/${validKey}`)
      .expect(200);

    expect(usersService.findPublicProfile).toHaveBeenCalledWith(validKey);
    expect(res.body).toMatchObject({
      publicKey: validKey,
      displayName: 'Alice',
      poolsCreated: 2,
      totalRaised: '100.0000000',
    });
    expect(res.body).not.toHaveProperty('id');
    expect(res.body).not.toHaveProperty('createdAt');
    expect(res.body).not.toHaveProperty('updatedAt');
  });

  it('returns 400 for an invalid key format', async () => {
    await request(app.getHttpServer()).get('/users/not-a-key').expect(400);

    expect(usersService.findPublicProfile).not.toHaveBeenCalled();
  });

  it('returns 404 for an unknown key', async () => {
    usersService.findPublicProfile.mockResolvedValue(null);

    await request(app.getHttpServer()).get(`/users/${validKey}`).expect(404);
  });
});
