import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { Networks } from '@stellar/stellar-sdk';
import { ContractService } from './contract.service.js';
import { StellarConfig } from './stellar.config.js';

const CONTRACT_ID = 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';

/** ConfigService stand-in backed by a plain map, like values loaded from .env. */
function configFrom(env: Record<string, string | undefined>) {
  return {
    get: (key: string) => env[key],
    getOrThrow: (key: string) => {
      const value = env[key];
      if (value === undefined) {
        throw new Error(`Configuration key "${key}" does not exist`);
      }
      return value;
    },
  };
}

async function build(env: Record<string, string | undefined>) {
  const module: TestingModule = await Test.createTestingModule({
    providers: [
      StellarConfig,
      ContractService,
      { provide: ConfigService, useValue: configFrom(env) },
    ],
  }).compile();
  return {
    stellar: module.get(StellarConfig),
    contractService: module.get(ContractService),
  };
}

describe('StellarConfig', () => {
  it('uses the public passphrase when STELLAR_NETWORK is mainnet', async () => {
    const { stellar, contractService } = await build({
      STELLAR_NETWORK: 'mainnet',
    });

    expect(stellar.network).toBe('mainnet');
    expect(stellar.networkPassphrase).toBe(Networks.PUBLIC);
    expect(contractService.networkPassphrase).toBe(Networks.PUBLIC);
  });

  it('uses the testnet passphrase when STELLAR_NETWORK is testnet', async () => {
    const { stellar, contractService } = await build({
      STELLAR_NETWORK: 'testnet',
    });

    expect(stellar.network).toBe('testnet');
    expect(contractService.networkPassphrase).toBe(Networks.TESTNET);
  });

  it('defaults to testnet when STELLAR_NETWORK is unset', async () => {
    const { stellar } = await build({});

    expect(stellar.network).toBe('testnet');
    expect(stellar.networkPassphrase).toBe(Networks.TESTNET);
  });

  it('ignores STELLAR_NETWORK in process.env when ConfigService says otherwise', async () => {
    const previous = process.env.STELLAR_NETWORK;
    process.env.STELLAR_NETWORK = 'testnet';
    try {
      const { contractService } = await build({ STELLAR_NETWORK: 'mainnet' });
      expect(contractService.networkPassphrase).toBe(Networks.PUBLIC);
    } finally {
      if (previous === undefined) {
        delete process.env.STELLAR_NETWORK;
      } else {
        process.env.STELLAR_NETWORK = previous;
      }
    }
  });

  it('reads RPC URL, contract id, token address and source secret from config', async () => {
    const { stellar } = await build({
      STELLAR_RPC_URL: 'https://rpc.example.com',
      CONTRACT_ID: CONTRACT_ID,
      TOKEN_ADDRESS: 'CTOKEN',
      SOURCE_SECRET_KEY: 'SSECRET',
    });

    expect(stellar.rpcUrl).toBe('https://rpc.example.com');
    expect(stellar.contractId).toBe(CONTRACT_ID);
    expect(stellar.tokenAddress).toBe('CTOKEN');
    expect(stellar.sourceSecret).toBe('SSECRET');
  });

  it('falls back to defaults for optional values', async () => {
    const { stellar } = await build({});

    expect(stellar.rpcUrl).toBe('https://soroban-testnet.stellar.org');
    expect(stellar.sourceSecret).toBe('');
    expect(() => stellar.tokenAddress).toThrow('TOKEN_ADDRESS');
  });
});
