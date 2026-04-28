'use client';

import { isConnected, requestAccess, getAddress } from '@stellar/freighter-api';

/** Returns the connected public key, or null if not connected/allowed. */
export async function getPublicKey(): Promise<string | null> {
  try {
    const connected = await isConnected();
    if (!connected.isConnected) return null;
    const addr = await getAddress();
    return addr.address || null;
  } catch {
    return null;
  }
}

/** Prompts the user to connect Freighter and calls onConnect on success. */
export async function connect(onConnect: () => Promise<void>): Promise<void> {
  const connected = await isConnected();
  if (!connected.isConnected) {
    throw new Error('Freighter extension is not installed.');
  }
  const access = await requestAccess();
  if (access.error) throw new Error(access.error);
  await onConnect();
}

/** Clears the local wallet state (Freighter has no programmatic disconnect). */
export async function disconnect(): Promise<void> {
  // Freighter does not expose a disconnect API; state is cleared in the store.
}
