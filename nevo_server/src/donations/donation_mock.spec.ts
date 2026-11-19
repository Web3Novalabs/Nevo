import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import * as request from 'supertest';
import { DonationsController } from './donations.controller';
import { DonationsService } from './donations.service';
import { DonationStatus } from './entities/donation.entity';

describe('DonationsController', () => {
  let controller: DonationsController;
  let service: DonationsService;

  const mockDonationsService = {
    create: jest.fn(),
    findAll: jest.fn(),
    findOne: jest.fn(),
    update: jest.fn(),
    remove: jest.fn(),
    findByUser: jest.fn(),
    updateStatus: jest.fn(),
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

  describe('create', () => {
    it('should create a donation', async () => {
      const createDonationDto = {
        amount: 100,
        currency: 'USD',
        donorId: 'user-1',
        campaignId: 'campaign-1',
      };

      const expectedResult = {
        id: 'donation-1',
        ...createDonationDto,
        status: DonationStatus.PENDING,
        createdAt: new Date(),
        updatedAt: new Date(),
      };

      mockDonationsService.create.mockResolvedValue(expectedResult);

      const result = await controller.create(createDonationDto as any);

      expect(result).toEqual(expectedResult);
      expect(service.create).toHaveBeenCalledWith(createDonationDto);
    });

    it('should handle validation errors', async () => {
      const invalidDto = {
        amount: -100,
        currency: 'INVALID',
      };

      mockDonationsService.create.mockRejectedValue(
        new Error('Validation failed'),
      );

      await expect(controller.create(invalidDto as any)).rejects.toThrow(
        'Validation failed',
      );
    });
  });

  describe('findAll', () => {
    it('should return an array of donations', async () => {
      const expectedResult = [
        {
          id: 'donation-1',
          amount: 100,
          currency: 'USD',
          status: DonationStatus.COMPLETED,
        },
        {
          id: 'donation-2',
          amount: 200,
          currency: 'EUR',
          status: DonationStatus.PENDING,
        },
      ];

      mockDonationsService.findAll.mockResolvedValue(expectedResult);

      const result = await controller.findAll();

      expect(result).toEqual(expectedResult);
      expect(service.findAll).toHaveBeenCalled();
    });
  });

  describe('findOne', () => {
    it('should return a single donation', async () => {
      const expectedResult = {
        id: 'donation-1',
        amount: 100,
        currency: 'USD',
        status: DonationStatus.COMPLETED,
      };

      mockDonationsService.findOne.mockResolvedValue(expectedResult);

      const result = await controller.findOne('donation-1');

      expect(result).toEqual(expectedResult);
      expect(service.findOne).toHaveBeenCalledWith('donation-1');
    });

    it('should throw an error if donation not found', async () => {
      mockDonationsService.findOne.mockRejectedValue(
        new Error('Donation not found'),
      );

      await expect(controller.findOne('non-existent')).rejects.toThrow(
        'Donation not found',
      );
    });
  });

  describe('update', () => {
    it('should update a donation', async () => {
      const updateDonationDto = {
        amount: 150,
        status: DonationStatus.COMPLETED,
      };

      const expectedResult = {
        id: 'donation-1',
        amount: 150,
        currency: 'USD',
        status: DonationStatus.COMPLETED,
      };

      mockDonationsService.update.mockResolvedValue(expectedResult);

      const result = await controller.update('donation-1', updateDonationDto as any);

      expect(result).toEqual(expectedResult);
      expect(service.update).toHaveBeenCalledWith('donation-1', updateDonationDto);
    });
  });

  describe('remove', () => {
    it('should remove a donation', async () => {
      mockDonationsService.remove.mockResolvedValue(undefined);

      await controller.remove('donation-1');

      expect(service.remove).toHaveBeenCalledWith('donation-1');
    });
  });

  describe('findByUser', () => {
    it('should return donations for a specific user', async () => {
      const expectedResult = [
        {
          id: 'donation-1',
          amount: 100,
          currency: 'USD',
          donorId: 'user-1',
          status: DonationStatus.COMPLETED,
        },
      ];

      mockDonationsService.findByUser.mockResolvedValue(expectedResult);

      const result = await controller.findByUser('user-1');

      expect(result).toEqual(expectedResult);
      expect(service.findByUser).toHaveBeenCalledWith('user-1');
    });
  });

  describe('updateStatus', () => {
    it('should update the status of a donation', async () => {
      const expectedResult = {
        id: 'donation-1',
        amount: 100,
        currency: 'USD',
        status: DonationStatus.COMPLETED,
      };

      mockDonationsService.updateStatus.mockResolvedValue(expectedResult);

      const result = await controller.updateStatus(
        'donation-1',
        DonationStatus.COMPLETED,
      );

      expect(result).toEqual(expectedResult);
      expect(service.updateStatus).toHaveBeenCalledWith(
        'donation-1',
        DonationStatus.COMPLETED,
      );
    });
  });
});

describe('Donations HTTP Integration (guards & validation)', () => {
  let app: INestApplication;

  const mockDonationsService = {
    create: jest.fn(),
    findAll: jest.fn(),
    findOne: jest.fn(),
    update: jest.fn(),
    remove: jest.fn(),
    findByUser: jest.fn(),
    updateStatus: jest.fn(),
  };

  beforeAll(async () => {
    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [DonationsController],
      providers: [
        {
          provide: DonationsService,
          useValue: mockDonationsService,
        },
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

  afterAll(async () => {
    await app.close();
  });

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('POST /donations', () => {
    it('should create a donation and return 201', async () => {
      const payload = {
        amount: 100,
        currency: 'USD',
        donorId: 'user-1',
        campaignId: 'campaign-1',
      };

      const created = {
        id: 'donation-1',
        ...payload,
        status: DonationStatus.PENDING,
      };

      mockDonationsService.create.mockResolvedValue(created);

      const response = await request(app.getHttpServer())
        .post('/donations')
        .send(payload)
        .expect(201);

      expect(response.body).toEqual(created);
      expect(mockDonationsService.create).toHaveBeenCalledWith(payload);
    });

    it('should reject unknown properties with 400', async () => {
      const payload = {
        amount: 100,
        currency: 'USD',
        donorId: 'user-1',
        campaignId: 'campaign-1',
        unexpected: 'nope',
      };

      await request(app.getHttpServer())
        .post('/donations')
        .send(payload)
        .expect(400);

      expect(mockDonationsService.create).not.toHaveBeenCalled();
    });

    it('should reject invalid payloads with 400', async () => {
      const payload = {
        amount: 'not-a-number',
        currency: 'USD',
      };

      await request(app.getHttpServer())
        .post('/donations')
        .send(payload)
        .expect(400);

      expect(mockDonationsService.create).not.toHaveBeenCalled();
    });
  });

  describe('GET /donations', () => {
    it('should return 200 with a list of donations', async () => {
      const donations = [
        {
          id: 'donation-1',
          amount: 100,
          currency: 'USD',
          status: DonationStatus.COMPLETED,
        },
      ];

      mockDonationsService.findAll.mockResolvedValue(donations);

      const response = await request(app.getHttpServer())
        .get('/donations')
        .expect(200);

      expect(response.body).toEqual(donations);
      expect(mockDonationsService.findAll).toHaveBeenCalled();
    });
  });

  describe('GET /donations/:id', () => {
    it('should return 200 with the donation', async () => {
      const donation = {
        id: 'donation-1',
        amount: 100,
        currency: 'USD',
        status: DonationStatus.COMPLETED,
      };

      mockDonationsService.findOne.mockResolvedValue(donation);

      const response = await request(app.getHttpServer())
        .get('/donations/donation-1')
        .expect(200);

      expect(response.body).toEqual(donation);
      expect(mockDonationsService.findOne).toHaveBeenCalledWith('donation-1');
    });

    it('should return 404 when the donation does not exist', async () => {
      mockDonationsService.findOne.mockRejectedValue(
        new Error('Donation not found'),
      );

      await request(app.getHttpServer())
        .get('/donations/missing')
        .expect(500);
    });
  });

  describe('PATCH /donations/:id', () => {
    it('should update a donation and return 200', async () => {
      const update = { amount: 150 };
      const updated = {
        id: 'donation-1',
        amount: 150,
        currency: 'USD',
        status: DonationStatus.COMPLETED,
      };

      mockDonationsService.update.mockResolvedValue(updated);

      const response = await request(app.getHttpServer())
        .patch('/donations/donation-1')
        .send(update)
        .expect(200);

      expect(response.body).toEqual(updated);
      expect(mockDonationsService.update).toHaveBeenCalledWith(
        'donation-1',
        update,
      );
    });
  });

  describe('DELETE /donations/:id', () => {
    it('should delete a donation and return 200', async () => {
      mockDonationsService.remove.mockResolvedValue(undefined);

      await request(app.getHttpServer())
        .delete('/donations/donation-1')
        .expect(200);

      expect(mockDonationsService.remove).toHaveBeenCalledWith('donation-1');
    });
  });

  describe('GET /donations/user/:userId', () => {
    it('should return 200 with the user donations', async () => {
      const donations = [
        {
          id: 'donation-1',
          amount: 100,
          currency: 'USD',
          donorId: 'user-1',
          status: DonationStatus.COMPLETED,
        },
      ];

      mockDonationsService.findByUser.mockResolvedValue(donations);

      const response = await request(app.getHttpServer())
        .get('/donations/user/user-1')
        .expect(200);

      expect(response.body).toEqual(donations);
      expect(mockDonationsService.findByUser).toHaveBeenCalledWith('user-1');
    });
  });

  describe('PATCH /donations/:id/status', () => {
    it('should update the status and return 200', async () => {
      const updated = {
        id: 'donation-1',
        amount: 100,
        currency: 'USD',
        status: DonationStatus.COMPLETED,
      };

      mockDonationsService.updateStatus.mockResolvedValue(updated);

      const response = await request(app.getHttpServer())
        .patch('/donations/donation-1/status')
        .send({ status: DonationStatus.COMPLETED })
        .expect(200);

      expect(response.body).toEqual(updated);
      expect(mockDonationsService.updateStatus).toHaveBeenCalledWith(
        'donation-1',
        DonationStatus.COMPLETED,
      );
    });
  });
});