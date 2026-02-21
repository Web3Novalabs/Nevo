import { signTransaction } from "@/app/stellar-wallets-kit";

export interface DonationParams {
  amount: string;
  asset: "XLM" | "USDC";
  poolAddress: string;
  donorAddress: string;
}

export interface TransactionResult {
  success: boolean;
  hash?: string;
  error?: string;
}

export async function executeDonation(
  params: DonationParams
): Promise<TransactionResult> {
  try {
    const StellarSdk = await import("@stellar/stellar-sdk");
    const server = new StellarSdk.Horizon.Server(
      "https://horizon.stellar.org"
    );

    const account = await server.loadAccount(params.donorAddress);
    const fee = await server.fetchBaseFee();

    let operation;
    if (params.asset === "XLM") {
      operation = StellarSdk.Operation.payment({
        destination: params.poolAddress,
        asset: StellarSdk.Asset.native(),
        amount: params.amount,
      });
    } else {
      const usdcAsset = new StellarSdk.Asset(
        "USDC",
        "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN"
      );
      operation = StellarSdk.Operation.payment({
        destination: params.poolAddress,
        asset: usdcAsset,
        amount: params.amount,
      });
    }

    const transaction = new StellarSdk.TransactionBuilder(account, {
      fee: fee.toString(),
      networkPassphrase: StellarSdk.Networks.PUBLIC,
    })
      .addOperation(operation)
      .setTimeout(180)
      .build();

    const { signedTxXdr } = await signTransaction(transaction.toXDR());
    const signedTx = StellarSdk.TransactionBuilder.fromXDR(
      signedTxXdr,
      StellarSdk.Networks.PUBLIC
    );

    const result = await server.submitTransaction(signedTx);

    return {
      success: true,
      hash: result.hash,
    };
  } catch (error) {
    console.error("Transaction failed:", error);
    return {
      success: false,
      error: error instanceof Error ? error.message : "Unknown error",
    };
  }
}
