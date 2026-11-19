import { Networks } from '@stellar/stellar-sdk';

const REQUIRED_PUBLIC_ENV_VARS = [
  'NEXT_PUBLIC_API_BASE_URL',
  'NEXT_PUBLIC_STELLAR_NETWORK',
] as const;

const DEFAULT_PUBLIC_ENV_VARS = {
  NEXT_PUBLIC_API_BASE_URL: 'http://localhost:3000',
  NEXT_PUBLIC_STELLAR_NETWORK: 'testnet',
} as const;

type RequiredPublicEnvVar = (typeof REQUIRED_PUBLIC_ENV_VARS)[number];

const DEFAULT_PUBLIC_ENV_VALUES: Record<RequiredPublicEnvVar, string> = {
  NEXT_PUBLIC_API_BASE_URL: 'http://localhost:3000',
  NEXT_PUBLIC_STELLAR_NETWORK: 'testnet',
};

type StellarNetwork = 'testnet' | 'public';

const DEFAULT_NETWORK_PASSPHRASES: Record<StellarNetwork, string> = {
  testnet: Networks.TESTNET,
  public: Networks.PUBLIC,
};

const DEFAULT_HORIZON_URLS: Record<StellarNetwork, string> = {
  testnet: 'https://horizon-testnet.stellar.org',
  public: 'https://horizon.stellar.org',
};

const DEFAULT_SOROBAN_RPC_URLS: Record<StellarNetwork, string> = {
  testnet: 'https://soroban-testnet.stellar.org',
  public: 'https://soroban.stellar.org',
};

function getPublicEnvVar(name: RequiredPublicEnvVar): string {
  return process.env[name] || DEFAULT_PUBLIC_ENV_VALUES[name];
}

function getOptionalEnvVar(name: string): string | undefined {
  const value = process.env[name];
  return value && value.trim() !== '' ? value : undefined;
}

function resolveNetwork(): StellarNetwork {
  const raw = getPublicEnvVar('NEXT_PUBLIC_STELLAR_NETWORK').trim().toLowerCase();
  if (raw === 'public' || raw === 'mainnet') {
    return 'public';
  }
  return 'testnet';
}

function resolveNetworkPassphrase(network: StellarNetwork): string {
  const configured = getOptionalEnvVar('NEXT_PUBLIC_NETWORK_PASSPHRASE');
  if (!configured) {
    return DEFAULT_NETWORK_PASSPHRASES[network];
  }
  const expected = DEFAULT_NETWORK_PASSPHRASES[network];
  if (configured !== expected) {
    throw new Error(
      `NEXT_PUBLIC_NETWORK_PASSPHRASE does not match NEXT_PUBLIC_STELLAR_NETWORK "${network}". ` +
        `Expected "${expected}" but got "${configured}".`,
    );
  }
  return configured;
}

function resolveHorizonUrl(network: StellarNetwork): string {
  return getOptionalEnvVar('NEXT_PUBLIC_HORIZON_URL') ?? DEFAULT_HORIZON_URLS[network];
}

function resolveSorobanRpcUrl(network: StellarNetwork): string {
  return getOptionalEnvVar('NEXT_PUBLIC_SOROBAN_RPC_URL') ?? DEFAULT_SOROBAN_RPC_URLS[network];
}

const resolvedNetwork = resolveNetwork();

export function validatePublicEnv(): void {
  REQUIRED_PUBLIC_ENV_VARS.forEach((name) => {
    getPublicEnvVar(name);
  });
  resolveNetworkPassphrase(resolvedNetwork);
}

export const env = {
  NEXT_PUBLIC_API_BASE_URL: getPublicEnvVar('NEXT_PUBLIC_API_BASE_URL'),
  NEXT_PUBLIC_STELLAR_NETWORK: getPublicEnvVar('NEXT_PUBLIC_STELLAR_NETWORK'),
  network: resolvedNetwork,
  networkPassphrase: resolveNetworkPassphrase(resolvedNetwork),
  horizonUrl: resolveHorizonUrl(resolvedNetwork),
  sorobanRpcUrl: resolveSorobanRpcUrl(resolvedNetwork),
} as const;