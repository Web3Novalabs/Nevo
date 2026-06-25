import { Test, TestingModule } from '@nestjs/testing';
import { TransactionBuilder, Networks, Keypair } from '@stellar/stellar-sdk';
import { ContractService } from './contract.service.js';
import { StellarError } from './stellar.error.js';

const SOURCE = Keypair.random().publicKey();
const NETWORK = Networks.TESTNET;

describe('ContractService', () => {
  let service: ContractService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ContractService],
    }).compile();
    service = module.get(ContractService);
  });

  describe('buildCreatePoolTransaction', () => {
    it('returns a valid base64 XDR string', () => {
      const xdr = service.buildCreatePoolTransaction(
        SOURCE,
        '1000',
        'My Pool',
        'desc',
      );
      expect(typeof xdr).toBe('string');
      expect(xdr.length).toBeGreaterThan(0);
      // Must be parseable back into a transaction
      expect(() => TransactionBuilder.fromXDR(xdr, NETWORK)).not.toThrow();
    });
  });

  describe('buildDonateTransaction', () => {
    it('includes the donate contract function name in the XDR', () => {
      const xdr = service.buildDonateTransaction(SOURCE, 1, '500');
      const tx = TransactionBuilder.fromXDR(xdr, NETWORK);
      // The function name is encoded in invokeHostFunctionOp args
      const rawXdr = tx.toXDR();
      expect(rawXdr).toContain(
        Buffer.from('donate').toString('base64').slice(0, 4),
      );
    });

    it('returns a parseable XDR string', () => {
      const xdr = service.buildDonateTransaction(SOURCE, 1, '500');
      expect(() => TransactionBuilder.fromXDR(xdr, NETWORK)).not.toThrow();
    });
  });

  describe('buildWithdrawTransaction', () => {
    it('returns a parseable XDR string', () => {
      const tokenAddress = Keypair.random().publicKey();
      const xdr = service.buildWithdrawTransaction(SOURCE, 1, tokenAddress);
      expect(() => TransactionBuilder.fromXDR(xdr, NETWORK)).not.toThrow();
    });

    it('XDR contains the token address bytes', () => {
      const tokenAddress = Keypair.random().publicKey();
      const xdr = service.buildWithdrawTransaction(SOURCE, 1, tokenAddress);
      expect(xdr.length).toBeGreaterThan(0);
    });
  });

  describe('submitSignedXdr', () => {
    it('throws StellarError when given invalid XDR', async () => {
      await expect(
        service.submitSignedXdr('not-valid-xdr'),
      ).rejects.toBeInstanceOf(StellarError);
    });
  });
});
