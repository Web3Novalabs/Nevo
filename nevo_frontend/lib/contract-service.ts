import {
  Address,
  Contract,
  Keypair,
  Networks,
  TransactionBuilder,
  nativeToScVal,
} from '@stellar/stellar-sdk';
import { Server } from '@stellar/stellar-sdk/rpc';

const RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ??
  'https://soroban-testnet.stellar.org';
const NETWORK_PASSPHRASE =
  process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE ?? Networks.TESTNET;
const BASE_FEE = '100';
const TX_TIMEOUT = 30;

function getContractId(): string {
  const id = process.env.NEXT_PUBLIC_CONTRACT_ID;
  if (!id) throw new Error('NEXT_PUBLIC_CONTRACT_ID is not set');
  return id;
}

function addressScVal(publicKey: string) {
  return Address.account(
    Keypair.fromPublicKey(publicKey).rawPublicKey()
  ).toScVal();
}

class ContractService {
  private readonly server = new Server(RPC_URL);

  async buildCreatePoolTransaction(
    creator: string,
    title: string,
    description: string,
    goal: bigint
  ): Promise<string> {
    const contract = new Contract(getContractId());
    const account = await this.server.getAccount(creator);
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        contract.call(
          'create_pool',
          addressScVal(creator),
          nativeToScVal(title, { type: 'string' }),
          nativeToScVal(description, { type: 'string' }),
          nativeToScVal(goal, { type: 'u128' })
        )
      )
      .setTimeout(TX_TIMEOUT)
      .build();

    const prepared = await this.server.prepareTransaction(tx);
    return prepared.toXDR();
  }

  async buildDonateTransaction(
    poolId: number,
    donor: string,
    amount: bigint
  ): Promise<string> {
    const contract = new Contract(getContractId());
    const account = await this.server.getAccount(donor);
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        contract.call(
          'donate',
          nativeToScVal(poolId, { type: 'u32' }),
          addressScVal(donor),
          nativeToScVal(amount, { type: 'u128' })
        )
      )
      .setTimeout(TX_TIMEOUT)
      .build();

    const prepared = await this.server.prepareTransaction(tx);
    return prepared.toXDR();
  }

  async buildWithdrawTransaction(
    poolId: number,
    creator: string,
    tokenAddress: string
  ): Promise<string> {
    const contract = new Contract(getContractId());
    const account = await this.server.getAccount(creator);
    const tx = new TransactionBuilder(account, {
      fee: BASE_FEE,
      networkPassphrase: NETWORK_PASSPHRASE,
    })
      .addOperation(
        contract.call(
          'withdraw_unallocated_funds',
          nativeToScVal(poolId, { type: 'u32' }),
          new Address(tokenAddress).toScVal()
        )
      )
      .setTimeout(TX_TIMEOUT)
      .build();

    const prepared = await this.server.prepareTransaction(tx);
    return prepared.toXDR();
  }
}

export const contractService = new ContractService();
