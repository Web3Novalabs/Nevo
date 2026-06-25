'use client';

import React from 'react';
import { Skeleton } from '@/components/Skeleton';

export function AboutPageSkeleton() {
  return (
    <main
      className="mx-auto max-w-5xl px-6 py-12"
      aria-busy="true"
      aria-label="Loading about page"
    >
      <div className="mb-16 text-center">
        <div className="h-10 w-48 mx-auto animate-pulse rounded bg-[var(--color-border)] mb-4" />
        <div className="h-5 w-full max-w-2xl mx-auto animate-pulse rounded bg-[var(--color-border)] mb-2" />
        <div className="h-5 w-3/4 max-w-xl mx-auto animate-pulse rounded bg-[var(--color-border)]" />
      </div>

      <section className="mb-16">
        <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-8 space-y-4">
          <div className="h-8 w-32 animate-pulse rounded bg-[var(--color-border)]" />
          <Skeleton lines={4} />
        </div>
      </section>

      <section className="mb-16">
        <div className="h-8 w-48 mx-auto animate-pulse rounded bg-[var(--color-border)] mb-8" />
        <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <div
              key={i}
              className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 space-y-3"
            >
              <div className="h-12 w-12 animate-pulse rounded-xl bg-[var(--color-border)]" />
              <div className="h-5 w-24 animate-pulse rounded bg-[var(--color-border)]" />
              <div className="h-4 w-full animate-pulse rounded bg-[var(--color-border)]" />
            </div>
          ))}
        </div>
      </section>

      <section className="mb-16">
        <div className="h-8 w-48 mx-auto animate-pulse rounded bg-[var(--color-border)] mb-8" />
        <div className="grid gap-8 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 3 }).map((_, i) => (
            <div
              key={i}
              className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6 flex flex-col items-center space-y-3"
            >
              <div className="h-14 w-14 animate-pulse rounded-full bg-[var(--color-border)]" />
              <div className="h-5 w-32 animate-pulse rounded bg-[var(--color-border)]" />
              <div className="h-4 w-24 animate-pulse rounded bg-[var(--color-border)]" />
              <div className="h-4 w-full animate-pulse rounded bg-[var(--color-border)]" />
              <div className="h-4 w-3/4 animate-pulse rounded bg-[var(--color-border)]" />
            </div>
          ))}
        </div>
      </section>
    </main>
  );
}
