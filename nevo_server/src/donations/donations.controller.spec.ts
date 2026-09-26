import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import request from 'supertest';
import { JwtService } from '@nestjs/jwt';
import { DonationsController } from './donations.controller';
import { DonationSortBy, DonationsService } from './donations.service';
import { JwtStrategy } from '../auth/jwt.strategy';
import { JWT_SECRET_FALLBACK } from '../auth/jwt.config';

// ---------------------------------------------------------------------------
// Unit-level tests – exercise the controller methods directly without HTTP
// ---------------------------------------------------------------------------
describe('DonationsController (unit)', () => {
  let controller: DonationsController;
  let service: DonationsService;

  const mockDonationsService = {
    findByPool: jest
      .fn()
      .mockImplementation((poolId: string, sort: string) =>
        Promise.resolve([{ id: 'don-1', poolId, sort }]),
      ),
    findByDonor: jest
      .fn()
      .mockImplementation((donor: string, sort: string) =>
        Promise.resolve([{ id: 'don-1', donor, sort }]),
      ),
  };

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [DonationsController],
      providers: [
        {
          provide: DonationsService,
          useValue: mockDonationsService,
        },
      ],
    }).compile();

    controller = module.get<DonationsController>(DonationsController);
    service = module.get<DonationsService>(DonationsService);
    jest.clearAllMocks();
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  // -------------------------------------------------------------------------
  // findByPool
  // -------------------------------------------------------------------------
  describe('findByPool (GET pools/:id/donations)', () => {
    it('calls service.findByPool with pool id and default sort ("newest")', async () => {
      const result = await controller.findByPool('pool-123', {});
      expect(service.findByPool).toHaveBeenCalledWith(
        'pool-123',
        DonationSortBy.newest,
        undefined,
        undefined,
      );
      expect(result).toEqual([
        { id: 'don-1', poolId: 'pool-123', sort: 'newest' },
      ]);
    });

    it('calls service.findByPool with sortBy="largest" when query param is passed', async () => {
      const result = await controller.findByPool('pool-123', {
        sortBy: DonationSortBy.largest,
      });
      expect(service.findByPool).toHaveBeenCalledWith(
        'pool-123',
        DonationSortBy.largest,
        undefined,
        undefined,
      );
      expect(result).toEqual([
        { id: 'don-1', poolId: 'pool-123', sort: 'largest' },
      ]);
    });

    it('forwards pagination params to the service', async () => {
      await controller.findByPool('pool-123', { page: 2, limit: 5 });
      expect(service.findByPool).toHaveBeenCalledWith(
        'pool-123',
        DonationSortBy.newest,
        2,
        5,
      );
    });

    it('falls back to "newest" when an unrecognised sortBy is supplied', async () => {
      await controller.findByPool('pool-123', { sortBy: 'unknown' as any });
      expect(service.findByPool).toHaveBeenCalledWith(
        'pool-123',
        DonationSortBy.newest,
        undefined,
        undefined,
      );
    });
  });

  // -------------------------------------------------------------------------
  // findMyDonations
  // -------------------------------------------------------------------------
  describe('findMyDonations (GET users/me/donations)', () => {
    it('calls service.findByDonor with the authenticated user publicKey', async () => {
      const req = { user: { publicKey: 'GABC123' } } as any;
      const result = await controller.findMyDonations(req, {});
      expect(service.findByDonor).toHaveBeenCalledWith(
        'GABC123',
        DonationSortBy.newest,
        undefined,
        undefined,
      );
      expect(result).toEqual([
        { id: 'don-1', donor: 'GABC123', sort: 'newest' },
      ]);
    });

    it('calls service.findByDonor with sortBy="largest" when query param is passed', async () => {
      const req = { user: { publicKey: 'GABC123' } } as any;
      await controller.findMyDonations(req, {
        sortBy: DonationSortBy.largest,
      });
      expect(service.findByDonor).toHaveBeenCalledWith(
        'GABC123',
        DonationSortBy.largest,
        undefined,
        undefined,
      );
    });

    it('forwards pagination params to the service', async () => {
      const req = { user: { publicKey: 'GABC123' } } as any;
      await controller.findMyDonations(req, { page: 3, limit: 10 });
      expect(service.findByDonor).toHaveBeenCalledWith(
        'GABC123',
        DonationSortBy.newest,
        3,
        10,
      );
    });
  });
});

