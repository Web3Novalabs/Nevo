'use client';

import React, { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { useWalletStore } from '@/src/store/walletStore';
import { Spinner } from './Spinner';

interface ProtectedRouteProps {
  children: React.ReactNode;
}

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  const router = useRouter();
  const pathname = usePathname();
  const { isAuthenticated, loading, initialize, token, reauthenticate } =
    useWalletStore();
  const [showReauthModal, setShowReauthModal] = useState(false);
  const [reauthError, setReauthError] = useState<string | null>(null);
  const [reauthLoading, setReauthLoading] = useState(false);

  useEffect(() => {
    initialize();
  }, [initialize]);

  useEffect(() => {
    if (loading) return;

    if (!isAuthenticated) {
      if (token) {
        // Token exists but is expired/invalid: prompt re-auth in place.
        setShowReauthModal(true);
      } else {
        // No token at all: hard redirect to login.
        setShowReauthModal(false);
        router.push(`/login?from=${encodeURIComponent(pathname)}`);
      }
    } else {
      // Authenticated (e.g. after successful re-auth): clear the modal.
      setShowReauthModal(false);
      setReauthError(null);
    }
  }, [loading, isAuthenticated, token, pathname, router]);

  const handleReauthenticate = async () => {
    setReauthLoading(true);
    setReauthError(null);
    try {
      await reauthenticate();
      // The store updates isAuthenticated; the effect above clears the modal.
    } catch (err) {
      setReauthError(
        err instanceof Error ? err.message : 'Re-authentication failed'
      );
    } finally {
      setReauthLoading(false);
    }
  };

  const handleGoToLogin = () => {
    setShowReauthModal(false);
    router.push(`/login?from=${encodeURIComponent(pathname)}`);
  };

  if (loading) {
    return (
      <div className="flex min-h-[50vh] items-center justify-center">
        <Spinner size="lg" />
      </div>
    );
  }

  if (!isAuthenticated) {
    if (!token) {
      return (
        <div className="flex min-h-[50vh] items-center justify-center">
          <Spinner size="lg" />
        </div>
      );
    }

    // Keep children mounted (preserving form state) behind the modal.
    return (
      <>
        <div aria-hidden="true" className="pointer-events-none select-none">
          {children}
        </div>
        {showReauthModal && (
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="reauth-title"
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
          >
            <div className="w-full max-w-md rounded-lg bg-white p-6 shadow-xl">
              <h2
                id="reauth-title"
                className="mb-2 text-lg font-semibold text-gray-900"
              >
                Session expired
              </h2>
              <p className="mb-4 text-sm text-gray-600">
                Your session has expired. Please re-authenticate to continue.
                Your current progress will be preserved.
              </p>
              {reauthError && (
                <p role="alert" className="mb-4 text-sm text-red-600">
                  {reauthError}
                </p>
              )}
              <div className="flex justify-end gap-3">
                <button
                  type="button"
                  onClick={handleGoToLogin}
                  className="rounded-md border border-gray-300 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50"
                >
                  Go to login
                </button>
                <button
                  type="button"
                  onClick={handleReauthenticate}
                  disabled={reauthLoading}
                  className="rounded-md bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:opacity-50"
                >
                  {reauthLoading ? 'Re-authenticating…' : 'Re-authenticate'}
                </button>
              </div>
            </div>
          </div>
        )}
      </>
    );
  }

  return <>{children}</>;
}