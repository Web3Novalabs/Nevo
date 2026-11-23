import {
  Address,
  Asset,
  Contract,
  Keypair,
  TransactionBuilder,
  scValToNative,
} from '@stellar/stellar-sdk';
import { Server } from '@stellar/stellar-sdk/rpc';
import { Client as NevoContractClient } from '@nevo/contract-client';
import { env } from './env';

export type DonationAsset = 'XLM' | 'USDC';

/** Mirrors the on-chain `ApplicationStatus` contracttype enum. */
export type ApplicationStatus = 'Pending' | 'Approved' | 'Rejected';

const APPLICATION_STATUS_VARIANTS: readonly ApplicationStatus[] = [
  'Pending',
  'Approved',
  'Rejected',
];

/**
 * Normalizes a decoded `ApplicationStatus` enum value into a typed status.
 *
 * Soroban enums decode to their variant name (e.g. `"Approved"`), but some
 * SDK versions may surface them as `{ tag: "Approved" }` or `["Approved"]`.
 * Anything unrecognized is rejected rather than silently coerced.
 */
export function parseApplicationStatus(value: unknown): ApplicationStatus {
  let variant: unknown = value;
  if (Array.isArray(variant)) variant = variant[0];
  if (variant && typeof variant === 'object' && 'tag' in variant) {
    variant = (variant as { tag: unknown }).tag;
  }
  if (typeof variant === 'string') {
    const match = APPLICATION_STATUS_VARIANTS.find((s) => s === variant);
    if (match) return match;
  }
  throw new Error(`Unexpected application status: ${String(value)}`);
}

const USDC_ISSUER = 'GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN';

/** SAC contract ID for the asset used by a token-backed donation. */
export function getDonationTokenAddress(asset: DonationAsset = 'XLM'): string {
  const envVar =
    asset === 'XLM'
      ? 'NEXT_PUBLIC_NATIVE_XLM_SAC_ADDRESS'
      : 'NEXT_PUBLIC_USDC_SAC_ADDRESS';
  const configured = process.env[envVar]?.trim();
  if (configured) return configured;

  const stellarAsset =
    asset === 'XLM' ? Asset.native() : new Asset('USDC', USDC_ISSUER);
  return stellarAsset.contractId(env.networkPassphrase);
}

const RPC_URL = env.sorobanRpcUrl;
const NETWORK_PASSPHRASE = env.networkPassphrase;
const BASE_FEE = '100';
const TX_TIMEOUT = 30;

function getContractId(): string {
  const id = process.env.NEXT_PUBLIC_CONTRACT_ID;
  if (!id) throw new Error('NEXT_PUBLIC_CONTRACT_ID is not set');
  return id;
}

/**
 * Builds the generated contract client used to encode contract calls.
 *
 * The generated bindings are the single source of truth for argument order
 * and types, so any contract signature change breaks this build.
 */
function getGeneratedClient(publicKey: string): NevoContractClient {
  return new NevoContractClient({
    contractId: getContractId(),
    networkPassphrase: NETWORK_PASSPHRASE,
    rpcUrl: RPC_URL,
    publicKey,
  });
}

const I128_MIN = -(2n ** 127n);
const I128_MAX = 2n ** 127n - 1n;

/**
 * Validates that a monetary amount fits in the contract's i128 representation.
 * Throws instead of silently wrapping, so lossy conversions never reach the chain.
 */
function toI128(value: bigint, field: string): bigint {
  if (value < I128_MIN || value > I128_MAX) {
    throw new Error(
      `${field} ${value.toString()} is out of range for i128 ` +
        `[${I128_MIN.toString()}, ${I128_MAX.toString()}]`
    );
  }
  return value;
}

class ContractService {
  private readonly server = new Server(RPC_URL);

  async getPoolCount(): Promise<number> {
    try {
      const contract = new Contract(getContractId());
      const source = Keypair.random();
      const account = await this.server.getAccount(source.publicKey());
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(contract.call('get_pool_count'))
        .setTimeout(TX_TIMEOUT)
        .build();

      const result = await this.server.simulateTransaction(tx);
      if ('error' in result) return -1;
      const retVal = result.result?.retval;
      if (!retVal) return -1;
      return Number(scValToNative(retVal));
    } catch {
      return -1;
    }
  }

