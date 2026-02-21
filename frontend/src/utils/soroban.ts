import {
  Contract,
  Networks,
  SorobanRpc,
  xdr,
  Address,
  nativeToScVal,
  scValToNative,
  TransactionBuilder,
} from "@stellar/stellar-sdk";
import { signTransaction, getPublicKey } from "@/app/stellar-wallets-kit";

// Contract address - should be set via environment variable in production
const CONTRACT_ADDRESS =
  process.env.NEXT_PUBLIC_CONTRACT_ADDRESS ||
  "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVHIX3R2KZ5XK6D3V3V4X5X5X5X5";

// RPC server URL - should be set via environment variable
const RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL ||
  "https://soroban-rpc.mainnet.stellar.org";

// Network passphrase
const NETWORK_PASSPHRASE = Networks.PUBLICNET_PASSPHRASE;

/**
 * Get the RPC server instance
 */
export function getRpcServer(): SorobanRpc.Server {
  return new SorobanRpc.Server(RPC_URL, { allowHttp: RPC_URL.startsWith("http://") });
}

/**
 * Get the contract instance
 */
export function getContract(): Contract {
  return new Contract(CONTRACT_ADDRESS);
}

/**
 * Convert a Stellar address string to an Address ScVal
 */
function addressToScVal(address: string): xdr.ScVal {
  return Address.fromString(address).toScVal();
}

/**
 * Convert a string to ScVal
 */
function stringToScVal(str: string): xdr.ScVal {
  return nativeToScVal(str, { type: "string" });
}

/**
 * Convert a number to i128 ScVal (assuming 7 decimals for Stellar)
 */
function i128ToScVal(value: number | string): xdr.ScVal {
  // Convert to BigInt, multiply by 10^7 for Stellar's native precision
  const numValue = typeof value === "string" ? parseFloat(value) : value;
  const bigIntValue = BigInt(Math.floor(numValue * 10_000_000));
  return nativeToScVal(bigIntValue.toString(), { type: "i128" });
}

/**
 * Convert a number to u64 ScVal
 */
function u64ToScVal(value: number): xdr.ScVal {
  return nativeToScVal(value, { type: "u64" });
}

/**
 * Create a pool on the Soroban contract
 * @param formData - Pool creation form data
 * @returns Transaction hash and pool ID
 */
export async function createPool(formData: {
  name: string;
  description: string;
  externalUrl: string;
  imageHash: string;
  targetAmount: string;
  deadline: number; // Unix timestamp in seconds
}): Promise<{ txHash: string; poolId: number }> {
  const publicKey = await getPublicKey();
  if (!publicKey) {
    throw new Error("Wallet not connected");
  }

  const rpc = getRpcServer();
  const contract = getContract();

  // Get source account
  const sourceAccount = await rpc.getAccount(publicKey);

  // Build metadata struct
  const metadataStruct = xdr.ScMapEntry.fromArray([
    {
      key: xdr.ScVal.scvSymbol("description"),
      val: stringToScVal(formData.description),
    },
    {
      key: xdr.ScVal.scvSymbol("external_url"),
      val: stringToScVal(formData.externalUrl),
    },
    {
      key: xdr.ScVal.scvSymbol("image_hash"),
      val: stringToScVal(formData.imageHash),
    },
  ]);
  const metadataScVal = xdr.ScVal.scvMap(metadataStruct);

  // Build function arguments
  const args = [
    stringToScVal(formData.name), // name
    metadataScVal, // metadata: PoolMetadata
    addressToScVal(publicKey), // creator
    i128ToScVal(formData.targetAmount), // target_amount
    u64ToScVal(formData.deadline), // deadline
    xdr.ScVal.scvVoid(), // required_signatures: None (void for Option::None)
    xdr.ScVal.scvVoid(), // signers: None (void for Option::None)
  ];

  // Build the transaction using TransactionBuilder
  const builder = new TransactionBuilder(sourceAccount, {
    fee: "100",
    networkPassphrase: NETWORK_PASSPHRASE,
  });

  // Add contract invocation
  const operation = contract.call("save_pool", ...args);
  builder.addOperation(operation);

  // Set timeout (30 minutes for Soroban)
  builder.setTimeout(1800);

  // Build the transaction
  const transaction = builder.build();

  // Simulate the transaction
  const simResponse = await rpc.simulateTransaction(transaction);

  if (SorobanRpc.Api.isSimulationError(simResponse)) {
    throw new Error(
      `Simulation failed: ${simResponse.error?.message || JSON.stringify(simResponse.error)}`
    );
  }

  if (!simResponse.result) {
    throw new Error("Simulation returned no result");
  }

  // Restore the transaction with the simulation results
  const restoredTx = SorobanRpc.assembleTransaction(
    transaction,
    simResponse
  ).build();

  // Sign the transaction using the wallet kit
  // The signTransaction function from wallet kit expects XDR string and returns signed XDR string
  const signedTxXdr = await signTransaction(restoredTx.toXDR());

  // Parse the signed transaction
  const signedTx = TransactionBuilder.fromXDR(signedTxXdr, NETWORK_PASSPHRASE);

  // Send the transaction
  const sendResponse = await rpc.sendTransaction(signedTx);

  if (sendResponse.status === "ERROR") {
    throw new Error(
      `Transaction failed: ${sendResponse.errorResult?.message || JSON.stringify(sendResponse.errorResult)}`
    );
  }

  // Wait for the transaction to be included in a ledger
  let attempts = 0;
  const maxAttempts = 30;
  while (attempts < maxAttempts) {
    await new Promise((resolve) => setTimeout(resolve, 2000));
    const tx = await rpc.getTransaction(sendResponse.hash);

    if (tx.status === "SUCCESS" && tx.resultXdr) {
      // Decode the result to get the pool ID
      const result = scValToNative(xdr.ScVal.fromXDR(tx.resultXdr, "base64"));
      return {
        txHash: sendResponse.hash,
        poolId: Number(result),
      };
    } else if (tx.status === "ERROR") {
      throw new Error(`Transaction failed: ${tx.errorResult?.message || "Unknown error"}`);
    } else if (tx.status === "NOT_FOUND") {
      attempts++;
      continue;
    } else {
      attempts++;
      continue;
    }
  }

  throw new Error("Transaction timeout - transaction may still be processing");
}

/**
 * Get Stellar Expert explorer URL for a transaction
 */
export function getStellarExpertUrl(txHash: string): string {
  return `https://stellar.expert/explorer/public/tx/${txHash}`;
}
