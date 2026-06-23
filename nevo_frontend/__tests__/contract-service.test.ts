// Must be set before the module is loaded
process.env.NEXT_PUBLIC_CONTRACT_ID =
  'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';

import { Account } from '@stellar/stellar-sdk';

const TEST_PUBKEY = 'GAVQGTVZ25JJSLUC72LD5T73CKE4EHUEKOKY7CKUF4GSESY5V726ISXJ';

const mockPrepareTransaction = jest
  .fn()
  .mockImplementation((tx) => Promise.resolve(tx));
const mockGetAccount = jest.fn();

jest.mock('@stellar/stellar-sdk/rpc', () => ({
  Server: jest.fn().mockImplementation(() => ({
    getAccount: mockGetAccount,
    prepareTransaction: mockPrepareTransaction,
  })),
}));

import { contractService } from '../lib/contract-service';

describe('ContractService.buildDonateTransaction', () => {
  beforeEach(() => {
    mockGetAccount.mockResolvedValue(new Account(TEST_PUBKEY, '100'));
    mockPrepareTransaction.mockImplementation((tx) => Promise.resolve(tx));
  });

  it('returns a non-empty XDR string', async () => {
    const xdr = await contractService.buildDonateTransaction(
      1,
      TEST_PUBKEY,
      BigInt(5_000_000)
    );
    expect(typeof xdr).toBe('string');
    expect(xdr.length).toBeGreaterThan(0);
  });

  it('produces a base64-decodable XDR envelope', async () => {
    const xdr = await contractService.buildDonateTransaction(
      42,
      TEST_PUBKEY,
      BigInt(10_000_000_000)
    );
    expect(() => Buffer.from(xdr, 'base64')).not.toThrow();
    expect(Buffer.from(xdr, 'base64').length).toBeGreaterThan(0);
  });

  it('handles large bigint amounts without serialization errors', async () => {
    const largeAmount = BigInt('9007199254740992'); // > Number.MAX_SAFE_INTEGER
    await expect(
      contractService.buildDonateTransaction(1, TEST_PUBKEY, largeAmount)
    ).resolves.toBeDefined();
  });
});
