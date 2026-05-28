export function useWalletStore() {
  return {
    publicKey: null as string | null,
    balances: null as { XLM: string; USDC: string } | null,
    loading: false,
    initialize: async () => {},
    connectWallet: async (_cb?: () => void) => {},
    disconnectWallet: async () => {},
  };
}
