import { Injectable } from '@nestjs/common';
import {
  Contract,
  Networks,
  TransactionBuilder,
  nativeToScVal,
  xdr,
  BASE_FEE,
  rpc,
  Transaction,
} from '@stellar/stellar-sdk';
import { StellarError } from './stellar.error';

const CONTRACT_ID =
  process.env.CONTRACT_ID ?? 'CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD2KM';
const TOKEN_ADDRESS =
  process.env.TOKEN_ADDRESS ?? 'CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCN3';
const NETWORK_PASSPHRASE = process.env.NETWORK_PASSPHRASE ?? Networks.TESTNET;
const RPC_URL = process.env.STELLAR_RPC_URL ?? 'https://soroban-testnet.stellar.org';
const TIMEOUT = 30;

@Injectable()
export class ContractService {
  private readonly contract = new Contract(CONTRACT_ID);
  private readonly server = new rpc.Server(RPC_URL);

  // ── XDR builders ────────────────────────────────────────────────────────────

  async buildCreatePoolTransaction(
    sourceAccount: string,
    goal: bigint,
    title: string,
    description: string,
  ): Promise<string> {
    try {
      const account = await this.server.getAccount(sourceAccount);
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'create_pool',
            nativeToScVal(goal, { type: 'i128' }),
            nativeToScVal(title, { type: 'string' }),
            nativeToScVal(description, { type: 'string' }),
          ),
        )
        .setTimeout(TIMEOUT)
        .build();
      return tx.toXDR();
    } catch (e) {
      this.rethrow(e);
    }
  }

  async buildDonateTransaction(
    sourceAccount: string,
    poolId: number,
    amount: bigint,
  ): Promise<string> {
    try {
      const account = await this.server.getAccount(sourceAccount);
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'donate',
            nativeToScVal(poolId, { type: 'u32' }),
            nativeToScVal(amount, { type: 'i128' }),
          ),
        )
        .setTimeout(TIMEOUT)
        .build();
      return tx.toXDR();
    } catch (e) {
      this.rethrow(e);
    }
  }

  async buildWithdrawTransaction(
    sourceAccount: string,
    poolId: number,
    amount: bigint,
  ): Promise<string> {
    try {
      const account = await this.server.getAccount(sourceAccount);
      const tx = new TransactionBuilder(account, {
        fee: BASE_FEE,
        networkPassphrase: NETWORK_PASSPHRASE,
      })
        .addOperation(
          this.contract.call(
            'withdraw',
            nativeToScVal(poolId, { type: 'u32' }),
            nativeToScVal(amount, { type: 'i128' }),
            nativeToScVal(TOKEN_ADDRESS, { type: 'address' }),
          ),
        )
        .setTimeout(TIMEOUT)
        .build();
      return tx.toXDR();
    } catch (e) {
      this.rethrow(e);
    }
  }

  // ── Submit ───────────────────────────────────────────────────────────────────

  async submitSignedXdr(signedXdr: string): Promise<string> {
    try {
      const tx = new Transaction(signedXdr, NETWORK_PASSPHRASE);
      const result = await this.server.sendTransaction(tx);
      if (result.status === 'ERROR') {
        const code =
          (result as any).errorResult?.result()?.results()?.[0]?.tr()?.type()?.name ?? 'unknown';
        throw new StellarError(code);
      }
      return result.hash;
    } catch (e) {
      this.rethrow(e);
    }
  }

  // ── Read-only ────────────────────────────────────────────────────────────────

  /** #675 — Returns how much `donor` has contributed to pool `poolId`. */
  async getContributionOnChain(poolId: number, donor: string): Promise<bigint> {
    try {
      const result = await this.server.simulateTransaction(
        await this.buildSimulationTx(
          this.contract.call(
            'get_contribution',
            nativeToScVal(poolId, { type: 'u32' }),
            nativeToScVal(donor, { type: 'address' }),
          ),
        ),
      );
      if (rpc.Api.isSimulationError(result)) return 0n;
      const retval = (result as rpc.Api.SimulateTransactionSuccessResponse).result?.retval;
      if (!retval) return 0n;
      return BigInt(retval.i128().lo().toString());
    } catch {
      return 0n;
    }
  }

  async getPoolOnChain(poolId: number): Promise<xdr.ScVal | null> {
    try {
      const result = await this.server.simulateTransaction(
        await this.buildSimulationTx(
          this.contract.call('get_pool', nativeToScVal(poolId, { type: 'u32' })),
        ),
      );
      if (rpc.Api.isSimulationError(result)) return null;
      return (
        (result as rpc.Api.SimulateTransactionSuccessResponse).result?.retval ?? null
      );
    } catch {
      return null;
    }
  }

  // ── Helpers ──────────────────────────────────────────────────────────────────

  private async buildSimulationTx(operation: xdr.Operation): Promise<Transaction> {
    // Use a dummy funded account for read-only simulations
    const dummyAccount = await this.server
      .getAccount('GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN')
      .catch(() => null);
    if (!dummyAccount) throw new StellarError('unknown');
    return new TransactionBuilder(dummyAccount, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(operation)
      .setTimeout(TIMEOUT)
      .build();
  }

  private rethrow(e: unknown): never {
    if (e instanceof StellarError) throw e;
    const code =
      (e as any)?.response?.data?.extras?.result_codes?.operations?.[0] ??
      (e as any)?.response?.data?.extras?.result_codes?.transaction ??
      'unknown';
    throw new StellarError(code);
  }
}
