// Stub — replace with real Stellar SDK integration
export interface AccountBalances {
  xlm: string;
  usdc: string;
}

export async function getAccountBalances(
  _publicKey: string
): Promise<AccountBalances> {
  return { xlm: "0", usdc: "0" };
}
