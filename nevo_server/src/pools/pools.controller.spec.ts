import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication } from '@nestjs/common';
import request from 'supertest';
import { PoolsController } from './pools.controller';
import { PoolsService } from './pools.service';
import { JwtService } from '@nestjs/jwt';
import { DonationsService } from '../donations/donations.service';
import { ContractService } from '../contract/contract.service';
import { JwtStrategy } from '../auth/jwt.strategy';
import { UsersService } from '../users/users.service';
import { FilterPoolsDto } from './dto/filter-pools.dto';
import { CreatePoolDto } from './dto/create-pool.dto';
import { UpdatePoolDto } from './dto/update-pool.dto';
import { DonatePoolDto } from './dto/donate-pool.dto';

describe('PoolsController', () => {
  let controller: PoolsController;
  let service: PoolsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [PoolsController],
      providers: [
        {
          provide: PoolsService,
          useValue: {
            findAll: jest.fn().mockImplementation((query: FilterPoolsDto) =>
              Promise.resolve({
                data: [],
                total: 0,
                page: query.page ?? 1,
                limit: query.limit ?? 20,
              }),
            ),
            findOneMerged: jest.fn(),
            create: jest.fn(),
            updateMeta: jest.fn(),
            findByContractId: jest.fn(),
            buildWithdrawTx: jest.fn(),
            buildClosePoolTx: jest.fn(),
          },
        },
        {
          provide: ContractService,
          useValue: {
            buildDonateTransaction: jest.fn(),
            buildClosePoolTransaction: jest.fn(),
          },
        },
        {
          provide: JwtService,
          useValue: {},
        },
      ],
    }).compile();

    controller = module.get<PoolsController>(PoolsController);
    service = module.get<PoolsService>(PoolsService);
  });

  it('should be defined', () => {
    expect(controller).toBeDefined();
  });

  it('should call service.findAll with query parameters', async () => {
    const query: FilterPoolsDto = { page: 2, limit: 5, search: 'Water' };
    const spy = jest.spyOn(service, 'findAll');

    const result = await controller.findAll(query);

    expect(spy).toHaveBeenCalledWith(query);
    expect(result.page).toBe(2);
    expect(result.limit).toBe(5);
  });

  describe('findOne', () => {
    it('should return a pool when found', async () => {
      const mockPool = { contractPoolId: '1', title: 'Test Pool' };
      jest.spyOn(service, 'findOneMerged').mockResolvedValue(mockPool);

      const result = await controller.findOne('1');

      expect(result).toEqual(mockPool);
      expect(service.findOneMerged).toHaveBeenCalledWith('1');
    });

    it('should throw NotFoundException when pool not found', async () => {
      jest.spyOn(service, 'findOneMerged').mockResolvedValue(null);

      await expect(controller.findOne('999')).rejects.toThrow('Pool not found');
    });
  });

  describe('create', () => {
    it('should create a pool', async () => {
      const dto: CreatePoolDto = {
        contractPoolId: '1',
        creatorWallet: 'GCREATOR',
        title: 'Test Pool',
        description: 'Test Description',
        goal: '1000000000',
      };
      const mockPool = { contractPoolId: '1', ...dto };
      jest.spyOn(service, 'create').mockResolvedValue(mockPool);

      const result = await controller.create(dto);

      expect(result).toEqual(mockPool);
      expect(service.create).toHaveBeenCalledWith(dto);
    });
  });

  describe('updateMeta', () => {
    it('should update pool metadata', async () => {
      const dto: UpdatePoolDto = { description: 'Updated description' };
      const mockPool = {
        contractPoolId: '1',
        description: 'Updated description',
      };
      jest.spyOn(service, 'updateMeta').mockResolvedValue(mockPool);

      const result = await controller.updateMeta('1', dto);

      expect(result).toEqual(mockPool);
      expect(service.updateMeta).toHaveBeenCalledWith('1', dto);
    });

    it('should throw NotFoundException when pool not found', async () => {
      const dto: UpdatePoolDto = { description: 'Updated description' };
      jest.spyOn(service, 'updateMeta').mockResolvedValue(null);

      await expect(controller.updateMeta('999', dto)).rejects.toThrow(
        'Pool not found',
      );
    });
  });
});

