import { Injectable } from '@nestjs/common';
import {
  Keypair,
  Networks,
  TransactionBuilder,
  BASE_FEE,
  Account,
  xdr,
  nativeToScVal,
  Contract,
  scValToNative,
} from '@stellar/stellar-sdk';
import { rpc as StellarRpc } from '@stellar/stellar-sdk';
import { StellarError } from './stellar.error.js';

const CONTRACT_ID =
  process.env.CONTRACT_ID ??
  'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';
const NETWORK_PASSPHRASE =
  process.env.STELLAR_NETWORK === 'mainnet'
    ? Networks.PUBLIC
    : Networks.TESTNET;
const SOROBAN_URL =
  process.env.SOROBAN_URL ?? 'https://soroban-testnet.stellar.org';
const SOURCE_SECRET = process.env.SOURCE_SECRET_KEY ?? '';

@Injectable()
export class ContractService {
  private readonly contract = new Contract(CONTRACT_ID);

  buildCreatePoolTransaction(
    sourcePublicKey: string,
    goal: string,
    title: string,
    description: string,
  ): string {
    try {
      const source = new Account(sourcePublicKey, '0');
      const tx = new TransactionBuilder(source, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'create_pool',
            nativeToScVal(sourcePublicKey, { type: 'address' }),
            nativeToScVal(BigInt(goal), { type: 'i128' }),
            nativeToScVal(title, { type: 'string' }),
            nativeToScVal(description, { type: 'string' }),
          ),
        )
        .setTimeout(30)
        .build();
      return tx.toXDR();
    } catch (err: unknown) {
      throw this.mapError(err);
    }
  }

  buildDonateTransaction(
    sourcePublicKey: string,
    poolId: number,
    amount: string,
  ): string {
    try {
      const source = new Account(sourcePublicKey, '0');
      const tx = new TransactionBuilder(source, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'donate',
            nativeToScVal(poolId, { type: 'u32' }),
            nativeToScVal(BigInt(amount), { type: 'i128' }),
          ),
        )
        .setTimeout(30)
        .build();
      return tx.toXDR();
    } catch (err: unknown) {
      throw this.mapError(err);
    }
  }

  buildWithdrawTransaction(
    sourcePublicKey: string,
    poolId: number,
    tokenAddress: string,
  ): string {
    try {
      const source = new Account(sourcePublicKey, '0');
      const tx = new TransactionBuilder(source, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'withdraw',
            nativeToScVal(poolId, { type: 'u32' }),
            nativeToScVal(tokenAddress, { type: 'address' }),
          ),
        )
        .setTimeout(30)
        .build();
      return tx.toXDR();
    } catch (err: unknown) {
      throw this.mapError(err);
    }
  }

  async submitSignedXdr(signedXdr: string): Promise<string> {
    try {
      const server = new StellarRpc.Server(SOROBAN_URL);
      const tx = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);
      const result = await server.sendTransaction(tx);
      return result.hash;
    } catch (err: unknown) {
      throw this.mapError(err);
    }
  }

  async getContributionOnChain(poolId: number, donor: string): Promise<bigint> {
    try {
      const server = new StellarRpc.Server(SOROBAN_URL);
      const keypair = SOURCE_SECRET
        ? Keypair.fromSecret(SOURCE_SECRET)
        : Keypair.random();
      const account = await server.getAccount(keypair.publicKey());
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'get_contribution',
            nativeToScVal(poolId, { type: 'u32' }),
            nativeToScVal(donor, { type: 'address' }),
          ),
        )
        .setTimeout(30)
        .build();

      const result = await server.simulateTransaction(tx);
      if ('error' in result) return 0n;

      const simResult = result;
      const retVal = simResult.result?.retval;
      if (!retVal) return 0n;

      const val = xdr.ScVal.fromXDR(retVal.toXDR());
      if (val.switch().name === 'scvI128' || val.switch().name === 'scvU128') {
        const parts = val.i128?.() ?? val.u128?.();
        if (!parts) return 0n;
        return (
          (BigInt(parts.hi().toString()) << 64n) | BigInt(parts.lo().toString())
        );
      }
      return 0n;
    } catch {
      return 0n;
    }
  }

  async getPoolOnChain(poolId: number): Promise<{
    sponsor: string;
    goal: bigint;
    collected: bigint;
    isClosed: boolean;
    applicationDeadline: bigint;
  } | null> {
    try {
      const server = new StellarRpc.Server(SOROBAN_URL);
      const keypair = SOURCE_SECRET
        ? Keypair.fromSecret(SOURCE_SECRET)
        : Keypair.random();
      const account = await server.getAccount(keypair.publicKey());
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'get_pool',
            nativeToScVal(poolId, { type: 'u32' }),
          ),
        )
        .setTimeout(30)
        .build();

      const result = await server.simulateTransaction(tx);
      if ('error' in result) return null;

      const retVal = result.result?.retval;
      if (!retVal) return null;

      const native = scValToNative(retVal);
      if (Array.isArray(native) && native.length >= 6) {
        return {
          sponsor: String(native[1]),
          goal: BigInt(native[2]),
          collected: BigInt(native[3]),
          isClosed: Boolean(native[4]),
          applicationDeadline: BigInt(native[5]),
        };
      }
      return null;
    } catch {
      return null;
    }
  }

  async getTotalRaisedOnChain(poolId: number): Promise<bigint> {
    try {
      const server = new StellarRpc.Server(SOROBAN_URL);
      const keypair = SOURCE_SECRET
        ? Keypair.fromSecret(SOURCE_SECRET)
        : Keypair.random();
      const account = await server.getAccount(keypair.publicKey());
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'get_total_raised',
            nativeToScVal(poolId, { type: 'u32' }),
          ),
        )
        .setTimeout(30)
        .build();

      const result = await server.simulateTransaction(tx);
      if ('error' in result) return 0n;

      const retVal = result.result?.retval;
      if (!retVal) return 0n;

      return BigInt(scValToNative(retVal));
    } catch {
      return 0n;
    }
  }

  async getDonorCountOnChain(poolId: number): Promise<number> {
    try {
      const server = new StellarRpc.Server(SOROBAN_URL);
      const keypair = SOURCE_SECRET
        ? Keypair.fromSecret(SOURCE_SECRET)
        : Keypair.random();
      const account = await server.getAccount(keypair.publicKey());
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'get_donor_count',
            nativeToScVal(poolId, { type: 'u32' }),
          ),
        )
        .setTimeout(30)
        .build();

      const result = await server.simulateTransaction(tx);
      if ('error' in result) return 0;

      const retVal = result.result?.retval;
      if (!retVal) return 0;

      return Number(scValToNative(retVal));
    } catch {
      return 0;
    }
  }

  private mapError(err: unknown): StellarError {
    if (err instanceof StellarError) return err;
    const msg = (err as { message?: string })?.message ?? String(err);
    if (msg.includes('tx_bad_auth')) return new StellarError('tx_bad_auth');
    if (msg.includes('op_underfunded'))
      return new StellarError('op_underfunded');
    return new StellarError(msg);
  }
}
