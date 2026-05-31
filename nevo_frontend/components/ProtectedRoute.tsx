'use client';

import React, { useEffect } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { useWalletStore } from '@/src/store/walletStore';
import { Spinner } from './Spinner';

interface ProtectedRouteProps {
  children: React.ReactNode;
}

export default function ProtectedRoute({ children }: ProtectedRouteProps) {
  const router = useRouter();
  const pathname = usePathname();
  const { publicKey, loading, initialize } = useWalletStore();

  useEffect(() => {
    initialize();
  }, [initialize]);

  useEffect(() => {
    if (!loading && !publicKey) {
      router.push(`/login?from=${encodeURIComponent(pathname)}`);
    }
  }, [loading, publicKey, pathname, router]);

  if (loading || !publicKey) {
    return (
      <div className="flex min-h-[50vh] items-center justify-center">
        <Spinner size="lg" />
      </div>
    );
  }

  return <>{children}</>;
}
