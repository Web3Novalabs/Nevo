'use client';

import {
  StellarWalletsKit,
  WalletNetwork,
  allowAllModules,
  FREIGHTER_ID,
  XBULL_ID,
  ALBEDO_ID,
  LOBSTR_ID,
} from '@creit.tech/stellar-wallets-kit';

/**
 * Supported wallet ids, keyed by the ids exported from
 * `@creit.tech/stellar-wallets-kit`.
 *
 * SEP-53 message signing support (via the kit's `signMessage`):
 *  - xBull   : supported
 *  - Albedo  : supported
 *  - Lobstr  : supported
 *  - Freighter: supported
 *
 * Wallets that cannot sign arbitrary messages fall back to
 * `signAuthEntry` (see `signWithWallet`); if neither path is available the
 * call rejects with a descriptive error so the UI can prompt the user to
 * pick a different wallet.
 */
export const SUPPORTED_WALLETS = {
  xbull: XBULL_ID,
  albedo: ALBEDO_ID,
  lobstr: LOBSTR_ID,
  freighter: FREIGHTER_ID,
} as const;

export type WalletId = keyof typeof SUPPORTED_WALLETS;

/** Ordered list of supported wallet ids for the picker modal. */
export const SUPPORTED_WALLET_IDS: WalletId[] = ['xbull', 'albedo', 'lobstr', 'freighter'];

/** Human-readable labels for the wallet picker modal. */
export const WALLET_LABELS: Record<WalletId, string> = {
  xbull: 'xBull',
  albedo: 'Albedo',
  lobstr: 'Lobstr',
  freighter: 'Freighter',
};

/** localStorage key used to persist the selected wallet. */
const STORAGE_KEY = 'nevo:selected-wallet';

/** Returns the persisted wallet id, or null if none/invalid. */
export function getSelectedWalletId(): WalletId | null {
  if (typeof window === 'undefined') return null;
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    if (stored && (SUPPORTED_WALLET_IDS as string[]).includes(stored)) {
      return stored as WalletId;
    }
  } catch {
    // localStorage may be unavailable (private mode, SSR); ignore.
  }
  return null;
}

/** Persists the selected wallet id so it survives reloads. */
export function setSelectedWalletId(id: WalletId): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(STORAGE_KEY, id);
  } catch {
    // Ignore storage failures; the in-memory selection still works.
  }
}

/** Clears the persisted wallet selection. */
export function clearSelectedWalletId(): void {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.removeItem(STORAGE_KEY);
  } catch {
    // Ignore.
  }
}

let kit: StellarWalletsKit | null = null;

/** Returns (and lazily initialises) the shared wallets-kit instance. */
function getKit(): StellarWalletsKit {
  if (!kit) {
    kit = new StellarWalletsKit({
      network: WalletNetwork.TESTNET,
      selectedWalletId: SUPPORTED_WALLETS[getSelectedWalletId() ?? 'freighter'],
      modules: allowAllModules(),
    });
  }
  return kit;
}

/** Switches the active wallet in the kit and persists the choice. */
export function selectWallet(id: WalletId): void {
  setSelectedWalletId(id);
  getKit().setWallet(SUPPORTED_WALLETS[id]);
}

/**
 * Returns true if at least one supported wallet is available in this
 * environment (browser with the kit's modules loaded).
 */
export function isWalletAvailable(): boolean {
  return typeof window !== 'undefined';
}

/**
 * @deprecated Use {@link isWalletAvailable}. Kept for backwards
 * compatibility with code that still imports the Freighter-specific check.
 */
export function isFreighterInstalled(): boolean {
  return isWalletAvailable();
}

/** Returns the connected public key, or null if not connected/allowed. */
export async function getPublicKey(): Promise<string | null> {
  try {
    const { address } = await getKit().getAddress();
    return address || null;
  } catch {
    return null;
  }
}

/**
 * Prompts the user to connect the selected wallet (opening the picker if no
 * wallet has been chosen yet) and calls onConnect on success.
 */
export async function connect(onConnect: () => Promise<void>): Promise<void> {
  if (typeof window === 'undefined') {
    throw new Error('Wallets are only available in the browser.');
  }
  const selected = getSelectedWalletId();
  if (!selected) {
    throw new Error('No wallet selected. Please choose a wallet first.');
  }
  const activeKit = getKit();
  activeKit.setWallet(SUPPORTED_WALLETS[selected]);
  try {
    await activeKit.getAddress();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Could not connect to ${WALLET_LABELS[selected]}: ${message}`);
  }
  await onConnect();
}

/** Clears the local wallet state and forgets the persisted selection. */
export async function disconnect(): Promise<void> {
  try {
    getKit().disconnect();
  } catch {
    // Some wallets do not expose a programmatic disconnect; ignore.
  }
  clearSelectedWalletId();
}

/**
 * Signs a SEP-53 message with the connected wallet.
 *
 * Uses the kit's `signMessage` (SEP-53) path first; if the wallet does not
 * support message signing it falls back to `signAuthEntry`. Wallets that
 * support neither reject with a descriptive error.
 */
export async function signWithWallet(message: string): Promise<string> {
  const activeKit = getKit();
  try {
    const { signedMessage } = await activeKit.signMessage(message);
    return signedMessage;
  } catch (messageErr) {
    try {
      const { signedAuthEntry } = await activeKit.signAuthEntry(message);
      return signedAuthEntry;
    } catch {
      const detail = messageErr instanceof Error ? messageErr.message : String(messageErr);
      throw new Error(`This wallet cannot sign messages: ${detail}`);
    }
  }
}
