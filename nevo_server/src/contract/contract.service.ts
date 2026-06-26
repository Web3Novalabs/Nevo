import { Injectable, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
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

const NETWORK_PASSPHRASE =
  process.env.STELLAR_NETWORK === 'mainnet'
    ? Networks.PUBLIC
    : Networks.TESTNET;
const SOURCE_SECRET = process.env.SOURCE_SECRET_KEY ?? '';

@Injectable()
export class ContractService {
  private readonly logger = new Logger(ContractService.name);
  private readonly rpcServer: StellarRpc.Server;
  private readonly contract: Contract;

  constructor(private readonly config: ConfigService) {
    const rpcUrl =
      this.config.get<string>('STELLAR_RPC_URL') ??
      'https://soroban-testnet.stellar.org';
    const contractId =
      this.config.get<string>('CONTRACT_ID') ??
      'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';
    this.rpcServer = new StellarRpc.Server(rpcUrl);
    this.contract = new Contract(contractId);
    const network =
      NETWORK_PASSPHRASE === Networks.PUBLIC ? 'mainnet' : 'testnet';
    this.logger.log(`Stellar RPC connected: ${rpcUrl} (${network})`);
  }

  buildCreatePoolTransaction(params: {
    creator: string;
    goal: string;
    token: string;
    title: string;
    description: string;
  }): string {
    try {
      const { creator, goal, token, title, description } = params;
      const source = new Account(creator, '0');
      const tx = new TransactionBuilder(source, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'create_pool',
            nativeToScVal(creator, { type: 'address' }),
            nativeToScVal(BigInt(goal), { type: 'i128' }),
            nativeToScVal(token, { type: 'address' }),
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
      const tx = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);
      const result = await this.rpcServer.sendTransaction(tx);
      return result.hash;
    } catch (err: unknown) {
      throw this.mapError(err);
    }
  }

  async getContributionOnChain(poolId: number, donor: string): Promise<bigint> {
    try {
      const keypair = SOURCE_SECRET
        ? Keypair.fromSecret(SOURCE_SECRET)
        : Keypair.random();
      const account = await this.rpcServer.getAccount(keypair.publicKey());
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

      const result = await this.rpcServer.simulateTransaction(tx);
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
    id: number;
    creator: string;
    goal: bigint;
    collected: bigint;
    closed: boolean;
  } | null> {
    try {
      const keypair = SOURCE_SECRET
        ? Keypair.fromSecret(SOURCE_SECRET)
        : Keypair.random();
      const account = await this.rpcServer.getAccount(keypair.publicKey());
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

      const result = await this.rpcServer.simulateTransaction(tx);
      if ('error' in result) return null;

      const retVal = result.result?.retval;
      if (!retVal) return null;

      const native = scValToNative(retVal);
      if (Array.isArray(native) && native.length >= 5) {
        return {
          id: poolId,
          creator: String(native[1]),
          goal: BigInt(native[2]),
          collected: BigInt(native[3]),
          closed: Boolean(native[4]),
        };
      }
      return null;
    } catch {
      return null;
    }
  }

  async getTotalRaisedOnChain(poolId: number): Promise<bigint> {
    try {
      const keypair = SOURCE_SECRET
        ? Keypair.fromSecret(SOURCE_SECRET)
        : Keypair.random();
      const account = await this.rpcServer.getAccount(keypair.publicKey());
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

      const result = await this.rpcServer.simulateTransaction(tx);
      if ('error' in result) return 0n;

      const retVal = result.result?.retval;
      if (!retVal) return 0n;

      const native = scValToNative(retVal);
      return typeof native === 'bigint' ? native : BigInt(String(native));
    } catch {
      return 0n;
    }
  }

  async getDonorCountOnChain(poolId: number): Promise<number> {
    try {
      const keypair = SOURCE_SECRET
        ? Keypair.fromSecret(SOURCE_SECRET)
        : Keypair.random();
      const account = await this.rpcServer.getAccount(keypair.publicKey());
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

      const result = await this.rpcServer.simulateTransaction(tx);
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