describe('PoolsController (withdraw auth)', () => {
  let app: INestApplication;
  let poolsService: {
    findByContractId: jest.Mock;
    buildWithdrawTx: jest.Mock;
  };
  const jwtSecret = process.env.JWT_SECRET ?? 'dev-secret';
  const signToken = (sub: string) =>
    new JwtService({ secret: jwtSecret }).sign({ sub, tokenVersion: 0 });

  beforeEach(async () => {
    poolsService = {
      findByContractId: jest.fn(),
      buildWithdrawTx: jest.fn(),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [PoolsController],
      providers: [
        { provide: PoolsService, useValue: poolsService },
        { provide: DonationsService, useValue: {} },
        { provide: ContractService, useValue: {} },
        { provide: JwtService, useValue: {} },
        {
          provide: UsersService,
          useValue: {
            findByPublicKey: jest
              .fn()
              .mockImplementation((publicKey: string) =>
                Promise.resolve({ publicKey, tokenVersion: 0 }),
              ),
          },
        },
        JwtStrategy,
      ],
    }).compile();

    app = moduleRef.createNestApplication();
    await app.init();
  });

  afterEach(async () => {
    await app.close();
  });

  it('rejects an unauthenticated withdraw request', async () => {
    await request(app.getHttpServer()).post('/pools/1/withdraw').expect(401);

    expect(poolsService.findByContractId).not.toHaveBeenCalled();
  });

  it('derives the requester wallet from the JWT instead of the body', async () => {
    poolsService.findByContractId.mockResolvedValue({
      contractPoolId: '1',
      creatorWallet: 'GCREATOR',
    });
    poolsService.buildWithdrawTx.mockResolvedValue({
      unsignedXdr: 'xdr',
      poolId: '1',
    });
    const token = signToken('GCREATOR');

    await request(app.getHttpServer())
      .post('/pools/1/withdraw')
      .set('Authorization', `Bearer ${token}`)
      .send({ requesterWallet: 'GATTACKER' })
      .expect(201);

    expect(poolsService.buildWithdrawTx).toHaveBeenCalled();
  });

  it('forbids withdrawal when the JWT wallet is not the pool creator', async () => {
    poolsService.findByContractId.mockResolvedValue({
      contractPoolId: '1',
      creatorWallet: 'GCREATOR',
    });
    const token = signToken('GATTACKER');

    await request(app.getHttpServer())
      .post('/pools/1/withdraw')
      .set('Authorization', `Bearer ${token}`)
      .expect(403);

    expect(poolsService.buildWithdrawTx).not.toHaveBeenCalled();
  });

  it('returns 404 when pool does not exist for withdraw', async () => {
    poolsService.findByContractId.mockResolvedValue(null);
    const token = signToken('GCREATOR');

    await request(app.getHttpServer())
      .post('/pools/999/withdraw')
      .set('Authorization', `Bearer ${token}`)
      .expect(404);

    expect(poolsService.buildWithdrawTx).not.toHaveBeenCalled();
  });
});

