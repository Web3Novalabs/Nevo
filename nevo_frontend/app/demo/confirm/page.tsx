'use client';

import React, { useState } from 'react';
import { ConfirmDialog } from '@/components/ConfirmDialog';
import { Button } from '@/components/Button';

type DialogKey = 'delete' | 'withdraw' | 'cancel' | null;

export default function ConfirmDemoPage() {
  const [open, setOpen] = useState<DialogKey>(null);
  const [loading, setLoading] = useState(false);
  const [lastAction, setLastAction] = useState<string | null>(null);

  const handleConfirm = (label: string) => {
    setLoading(true);
    setTimeout(() => {
      setLoading(false);
      setOpen(null);
      setLastAction(`Confirmed: ${label}`);
    }, 1200);
  };

  return (
    <main className="mx-auto max-w-xl px-6 py-16 flex flex-col gap-8">
      <h1 className="text-2xl font-bold">ConfirmDialog Demo</h1>

      <div className="flex flex-wrap gap-3">
        <Button variant="danger" onClick={() => setOpen('delete')}>
          Delete Pool
        </Button>
        <Button variant="primary" onClick={() => setOpen('withdraw')}>
          Withdraw Funds
        </Button>
        <Button variant="outlined" onClick={() => setOpen('cancel')}>
          Cancel Donation
        </Button>
      </div>

      {lastAction && (
        <p className="text-sm text-[var(--color-text-muted)]">{lastAction}</p>
      )}

      {/* Delete */}
      <ConfirmDialog
        open={open === 'delete'}
        variant="danger"
        title="Delete this pool?"
        message="This action is permanent. All donation records and pool data will be removed from the contract."
        confirmLabel="Delete Pool"
        loading={loading}
        onConfirm={() => handleConfirm('Delete Pool')}
        onCancel={() => setOpen(null)}
      />

      {/* Withdraw */}
      <ConfirmDialog
        open={open === 'withdraw'}
        variant="primary"
        title="Withdraw funds?"
        message="500 XLM will be transferred to your connected wallet. This will close the pool."
        confirmLabel="Withdraw"
        loading={loading}
        onConfirm={() => handleConfirm('Withdraw')}
        onCancel={() => setOpen(null)}
      />

      {/* Cancel donation */}
      <ConfirmDialog
        open={open === 'cancel'}
        variant="danger"
        title="Cancel your donation?"
        message="Your 50 XLM contribution will be refunded to your wallet. This cannot be undone."
        confirmLabel="Cancel Donation"
        loading={loading}
        onConfirm={() => handleConfirm('Cancel Donation')}
        onCancel={() => setOpen(null)}
      />
    </main>
  );
}
