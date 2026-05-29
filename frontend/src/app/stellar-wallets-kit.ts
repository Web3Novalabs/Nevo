import {
  allowAllModules,
  FREIGHTER_ID,
  StellarWalletsKit,
  WalletNetwork,
} from "@creit.tech/stellar-wallets-kit";

const SELECTED_WALLET_ID = "selectedWalletId";
let kit: StellarWalletsKit | null = null;

function getSelectedWalletId() {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(SELECTED_WALLET_ID);
}

function getKit() {
  if (typeof window === "undefined") {
    throw new Error("Stellar wallet kit is only available in the browser.");
  }

  if (!kit) {
    kit = new StellarWalletsKit({
      modules: allowAllModules(),
      network: WalletNetwork.PUBLIC,
      // StellarWalletsKit forces a wallet selection, even before the user has
      // chosen one, so we default to Freighter and gate usage in getPublicKey.
      selectedWalletId: getSelectedWalletId() ?? FREIGHTER_ID,
    });
  }

  return kit;
}

export async function signTransaction(
  ...args: Parameters<StellarWalletsKit["signTransaction"]>
) {
  return getKit().signTransaction(...args);
}

export async function getPublicKey() {
  if (!getSelectedWalletId()) return null;
  const { address } = await getKit().getAddress();
  return address;
}

export async function setWallet(walletId: string) {
  if (typeof window !== "undefined") {
    localStorage.setItem(SELECTED_WALLET_ID, walletId);
  }
  getKit().setWallet(walletId);
}

export async function disconnect(callback?: () => Promise<void>) {
  if (typeof window !== "undefined") {
    localStorage.removeItem(SELECTED_WALLET_ID);
  }
  getKit().disconnect();
  if (callback) await callback();
}

export async function connect(callback?: () => Promise<void>) {
  await getKit().openModal({
    onWalletSelected: async (option) => {
      try {
        await setWallet(option.id);
        if (callback) await callback();
      } catch (e) {
        console.error(e);
      }
      return option.id;
    },
  });
}
