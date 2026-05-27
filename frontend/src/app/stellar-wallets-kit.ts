import {
  allowAllModules,
  FREIGHTER_ID,
  StellarWalletsKit,
  WalletNetwork,
} from "@creit.tech/stellar-wallets-kit";

const SELECTED_WALLET_ID = "selectedWalletId";

function getSelectedWalletId() {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(SELECTED_WALLET_ID);
}

let kitInstance: StellarWalletsKit | null = null;

function getKit(): StellarWalletsKit {
  if (typeof window === "undefined") {
    throw new Error("StellarWalletsKit is not available on the server side.");
  }
  if (!kitInstance) {
    kitInstance = new StellarWalletsKit({
      modules: allowAllModules(),
      network: WalletNetwork.PUBLIC,
      // StellarWalletsKit forces you to specify a wallet, even if the user didn't
      // select one yet, so we default to Freighter.
      // We'll work around this later in `getPublicKey`.
      selectedWalletId: getSelectedWalletId() ?? FREIGHTER_ID,
    });
  }
  return kitInstance;
}

export async function signTransaction(
  ...args: Parameters<StellarWalletsKit["signTransaction"]>
): ReturnType<StellarWalletsKit["signTransaction"]> {
  const kit = getKit();
  return kit.signTransaction(...args);
}

export async function getPublicKey() {
  if (!getSelectedWalletId()) return null;
  const kit = getKit();
  const { address } = await kit.getAddress();
  return address;
}

export async function setWallet(walletId: string) {
  if (typeof window !== "undefined") {
    localStorage.setItem(SELECTED_WALLET_ID, walletId);
  }
  const kit = getKit();
  kit.setWallet(walletId);
}

export async function disconnect(callback?: () => Promise<void>) {
  if (typeof window !== "undefined") {
    localStorage.removeItem(SELECTED_WALLET_ID);
  }
  const kit = getKit();
  kit.disconnect();
  if (callback) await callback();
}

export async function connect(callback?: () => Promise<void>) {
  const kit = getKit();
  await kit.openModal({
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