describe('PoolsController (close auth)', () => {
  let app: INestApplication;
  let poolsService: {
    findByContractId: jest.Mock;
    buildClosePoolTx: jest.Mock;
  };
  const jwtSecret = process.env.JWT_SECRET ?? 'dev-secret';
  const signToken = (sub: string) =>
    new JwtService({ secret: jwtSecret }).sign({ sub, tokenVersion: 0 });

  beforeEach(async () => {
    poolsService = {
      findByContractId: jest.fn(),
      buildClosePoolTx: jest.fn(),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [PoolsController],
      providers: [
        { provide: PoolsService, useValue: poolsService },
        { provide: DonationsService, useValue: {} },
        { provide: ContractService, useValue: {} },
        { provide: JwtService, useValue: {} },
        {
          provide: UsersService,
          useValue: {
            findByPublicKey: jest
              .fn()
              .mockImplementation((publicKey: string) =>
                Promise.resolve({ publicKey, tokenVersion: 0 }),
              ),
          },
        },
        JwtStrategy,
      ],
    }).compile();

    app = moduleRef.createNestApplication();
    await app.init();
  });

  afterEach(async () => {
    await app.close();
  });

  it('rejects an unauthenticated close request', async () => {
    await request(app.getHttpServer()).post('/pools/1/close').expect(401);

    expect(poolsService.findByContractId).not.toHaveBeenCalled();
  });

  it('allows close when JWT wallet is the pool creator', async () => {
    poolsService.findByContractId.mockResolvedValue({
      contractPoolId: '1',
      creatorWallet: 'GCREATOR',
    });
    poolsService.buildClosePoolTx.mockResolvedValue({
      unsignedXdr: 'xdr',
    });
    const token = signToken('GCREATOR');

    await request(app.getHttpServer())
      .post('/pools/1/close')
      .set('Authorization', `Bearer ${token}`)
      .expect(201);

    expect(poolsService.buildClosePoolTx).toHaveBeenCalled();
  });

  it('forbids close when the JWT wallet is not the pool creator', async () => {
    poolsService.findByContractId.mockResolvedValue({
      contractPoolId: '1',
      creatorWallet: 'GCREATOR',
    });
    const token = signToken('GATTACKER');

    await request(app.getHttpServer())
      .post('/pools/1/close')
      .set('Authorization', `Bearer ${token}`)
      .expect(403);

    expect(poolsService.buildClosePoolTx).not.toHaveBeenCalled();
  });

  it('returns 404 when pool does not exist for close', async () => {
    poolsService.findByContractId.mockResolvedValue(null);
    const token = signToken('GCREATOR');

    await request(app.getHttpServer())
      .post('/pools/999/close')
      .set('Authorization', `Bearer ${token}`)
      .expect(404);

    expect(poolsService.buildClosePoolTx).not.toHaveBeenCalled();
  });
});

describe('PoolsController (donate auth)', () => {
  let app: INestApplication;
  let poolsService: {
    findByContractId: jest.Mock;
  };
  let contractService: {
    buildDonateTransaction: jest.Mock;
  };
  const jwtSecret = process.env.JWT_SECRET ?? 'dev-secret';
  const signToken = (sub: string) =>
    new JwtService({ secret: jwtSecret }).sign({ sub, tokenVersion: 0 });

  beforeEach(async () => {
    poolsService = {
      findByContractId: jest.fn(),
    };
    contractService = {
      buildDonateTransaction: jest.fn(),
    };

    const moduleRef: TestingModule = await Test.createTestingModule({
      controllers: [PoolsController],
      providers: [
        { provide: PoolsService, useValue: poolsService },
        { provide: DonationsService, useValue: {} },
        { provide: ContractService, useValue: contractService },
        { provide: JwtService, useValue: {} },
        {
          provide: UsersService,
          useValue: {
            findByPublicKey: jest
              .fn()
              .mockImplementation((publicKey: string) =>
                Promise.resolve({ publicKey, tokenVersion: 0 }),
              ),
          },
        },
        JwtStrategy,
      ],
    }).compile();

    app = moduleRef.createNestApplication();
    await app.init();
  });

  afterEach(async () => {
    await app.close();
  });

  it('rejects an unauthenticated donate request', async () => {
    await request(app.getHttpServer()).post('/pools/1/donate').expect(401);

    expect(poolsService.findByContractId).not.toHaveBeenCalled();
  });

  it('builds donation transaction for authenticated user', async () => {
    poolsService.findByContractId.mockResolvedValue({
      contractPoolId: '1',
      creatorWallet: 'GCREATOR',
      tokenAddress: 'GTOKEN',
    });
    contractService.buildDonateTransaction.mockResolvedValue('unsigned_xdr');
    const token = signToken('GDONOR');
    const dto: DonatePoolDto = { amount: '10000000' };

    await request(app.getHttpServer())
      .post('/pools/1/donate')
      .set('Authorization', `Bearer ${token}`)
      .send(dto)
      .expect(201)
      .expect({ unsignedXdr: 'unsigned_xdr' });

    expect(contractService.buildDonateTransaction).toHaveBeenCalledWith(
      'GDONOR',
      1,
      'GTOKEN',
      '10000000',
    );
  });

  it('returns 404 when pool does not exist for donate', async () => {
    poolsService.findByContractId.mockResolvedValue(null);
    const token = signToken('GDONOR');
    const dto: DonatePoolDto = { amount: '10000000' };

    await request(app.getHttpServer())
      .post('/pools/999/donate')
      .set('Authorization', `Bearer ${token}`)
      .send(dto)
      .expect(404);

    expect(contractService.buildDonateTransaction).not.toHaveBeenCalled();
  });
});