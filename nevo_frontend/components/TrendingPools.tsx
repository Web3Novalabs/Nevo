'use client';

import React, { useEffect, useState } from 'react';
import { usePoolsStore } from '@/src/store/poolsStore';
import { PoolCard } from '@/components/PoolCard';

export function TrendingPools() {
  const { pools, loading, fetchPools } = usePoolsStore();
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setMounted(true);
    if (pools.length === 0) {
      fetchPools();
    }
  }, [pools.length, fetchPools]);

  if (!mounted) return null;

  // Mock trending data: take top 4 pools sorted by raised amount
  const trendingPools = [...pools]
    .filter((p) => p.status === 'Active')
    .sort((a, b) => b.raised - a.raised)
    .slice(0, 4);

  return (
    <section
      aria-labelledby="trending-heading"
      className="mx-auto max-w-5xl px-6 py-20 border-b border-[var(--color-border)]"
    >
      <div className="flex items-center justify-between mb-10">
        <div>
          <h2
            id="trending-heading"
            className="text-3xl font-bold tracking-tight sm:text-4xl"
          >
            Trending Pools
          </h2>
          <p className="mt-2 text-[var(--color-text-muted)]">
            Discover and support the most popular campaigns right now.
          </p>
        </div>
      </div>

      {loading ? (
        <div className="flex gap-6 overflow-x-auto pb-6 snap-x">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="min-w-[300px] sm:min-w-[340px] snap-center">
              <div className="h-64 animate-pulse rounded-2xl bg-[var(--color-border)]"></div>
            </div>
          ))}
        </div>
      ) : trendingPools.length > 0 ? (
        <div
          className="flex gap-6 overflow-x-auto pb-6 snap-x hide-scrollbar"
          style={{ scrollbarWidth: 'none', msOverflowStyle: 'none' }}
        >
          {trendingPools.map((pool) => (
            <div
              key={pool.id}
              className="min-w-[85vw] sm:min-w-[340px] snap-center"
            >
              <PoolCard pool={pool} />
            </div>
          ))}
        </div>
      ) : (
        <p className="text-[var(--color-text-muted)]">
          No trending pools at the moment.
        </p>
      )}
    </section>
  );
}
