import { Injectable, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import {
  Keypair,
  Networks,
  Transaction,
  FeeBumpTransaction,
  TransactionBuilder,
  BASE_FEE,
  Account,
  xdr,
  nativeToScVal,
  Contract,
  scValToNative,
} from '@stellar/stellar-sdk';
import { rpc as StellarRpc } from '@stellar/stellar-sdk';
import type { HistoryEntry } from '../transactions/history.service.js';
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
    const { creator, goal, token, title, description } = params;
    return this.buildTransaction(
      creator,
      this.contract.call(
        'create_pool',
        nativeToScVal(creator, { type: 'address' }),
        nativeToScVal(BigInt(goal), { type: 'i128' }),
        nativeToScVal(token, { type: 'address' }),
        nativeToScVal(title, { type: 'string' }),
        nativeToScVal(description, { type: 'string' }),
      ),
    );
  }

  buildDonateTransaction(
    sourcePublicKey: string,
    poolId: number,
    amount: string,
  ): string {
    return this.buildTransaction(
      sourcePublicKey,
      this.contract.call(
        'donate',
        nativeToScVal(poolId, { type: 'u32' }),
        nativeToScVal(BigInt(amount), { type: 'i128' }),
      ),
    );
  }

  buildWithdrawTransaction(
    sourcePublicKey: string,
    poolId: number,
    tokenAddress: string,
  ): string {
    return this.buildTransaction(
      sourcePublicKey,
      this.contract.call(
        'withdraw',
        nativeToScVal(poolId, { type: 'u32' }),
        nativeToScVal(tokenAddress, { type: 'address' }),
      ),
    );
  }

  buildClosePoolTransaction(sourcePublicKey: string, poolId: number): string {
    return this.buildTransaction(
      sourcePublicKey,
      this.contract.call(
        'close_pool',
        nativeToScVal(poolId, { type: 'u32' }),
        nativeToScVal(sourcePublicKey, { type: 'address' }),
      ),
    );
  }

  private buildTransaction(
    sourcePublicKey: string,
    operation: xdr.Operation,
  ): string {
    try {
      const source = new Account(sourcePublicKey, '0');
      const tx = new TransactionBuilder(source, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(operation)
        .setTimeout(30)
        .build();
      return tx.toXDR();
    } catch (err: unknown) {
      throw this.mapError(err);
    }
  }

  describeSubmission(
    signedXdr: string,
    wallet: string,
  ): Omit<HistoryEntry, 'txHash' | 'status'> | undefined {
    let parsed: Transaction | FeeBumpTransaction;
    try {
      parsed = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);
    } catch {
      throw new StellarError('invalid_xdr');
    }
    const tx =
      parsed instanceof FeeBumpTransaction ? parsed.innerTransaction : parsed;
    const [op] = tx.operations;
    if (
      tx.operations.length !== 1 ||
      op?.type !== 'invokeHostFunction' ||
      (op.source ?? tx.source) !== wallet
    )
      return;
    if (op.func.switch().name !== 'hostFunctionTypeInvokeContract') return;
    const invocation = op.func.invokeContract();
    if (
      scValToNative(xdr.ScVal.scvAddress(invocation.contractAddress())) !==
      this.contract.contractId()
    )
      return;

    const functionName = invocation.functionName().toString();
    const [first, second, third] = invocation
      .args()
      .map((arg) => String(scValToNative(arg)));
    switch (functionName) {
      case 'create_pool':
        if (first === wallet)
          return { type: 'pool_creation', amount: '0', recipient: wallet };
        return;
      case 'donate':
        if (third !== undefined && second !== wallet) return;
        return {
          type: 'donation',
          amount: third ?? second ?? '0',
          recipient: first ?? '',
        };
      case 'withdraw':
        return { type: 'withdrawal', amount: '0', recipient: first ?? '' };
      case 'close_pool':
        if (second === undefined || second === wallet)
          return { type: 'pool_close', amount: '0', recipient: first ?? '' };
        return;
    }
  }

  async submitSignedXdr(signedXdr: string): Promise<string> {
    let tx: Transaction | FeeBumpTransaction;
    try {
      tx = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);
    } catch {
      throw new StellarError('invalid_xdr');
    }
    try {
      const result = await this.rpcServer.sendTransaction(tx);
      return result.hash;
    } catch (err: unknown) {
      throw this.mapError(err);
    }
  }

  async getSubmissionStatus(
    txHash: string,
  ): Promise<'completed' | 'failed' | undefined> {
    const result = await this.rpcServer.getTransaction(txHash);
    if (result.status === StellarRpc.Api.GetTransactionStatus.SUCCESS)
      return 'completed';
    if (result.status === StellarRpc.Api.GetTransactionStatus.FAILED)
      return 'failed';
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
    } catch (err: unknown) {
      this.logger.error(
        `Failed to fetch contribution on-chain for poolId ${poolId}, donor ${donor}`,
        err instanceof Error ? err.stack : String(err),
      );
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

      const native: unknown = scValToNative(retVal);
      if (Array.isArray(native) && native.length >= 5) {
        const fields = native as Array<bigint | number | string | boolean>;
        return {
          id: poolId,
          creator: String(fields[1]),
          goal: BigInt(fields[2]),
          collected: BigInt(fields[3]),
          closed: Boolean(fields[4]),
        };
      }
      return null;
    } catch (err: unknown) {
      this.logger.error(
        `Failed to fetch pool on-chain for poolId ${poolId}`,
        err instanceof Error ? err.stack : String(err),
      );
      return null;
    }
  }

  /**
   * Reads `get_total_raised` and throws on any RPC/simulation failure, so
   * callers that persist the result never mistake an outage for a zero total.
   */
  async fetchTotalRaisedOnChain(poolId: number): Promise<bigint> {
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
    if ('error' in result) {
      throw new Error(
        `get_total_raised simulation failed: ${String(result.error)}`,
      );
    }

    const retVal = result.result?.retval;
    if (!retVal) {
      throw new Error('get_total_raised simulation returned no value');
    }

    const native: unknown = scValToNative(retVal);
    return typeof native === 'bigint' ? native : BigInt(String(native));
  }

  async getTotalRaisedOnChain(poolId: number): Promise<bigint> {
    try {
      return await this.fetchTotalRaisedOnChain(poolId);
      const native: unknown = scValToNative(retVal);
      return typeof native === 'bigint' ? native : BigInt(String(native));
    } catch (err: unknown) {
      this.logger.error(
        `Failed to fetch total raised on-chain for poolId ${poolId}`,
        err instanceof Error ? err.stack : String(err),
      );
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
    } catch (err: unknown) {
      this.logger.error(
        `Failed to fetch donor count on-chain for poolId ${poolId}`,
        err instanceof Error ? err.stack : String(err),
      );
      return 0;
    }
  }

  private mapError(err: unknown): StellarError {
    if (err instanceof StellarError) return err;
    const msg = (err as { message?: string })?.message ?? String(err);
    if (msg.includes('tx_bad_auth')) return new StellarError('tx_bad_auth');
    if (msg.includes('op_underfunded'))
      return new StellarError('op_underfunded');
    if (msg.includes('op_no_source_account'))
      return new StellarError('op_no_source_account');
    if (msg.includes('timeout')) return new StellarError('timeout');
    return new StellarError(msg);
  }
}