  /**
   * Builds an XDR-encoded transaction to create a new donation pool.
   *
   * @param goal - Fundraising goal in stroops (1 XLM = 10,000,000 stroops).
   */
  async buildCreatePoolTransaction(
    creator: string,
    title: string,
    description: string,
    goal: bigint
  ): Promise<string> {
    const account = await this.server.getAccount(creator);
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        getGeneratedClient(creator).createPool({
          creator,
          title,
          description,
          goal,
        })
        contract.call(
          'create_pool',
          addressScVal(creator),
          nativeToScVal(title, { type: 'string' }),
          nativeToScVal(description, { type: 'string' }),
          nativeToScVal(toI128(goal, 'goal'), { type: 'i128' })
        )
      )
      .setTimeout(TX_TIMEOUT)
      .build();

    const prepared = await this.server.prepareTransaction(tx);
    return prepared.toXDR();
  }

  /**
   * Builds an XDR-encoded transaction that transfers tokens into a pool.
   *
   * @param amount - Donation amount in stroops (1 XLM = 10,000,000 stroops).
   * @param tokenAddress - SAC contract ID of the token being donated.
   */
  async buildDonateTransaction(
    poolId: number,
    donor: string,
    amount: bigint,
    tokenAddress: string = getDonationTokenAddress('XLM')
  ): Promise<string> {
    const account = await this.server.getAccount(donor);
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        getGeneratedClient(donor).donateWithToken({
          poolId,
          donor,
          token: tokenAddress,
          amount,
        })
        contract.call(
          'donate_with_token',
          nativeToScVal(poolId, { type: 'u32' }),
          addressScVal(donor),
          new Address(tokenAddress).toScVal(),
          nativeToScVal(toI128(amount, 'amount'), { type: 'i128' })
        )
      )
      .setTimeout(TX_TIMEOUT)
      .build();

    const prepared = await this.server.prepareTransaction(tx);
    return prepared.toXDR();
  }

  /** Builds an XDR-encoded transaction to withdraw unallocated funds from a pool. */
  async buildWithdrawTransaction(
    poolId: number,
    creator: string,
    tokenAddress: string
  ): Promise<string> {
    const account = await this.server.getAccount(creator);
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        getGeneratedClient(creator).withdrawUnallocatedFunds({
          poolId,
          token: tokenAddress,
        })
      )
      .setTimeout(TX_TIMEOUT)
      .build();

    const prepared = await this.server.prepareTransaction(tx);
    return prepared.toXDR();
  }

  /**
   * Reads the status of an application from the contract.
   *
   * The contract returns an `ApplicationStatus` enum, which is decoded into a
   * typed `ApplicationStatus` value.
   */
  async getApplicationStatus(applicationId: number): Promise<ApplicationStatus> {
    const contract = new Contract(getContractId());
    const source = Keypair.random();
    const account = await this.server.getAccount(source.publicKey());
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        contract.call(
          'get_application_status',
          nativeToScVal(applicationId, { type: 'u32' })
        )
      )
      .setTimeout(TX_TIMEOUT)
      .build();

    const result = await this.server.simulateTransaction(tx);
    if ('error' in result) {
      throw new Error(`Failed to read application status: ${result.error}`);
    }
    const retVal = result.result?.retval;
    if (!retVal) throw new Error('No application status returned');
    return parseApplicationStatus(scValToNative(retVal));
  }

  /** Builds an XDR-encoded transaction to close a pool permanently. */
  async buildClosePoolTransaction(
    poolId: number,
    creator: string
  ): Promise<string> {
    const account = await this.server.getAccount(creator);
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        getGeneratedClient(creator).closePool({
          poolId,
        })
      )
      .setTimeout(TX_TIMEOUT)
      .build();

    const prepared = await this.server.prepareTransaction(tx);
    return prepared.toXDR();
  }
}

export const contractService = new ContractService();
