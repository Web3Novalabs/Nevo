'use client';

interface Props {
  lastUpdated?: string;
}

export default function LegalPrint({ lastUpdated }: Props) {
  const when = lastUpdated ?? new Date().toLocaleDateString();

  return (
    <div className="mb-6 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
      <div className="text-sm text-[var(--color-text-muted)]">
        Last updated: {when}
      </div>
      <div className="flex gap-2">
        <button
          onClick={() => window.print()}
          className="rounded-md bg-[var(--color-surface-raised)] px-3 py-2 text-sm hover:opacity-95"
        >
          Print / Download
        </button>
      </div>
    </div>
  );
}
