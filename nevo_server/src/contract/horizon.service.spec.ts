import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { HorizonService } from './horizon.service.js';
import { HttpException } from '@nestjs/common';

describe('HorizonService', () => {
  let service: HorizonService;
  let mockConfigService: jest.Mocked<ConfigService>;
  let originalFetch: typeof global.fetch;

  beforeAll(() => {
    originalFetch = global.fetch;
  });

  afterAll(() => {
    global.fetch = originalFetch;
  });

  beforeEach(async () => {
    mockConfigService = {
      get: jest.fn().mockReturnValue('https://horizon-testnet.stellar.org'),
    } as any;

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        HorizonService,
        { provide: ConfigService, useValue: mockConfigService },
      ],
    }).compile();

    service = module.get(HorizonService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  describe('getTransactions', () => {
    it('should fetch transactions and return raw records', async () => {
      const mockRecords = [{ id: 'tx_1' }, { id: 'tx_2' }];
      const mockResponse = {
        ok: true,
        json: jest.fn().mockResolvedValue({
          _embedded: {
            records: mockRecords,
          },
        }),
      } as any;

      global.fetch = jest.fn().mockResolvedValue(mockResponse);

      const result = await service.getTransactions('contract_123');

      expect(result).toEqual(mockRecords);
      expect(global.fetch).toHaveBeenCalledWith(
        'https://horizon-testnet.stellar.org/accounts/contract_123/transactions?order=asc',
      );
    });

    it('should support passing a cursor', async () => {
      const mockRecords = [{ id: 'tx_1' }];
      const mockResponse = {
        ok: true,
        json: jest.fn().mockResolvedValue({
          _embedded: {
            records: mockRecords,
          },
        }),
      } as any;

      global.fetch = jest.fn().mockResolvedValue(mockResponse);

      const result = await service.getTransactions(
        'contract_123',
        'cursor_xyz',
      );

      expect(result).toEqual(mockRecords);
      expect(global.fetch).toHaveBeenCalledWith(
        'https://horizon-testnet.stellar.org/accounts/contract_123/transactions?order=asc&cursor=cursor_xyz',
      );
    });

    it('should throw an HttpException if response is not ok', async () => {
      const mockResponse = {
        ok: false,
        status: 400,
        statusText: 'Bad Request',
      } as any;

      global.fetch = jest.fn().mockResolvedValue(mockResponse);

      await expect(service.getTransactions('contract_123')).rejects.toThrow(
        HttpException,
      );
    });
  });
});
