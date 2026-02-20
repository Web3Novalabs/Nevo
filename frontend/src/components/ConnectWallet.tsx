"use client";

import { useEffect, useState } from "react";
import { getPublicKey, connect, disconnect } from "../app/stellar-wallets-kit";

export default function ConnectWallet() {
  const [publicKey, setPublicKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  async function showConnected() {
    const key = await getPublicKey();
    if (key) {
      setPublicKey(key);
    } else {
      setPublicKey(null);
    }
    setLoading(false);
  }

  async function showDisconnected() {
    setPublicKey(null);
    setLoading(false);
  }

  useEffect(() => {
    (async () => {
      const key = await getPublicKey();
      if (key) {
        setPublicKey(key);
      }
      setLoading(false);
    })();
  }, []);

  return (
    <div id="connect-wrap" className="wrap" aria-live="polite">
      {!loading && publicKey && (
        <>
          <div className="ellipsis" title={publicKey}>
            Signed in as {publicKey}
          </div>
          <button onClick={() => disconnect(showDisconnected)}>
            Disconnect
          </button>
        </>
      )}

      {!loading && !publicKey && (
        <>
          <button
            onClick={() => connect(showConnected)}
            className="bg-transparent text-[#50C878] hover:bg-blue-700 border border-[#50C878] px-6 py-2 rounded-lg transition font-medium"
          >
            Connect
          </button>
        </>
      )}
    </div>
  );
}