// ---------------------------------------------------------------------------
// HTTP-level tests – spin up a real NestJS app to verify the JwtAuthGuard
// ---------------------------------------------------------------------------
describe('DonationsController (guard / HTTP)', () => {
  let app: INestApplication;
  let donationsService: { findByPool: jest.Mock; findByDonor: jest.Mock };

  const jwtSecret = process.env.JWT_SECRET ?? JWT_SECRET_FALLBACK;
  const signToken = (publicKey: string) =>
    new JwtService({ secret: jwtSecret }).sign({ sub: publicKey });

  beforeEach(async () => {
    donationsService = {
      findByPool: jest.fn().mockResolvedValue([]),
      findByDonor: jest.fn().mockResolvedValue([]),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [DonationsController],
      providers: [
        { provide: DonationsService, useValue: donationsService },
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

  // -------------------------------------------------------------------------
  // GET /pools/:id/donations — public, no guard
  // -------------------------------------------------------------------------
  describe('GET /pools/:id/donations (public)', () => {
    it('returns 200 without any Authorization header', async () => {
      await request(app.getHttpServer())
        .get('/pools/pool-1/donations')
        .expect(200);

      expect(donationsService.findByPool).toHaveBeenCalledWith(
        'pool-1',
        DonationSortBy.newest,
        undefined,
        undefined,
      );
    });

    it('returns 200 with sortBy=largest query param', async () => {
      await request(app.getHttpServer())
        .get('/pools/pool-1/donations')
        .query({ sortBy: 'largest' })
        .expect(200);

      expect(donationsService.findByPool).toHaveBeenCalledWith(
        'pool-1',
        DonationSortBy.largest,
        undefined,
        undefined,
      );
    });

    it('forwards page and limit to the service', async () => {
      await request(app.getHttpServer())
        .get('/pools/pool-1/donations')
        .query({ page: '2', limit: '5' })
        .expect(200);

      expect(donationsService.findByPool).toHaveBeenCalledWith(
        'pool-1',
        DonationSortBy.newest,
        '2',
        '5',
      );
    });
  });

  // -------------------------------------------------------------------------
  // GET /users/me/donations — protected by JwtAuthGuard
  // -------------------------------------------------------------------------
  describe('GET /users/me/donations (JwtAuthGuard)', () => {
    it('returns 401 when no Authorization header is provided', async () => {
      await request(app.getHttpServer())
        .get('/users/me/donations')
        .expect(401);

      // Service must never be reached
      expect(donationsService.findByDonor).not.toHaveBeenCalled();
    });

    it('returns 401 when an invalid / malformed token is provided', async () => {
      await request(app.getHttpServer())
        .get('/users/me/donations')
        .set('Authorization', 'Bearer not-a-real-token')
        .expect(401);

      expect(donationsService.findByDonor).not.toHaveBeenCalled();
    });

    it('returns 200 and calls findByDonor when a valid JWT is provided', async () => {
      const publicKey = 'GABC123XYZ';
      const token = signToken(publicKey);

      await request(app.getHttpServer())
        .get('/users/me/donations')
        .set('Authorization', `Bearer ${token}`)
        .expect(200);

      expect(donationsService.findByDonor).toHaveBeenCalledWith(
        publicKey,
        DonationSortBy.newest,
        undefined,
        undefined,
      );
    });

    it('returns 200 with sortBy=largest for an authenticated user', async () => {
      const publicKey = 'GABC123XYZ';
      const token = signToken(publicKey);

      await request(app.getHttpServer())
        .get('/users/me/donations')
        .set('Authorization', `Bearer ${token}`)
        .query({ sortBy: 'largest' })
        .expect(200);

      expect(donationsService.findByDonor).toHaveBeenCalledWith(
        publicKey,
        DonationSortBy.largest,
        undefined,
        undefined,
      );
    });

    it('forwards pagination params for an authenticated user', async () => {
      const publicKey = 'GABC123XYZ';
      const token = signToken(publicKey);

      await request(app.getHttpServer())
        .get('/users/me/donations')
        .set('Authorization', `Bearer ${token}`)
        .query({ page: '2', limit: '10' })
        .expect(200);

      expect(donationsService.findByDonor).toHaveBeenCalledWith(
        publicKey,
        DonationSortBy.newest,
        '2',
        '10',
      );
    });
  });
});

// ---------------------------------------------------------------------------
// ValidationPipe tests – mirror the global pipe configured in main.ts
// ---------------------------------------------------------------------------
describe('DonationsController (ValidationPipe)', () => {
  let app: INestApplication;
  let donationsService: { findByPool: jest.Mock; findByDonor: jest.Mock };

  beforeEach(async () => {
    donationsService = {
      findByPool: jest.fn().mockResolvedValue([]),
      findByDonor: jest.fn().mockResolvedValue([]),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [DonationsController],
      providers: [
        { provide: DonationsService, useValue: donationsService },
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

  it('accepts page, limit and sortBy and coerces pagination to numbers', async () => {
    await request(app.getHttpServer())
      .get('/pools/pool-1/donations')
      .query({ page: '2', limit: '5', sortBy: 'largest' })
      .expect(200);

    expect(donationsService.findByPool).toHaveBeenCalledWith(
      'pool-1',
      DonationSortBy.largest,
      2,
      5,
    );
  });

  it('returns 400 when limit exceeds 100', async () => {
    await request(app.getHttpServer())
      .get('/pools/pool-1/donations')
      .query({ limit: '101' })
      .expect(400);

    expect(donationsService.findByPool).not.toHaveBeenCalled();
  });

  it('returns 400 for an unknown sortBy value', async () => {
    await request(app.getHttpServer())
      .get('/pools/pool-1/donations')
      .query({ sortBy: 'oldest' })
      .expect(400);

    expect(donationsService.findByPool).not.toHaveBeenCalled();
  });

  it('returns 400 for an unknown query param', async () => {
    await request(app.getHttpServer())
      .get('/pools/pool-1/donations')
      .query({ foo: 'bar' })
      .expect(400);
  });
});
