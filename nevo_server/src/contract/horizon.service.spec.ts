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

    describe('non-OK response handling', () => {
      it('throws HttpException with 400 status when Horizon returns 400 Bad Request', async () => {
        const mockResponse = {
          ok: false,
          status: 400,
          statusText: 'Bad Request',
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        try {
          await service.getTransactions('contract_123');
          fail('Expected HttpException to be thrown');
        } catch (error: any) {
          expect(error).toBeInstanceOf(HttpException);
          expect(error.getStatus()).toBe(400);
          expect(error.getMessage()).toContain('Bad Request');
        }
      });

      it('throws HttpException with 401 status when Horizon returns 401 Unauthorized', async () => {
        const mockResponse = {
          ok: false,
          status: 401,
          statusText: 'Unauthorized',
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        try {
          await service.getTransactions('contract_123');
          fail('Expected HttpException to be thrown');
        } catch (error: any) {
          expect(error).toBeInstanceOf(HttpException);
          expect(error.getStatus()).toBe(401);
          expect(error.getMessage()).toContain('Unauthorized');
        }
      });

      it('throws HttpException with 403 status when Horizon returns 403 Forbidden', async () => {
        const mockResponse = {
          ok: false,
          status: 403,
          statusText: 'Forbidden',
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        try {
          await service.getTransactions('contract_123');
          fail('Expected HttpException to be thrown');
        } catch (error: any) {
          expect(error).toBeInstanceOf(HttpException);
          expect(error.getStatus()).toBe(403);
          expect(error.getMessage()).toContain('Forbidden');
        }
      });

      it('throws HttpException with 404 status when Horizon returns 404 Not Found', async () => {
        const mockResponse = {
          ok: false,
          status: 404,
          statusText: 'Not Found',
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        try {
          await service.getTransactions('contract_123');
          fail('Expected HttpException to be thrown');
        } catch (error: any) {
          expect(error).toBeInstanceOf(HttpException);
          expect(error.getStatus()).toBe(404);
          expect(error.getMessage()).toContain('Not Found');
        }
      });

      it('throws HttpException with 500 status when Horizon returns 500 Internal Server Error', async () => {
        const mockResponse = {
          ok: false,
          status: 500,
          statusText: 'Internal Server Error',
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        try {
          await service.getTransactions('contract_123');
          fail('Expected HttpException to be thrown');
        } catch (error: any) {
          expect(error).toBeInstanceOf(HttpException);
          expect(error.getStatus()).toBe(500);
          expect(error.getMessage()).toContain('Internal Server Error');
        }
      });

      it('throws HttpException with 503 status when Horizon returns 503 Service Unavailable', async () => {
        const mockResponse = {
          ok: false,
          status: 503,
          statusText: 'Service Unavailable',
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        try {
          await service.getTransactions('contract_123');
          fail('Expected HttpException to be thrown');
        } catch (error: any) {
          expect(error).toBeInstanceOf(HttpException);
          expect(error.getStatus()).toBe(503);
          expect(error.getMessage()).toContain('Service Unavailable');
        }
      });
    });

    describe('network-level fetch rejection handling', () => {
      it('catches network fetch rejection and rethrows as 500 HttpException', async () => {
        const networkError = new Error('Network error: connection refused');

        global.fetch = jest.fn().mockRejectedValue(networkError);

        try {
          await service.getTransactions('contract_123');
          fail('Expected HttpException to be thrown');
        } catch (error: any) {
          expect(error).toBeInstanceOf(HttpException);
          expect(error.getStatus()).toBe(500);
          expect(error.getMessage()).toContain('unexpected error');
        }
      });

      it('catches fetch timeout and rethrows as 500 HttpException', async () => {
        const timeoutError = new Error('Request timeout');

        global.fetch = jest.fn().mockRejectedValue(timeoutError);

        try {
          await service.getTransactions('contract_123');
          fail('Expected HttpException to be thrown');
        } catch (error: any) {
          expect(error).toBeInstanceOf(HttpException);
          expect(error.getStatus()).toBe(500);
        }
      });

      it('does not leave network errors unhandled', async () => {
        const networkError = new TypeError('Failed to fetch');

        global.fetch = jest.fn().mockRejectedValue(networkError);

        const promise = service.getTransactions('contract_123');

        await expect(promise).rejects.toBeInstanceOf(HttpException);
      });
    });

    describe('HORIZON_URL normalization', () => {
      it('removes trailing slash from HORIZON_URL before building request URL', async () => {
        mockConfigService.get.mockReturnValue('https://horizon-testnet.stellar.org/');

        const module: TestingModule = await Test.createTestingModule({
          providers: [
            HorizonService,
            { provide: ConfigService, useValue: mockConfigService },
          ],
        }).compile();

        const normalizedService = module.get(HorizonService);

        const mockResponse = {
          ok: true,
          json: jest.fn().mockResolvedValue({ _embedded: { records: [] } }),
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        await normalizedService.getTransactions('contract_123');

        const fetchUrl = (global.fetch as jest.Mock).mock.calls[0][0];
        expect(fetchUrl).toBe(
          'https://horizon-testnet.stellar.org/accounts/contract_123/transactions?order=asc',
        );
        expect(fetchUrl).not.toContain('://');
        expect(fetchUrl).not.toMatch(/\/\//g);
      });

      it('does not double-slash when HORIZON_URL has no trailing slash', async () => {
        mockConfigService.get.mockReturnValue('https://horizon-testnet.stellar.org');

        const module: TestingModule = await Test.createTestingModule({
          providers: [
            HorizonService,
            { provide: ConfigService, useValue: mockConfigService },
          ],
        }).compile();

        const normalizedService = module.get(HorizonService);

        const mockResponse = {
          ok: true,
          json: jest.fn().mockResolvedValue({ _embedded: { records: [] } }),
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        await normalizedService.getTransactions('contract_123');

        const fetchUrl = (global.fetch as jest.Mock).mock.calls[0][0];
        expect(fetchUrl).not.toContain('//accounts');
      });
    });

    describe('cursor pagination params', () => {
      const getRequestedUrl = (): URL =>
        new URL((global.fetch as jest.Mock).mock.calls[0][0]);

      beforeEach(() => {
        global.fetch = jest.fn().mockResolvedValue({
          ok: true,
          json: jest.fn().mockResolvedValue({ _embedded: { records: [] } }),
        } as any);
      });

      it('sends no cursor query param when cursor is omitted', async () => {
        await service.getTransactions('contract_123');

        expect(getRequestedUrl().searchParams.has('cursor')).toBe(false);
      });

      it('sends no cursor query param when cursor is an empty string', async () => {
        await service.getTransactions('contract_123', '');

        expect(getRequestedUrl().searchParams.has('cursor')).toBe(false);
      });

      it('appends the supplied cursor exactly once', async () => {
        await service.getTransactions('contract_123', '123456789-1');

        expect(getRequestedUrl().searchParams.getAll('cursor')).toEqual([
          '123456789-1',
        ]);
      });

      it('always sends order=asc without a cursor', async () => {
        await service.getTransactions('contract_123');

        expect(getRequestedUrl().searchParams.getAll('order')).toEqual([
          'asc',
        ]);
      });

      it('always sends order=asc with a cursor', async () => {
        await service.getTransactions('contract_123', 'cursor_xyz');

        expect(getRequestedUrl().searchParams.getAll('order')).toEqual([
          'asc',
        ]);
      });
    });

    describe('missing _embedded records', () => {
      it('returns [] when response has no _embedded key', async () => {
        const mockResponse = {
          ok: true,
          json: jest.fn().mockResolvedValue({}),
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        const result = await service.getTransactions('contract_123');

        expect(result).toEqual([]);
      });

      it('returns [] when _embedded is null', async () => {
        const mockResponse = {
          ok: true,
          json: jest.fn().mockResolvedValue({ _embedded: null }),
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        const result = await service.getTransactions('contract_123');

        expect(result).toEqual([]);
      });

      it('returns [] when _embedded.records is undefined', async () => {
        const mockResponse = {
          ok: true,
          json: jest.fn().mockResolvedValue({ _embedded: {} }),
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        const result = await service.getTransactions('contract_123');

        expect(result).toEqual([]);
      });

      it('returns [] when _embedded.records is null', async () => {
        const mockResponse = {
          ok: true,
          json: jest.fn().mockResolvedValue({ _embedded: { records: null } }),
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        const result = await service.getTransactions('contract_123');

        expect(result).toEqual([]);
      });

      it('returns the records array unchanged when _embedded.records is populated', async () => {
        const mockRecords = [
          { id: 'tx_1', hash: 'abc123' },
          { id: 'tx_2', hash: 'def456' },
          { id: 'tx_3', hash: 'ghi789' },
        ];
        const mockResponse = {
          ok: true,
          json: jest.fn().mockResolvedValue({
            _embedded: { records: mockRecords },
          }),
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        const result = await service.getTransactions('contract_123');

        expect(result).toEqual(mockRecords);
        expect(result).toBe(mockRecords);
      });

      it('returns empty array when _embedded.records is an empty array', async () => {
        const mockResponse = {
          ok: true,
          json: jest.fn().mockResolvedValue({ _embedded: { records: [] } }),
        } as any;

        global.fetch = jest.fn().mockResolvedValue(mockResponse);

        const result = await service.getTransactions('contract_123');

        expect(result).toEqual([]);
      });
    });
  });
});
