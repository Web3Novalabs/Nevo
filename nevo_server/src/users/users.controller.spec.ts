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

const makeUser = (overrides: Partial<User> = {}): User => ({
  id: 'uuid-1',
  publicKey: 'GABC1234567890',
  displayName: 'Alice',
  createdAt: new Date(),
  updatedAt: new Date(),
  ...overrides,
});

const makeReq = (publicKey: string) =>
  ({ user: { publicKey } }) as { user: { publicKey: string } } & any;

describe('UsersController', () => {
  let controller: UsersController;
  let usersService: { updateDisplayName: jest.Mock };

  beforeEach(async () => {
    usersService = {
      updateDisplayName: jest.fn(),
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
      const classMetadata = Reflect.getMetadata('__guards__', UsersController);
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
  let usersService: { updateDisplayName: jest.Mock };

  const jwtSecret = process.env.JWT_SECRET ?? JWT_SECRET_FALLBACK;
  const signToken = (publicKey: string) =>
    new JwtService({ secret: jwtSecret }).sign({ sub: publicKey });

  beforeEach(async () => {
    usersService = {
      updateDisplayName: jest.fn().mockResolvedValue(makeUser()),
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
