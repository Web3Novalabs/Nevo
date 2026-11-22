import { Injectable } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import { Networks } from '@stellar/stellar-sdk';

const DEFAULT_RPC_URL = 'https://soroban-testnet.stellar.org';
const DEFAULT_CONTRACT_ID =
  'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';

/**
 * Stellar network settings, read from `ConfigService` so values in `.env` (and
 * overrides in tests) apply. Every value is read on access rather than when a
 * module is imported, which would run before `ConfigModule` loads `.env`.
 *
 * The frontend equivalents are `NEXT_PUBLIC_STELLAR_NETWORK` and
 * `NEXT_PUBLIC_NETWORK_PASSPHRASE`; see `.env.example`.
 */
@Injectable()
export class StellarConfig {
  constructor(private readonly config: ConfigService) {}

  /** `STELLAR_NETWORK`: `mainnet` or `testnet` (the default). */
  get network(): 'mainnet' | 'testnet' {
    return this.config.get<string>('STELLAR_NETWORK') === 'mainnet'
      ? 'mainnet'
      : 'testnet';
  }

  /** Network passphrase matching {@link network}. */
  get networkPassphrase(): string {
    return this.network === 'mainnet' ? Networks.PUBLIC : Networks.TESTNET;
  }

  /** `STELLAR_RPC_URL`. */
  get rpcUrl(): string {
    return this.config.get<string>('STELLAR_RPC_URL') ?? DEFAULT_RPC_URL;
  }

  /** `CONTRACT_ID`: the deployed Nevo contract. */
  get contractId(): string {
    return this.config.get<string>('CONTRACT_ID') ?? DEFAULT_CONTRACT_ID;
  }

  /** `TOKEN_ADDRESS`: token paid out by pool withdrawals. Throws if unset. */
  get tokenAddress(): string {
    return this.config.getOrThrow<string>('TOKEN_ADDRESS');
  }

  /** `SOURCE_SECRET_KEY`: source account for simulated read-only calls. */
  get sourceSecret(): string {
    return this.config.get<string>('SOURCE_SECRET_KEY') ?? '';
  }
}
