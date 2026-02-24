import * as StellarSdk from "@stellar/stellar-sdk";
import { signTransaction } from "../app/stellar-wallets-kit";

const SERVER_URL = "https://horizon.stellar.org";
const server = new StellarSdk.Horizon.Server(SERVER_URL);

export interface DonationParams {
  poolId: string;
  donor: string;
  asset: "XLM" | "USDC";
  amount: string;
  contractId: string;
}

export interface TransactionResult {
  hash: string;
  success: boolean;
  ledger?: number;
}

export async function executeDonation(
  params: DonationParams
): Promise<TransactionResult> {
  const { donor, amount, asset, poolId, contractId } = params;

  const account = await server.loadAccount(donor);
  const contract = new StellarSdk.Contract(contractId);

  const assetAddress =
    asset === "XLM"
      ? StellarSdk.Address.fromString(
        "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
      )
      : StellarSdk.Address.fromString(
        "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"
      );

  const tx = new StellarSdk.TransactionBuilder(account, {
    fee: StellarSdk.BASE_FEE,
    networkPassphrase: StellarSdk.Networks.PUBLIC,
  })
    .addOperation(
      contract.call(
        "contribute",
        StellarSdk.nativeToScVal(parseInt(poolId), { type: "u64" }),
        new StellarSdk.Address(donor).toScVal(),
        assetAddress.toScVal(),
        StellarSdk.nativeToScVal(
          Math.floor(parseFloat(amount) * 10000000),
          { type: "i128" }
        ),
        StellarSdk.nativeToScVal(false, { type: "bool" })
      )
    )
    .setTimeout(300)
    .build();

  const signedXdr = await signTransaction(tx.toXDR(), {
    networkPassphrase: StellarSdk.Networks.PUBLIC,
  });

  const signedTx = StellarSdk.TransactionBuilder.fromXDR(
    signedXdr.signedTxXdr,
    StellarSdk.Networks.PUBLIC
  );

  const result = await server.submitTransaction(signedTx);

  return {
    hash: result.hash,
    success: result.successful,
    ledger: result.ledger,
  };
}

export interface AccountBalances {
  XLM: string;
  USDC: string;
}

export async function getAccountBalances(address: string): Promise<AccountBalances> {
  try {
    const account = await server.loadAccount(address);
    const balances = account.balances;

    let xlmBalance = "0";
    let usdcBalance = "0";

    const nativeBalance = balances.find(b => b.asset_type === "native");
    if (nativeBalance) {
      xlmBalance = nativeBalance.balance;
    }

    // Checking for USDC (Circle) on Mainnet
    const usdc = balances.find(b => {
      const asset = b as StellarSdk.Horizon.HorizonApi.BalanceLineAsset;
      return (
        asset.asset_code === "USDC" &&
        asset.asset_issuer === "GA5ZSEJYB37JRC5AVCIAZBA2C3FSYV36AVH6C6X5S5F5TH6I2N7VCO3UP"
      );
    });
    if (usdc) {
      usdcBalance = usdc.balance;
    }

    return { XLM: xlmBalance, USDC: usdcBalance };
  } catch (error) {
    console.error("Error fetching account balances:", error);
    return { XLM: "0", USDC: "0" };
  }
}
