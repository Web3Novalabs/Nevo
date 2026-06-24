import { Test, TestingModule } from '@nestjs/testing';
import { ContractService } from './contract.service';
import { StellarError } from './stellar.error';
import { TransactionEnvelope } from '@stellar/stellar-sdk/lib/horizon/types/resources';

// ── Minimal mocks so tests run without a live Stellar node ──────────────────

const mockAccount = {
  accountId: () => 'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
  sequenceNumber: () => '1',
  incrementSequenceNumber: jest.fn(),
  // satisfy TransactionBuilder
  sequence: '1',
  _baseAccount: {},
} as any;

jest.mock('@stellar/stellar-sdk', () => {
  const actual = jest.requireActual('@stellar/stellar-sdk');
  return {
    ...actual,
    SorobanRpc: {
      ...actual.SorobanRpc,
      Server: jest.fn().mockImplementation(() => ({
        getAccount: jest.fn().mockResolvedValue(mockAccount),
        sendTransaction: jest.fn().mockResolvedValue({ status: 'PENDING', hash: 'abc123' }),
        simulateTransaction: jest.fn().mockResolvedValue({
          result: { retval: actual.nativeToScVal(0n, { type: 'i128' }) },
        }),
      })),
      Api: actual.SorobanRpc.Api,
    },
  };
});

// ─────────────────────────────────────────────────────────────────────────────

describe('ContractService', () => {
  let service: ContractService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ContractService],
    }).compile();
    service = module.get(ContractService);
  });

  describe('buildCreatePoolTransaction', () => {
    it('returns a valid base64 XDR string', async () => {
      const xdr = await service.buildCreatePoolTransaction(
        'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
        1000n,
        'Test Pool',
        'A pool for testing',
      );
      // base64 XDR is a non-empty string parseable with atob
      expect(typeof xdr).toBe('string');
      expect(xdr.length).toBeGreaterThan(0);
      expect(() => Buffer.from(xdr, 'base64')).not.toThrow();
    });
  });

  describe('buildDonateTransaction', () => {
    it('includes the correct contract function name "donate"', async () => {
      const xdr = await service.buildDonateTransaction(
        'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
        1,
        500n,
      );
      // The function name "donate" must appear in the serialised XDR
      expect(xdr).toContain(Buffer.from('donate').toString('base64').slice(0, 4));
      // More robust: decode and check raw bytes contain the string
      const decoded = Buffer.from(xdr, 'base64').toString('binary');
      expect(decoded).toContain('donate');
    });
  });

  describe('buildWithdrawTransaction', () => {
    it('includes token address as argument', async () => {
      const xdr = await service.buildWithdrawTransaction(
        'GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN',
        1,
        200n,
      );
      const decoded = Buffer.from(xdr, 'base64').toString('binary');
      expect(decoded).toContain('withdraw');
      // TOKEN_ADDRESS default contains part of the address
      expect(decoded).toContain('CDLZFC3S'.slice(0, 6));
    });
  });

  describe('submitSignedXdr', () => {
    it('throws StellarError when given invalid XDR', async () => {
      await expect(service.submitSignedXdr('not-valid-xdr')).rejects.toBeInstanceOf(StellarError);
    });
  });
});
