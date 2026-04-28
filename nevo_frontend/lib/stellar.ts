export interface AccountBalances {
  xlm: string;
  usdc: string;
}

const HORIZON = 'https://horizon.stellar.org';

/** Fetches XLM and USDC balances for a Stellar public key via Horizon. */
export async function getAccountBalances(
  publicKey: string
): Promise<AccountBalances> {
  try {
    const res = await fetch(`${HORIZON}/accounts/${publicKey}`);
    if (!res.ok) return { xlm: '0', usdc: '0' };
    const data = await res.json();
    let xlm = '0';
    let usdc = '0';
    for (const b of data.balances ?? []) {
      if (b.asset_type === 'native') xlm = parseFloat(b.balance).toFixed(2);
      if (b.asset_code === 'USDC') usdc = parseFloat(b.balance).toFixed(2);
    }
    return { xlm, usdc };
  } catch {
    return { xlm: '0', usdc: '0' };
  }
}
