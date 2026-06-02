'use client';

import React, { useState, useMemo } from 'react';
import Link from 'next/link';
import { usePoolsStore, type Pool } from '@/src/store/poolsStore';
import { ProgressBar } from '@/components/ProgressBar';
import { Avatar } from '@/components/Avatar';
import { Button } from '@/components/Button';
import { EmptyState } from '@/components/EmptyState';

type SortMetric = 'raised' | 'target' | 'progress' | 'donors';

export default function PoolComparePage() {
  const { pools } = usePoolsStore();
  const [selectedPools, setSelectedPools] = useState<string[]>([]);
  const [sortBy, setSortBy] = useState<SortMetric>('raised');
  const [isComparing, setIsComparing] = useState(false);

  // Calculate donor count for a pool
  const getDonorCount = (pool: Pool): number => {
    if (pool.id === '1') return 42;
    if (pool.id === '2') return 87;
    if (pool.id === '3') return 31;
    if (pool.id === '4') return 15;
    if (pool.id === '5') return 5;
    return Math.floor((pool.raised * 7.3) / 100) + 1;
  };

  // Sort pools in comparison by selected metric
  const sortedComparisonPools = useMemo(() => {
    const comparingPools = pools.filter((p) => selectedPools.includes(p.id));
    const sorted = [...comparingPools].sort((a, b) => {
      switch (sortBy) {
        case 'raised':
          return b.raised - a.raised;
        case 'target':
          return b.target - a.target;
        case 'progress':
          const progressA = (a.raised / a.target) * 100;
          const progressB = (b.raised / b.target) * 100;
          return progressB - progressA;
        case 'donors':
          return getDonorCount(b) - getDonorCount(a);
        default:
          return 0;
      }
    });
    return sorted;
  }, [selectedPools, sortBy, pools]);

  const comparisonStats = useMemo(() => {
    const pools = sortedComparisonPools;
    const maxRaised = Math.max(0, ...pools.map((pool) => pool.raised));
    const maxTarget = Math.max(0, ...pools.map((pool) => pool.target));
    const maxProgress = Math.max(
      0,
      ...pools.map((pool) =>
        pool.target ? (pool.raised / pool.target) * 100 : 0
      )
    );
    const maxDonors = Math.max(0, ...pools.map((pool) => getDonorCount(pool)));
    return { maxRaised, maxTarget, maxProgress, maxDonors };
  }, [sortedComparisonPools]);

  const highlightClass = (pool: Pool, metric: SortMetric) => {
    const { maxRaised, maxTarget, maxProgress, maxDonors } = comparisonStats;
    const value =
      metric === 'raised'
        ? pool.raised
        : metric === 'target'
          ? pool.target
          : metric === 'progress'
            ? pool.target
              ? (pool.raised / pool.target) * 100
              : 0
            : getDonorCount(pool);

    const best =
      metric === 'raised'
        ? value === maxRaised
        : metric === 'target'
          ? value === maxTarget
          : metric === 'progress'
            ? Math.round(value) === Math.round(maxProgress)
            : value === maxDonors;

    return best
      ? 'bg-brand-50 text-brand-700 dark:text-brand-300'
      : 'text-[var(--color-text)]';
  };

  const togglePoolSelection = (poolId: string) => {
    if (selectedPools.includes(poolId)) {
      setSelectedPools(selectedPools.filter((id) => id !== poolId));
    } else if (selectedPools.length < 4) {
      setSelectedPools([...selectedPools, poolId]);
    }
  };

  const clearSelection = () => {
    setSelectedPools([]);
  };

  if (!isComparing && selectedPools.length === 0) {
    return (
      <div className="min-h-screen bg-[var(--color-background)]">
        <div className="container mx-auto px-4 py-12">
          {/* Header */}
          <div className="mb-12">
            <h1 className="text-4xl font-bold text-[var(--color-text)] mb-3">
              Compare Pools
            </h1>
            <p className="text-lg text-[var(--color-text-secondary)]">
              Select up to 4 pools to compare side-by-side and make informed
              decisions
            </p>
          </div>

          {/* Pool Selection Grid */}
          <div>
            <h2 className="text-xl font-semibold text-[var(--color-text)] mb-6">
              Available Pools ({selectedPools.length}/4 selected)
            </h2>

            {pools.length === 0 ? (
              <EmptyState
                title="No pools available"
                description="Come back later to compare pools"
              />
            ) : (
              <>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-8">
                  {pools.map((pool) => (
                    <div
                      key={pool.id}
                      onClick={() => togglePoolSelection(pool.id)}
                      className={`p-5 rounded-lg border-2 cursor-pointer transition-all duration-200 ${
                        selectedPools.includes(pool.id)
                          ? 'border-brand-500 bg-brand-50/50 dark:bg-brand-950/20'
                          : 'border-[var(--color-border)] bg-[var(--color-surface)] hover:border-brand-300 dark:hover:border-brand-700'
                      } ${selectedPools.length >= 4 && !selectedPools.includes(pool.id) ? 'opacity-50 cursor-not-allowed' : ''}`}
                    >
                      {/* Selection Indicator */}
                      <div className="flex items-start justify-between mb-3">
                        <div
                          className={`w-5 h-5 rounded border-2 flex items-center justify-center transition-all ${
                            selectedPools.includes(pool.id)
                              ? 'bg-brand-500 border-brand-500'
                              : 'border-[var(--color-border)]'
                          }`}
                        >
                          {selectedPools.includes(pool.id) && (
                            <svg
                              className="w-3 h-3 text-white"
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24"
                            >
                              <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={3}
                                d="M5 13l4 4L19 7"
                              />
                            </svg>
                          )}
                        </div>
                        <span
                          className="text-xs font-semibold px-2 py-1 rounded-full"
                          style={{
                            backgroundColor: pool.imageColor + '20',
                            color: pool.imageColor,
                          }}
                        >
                          {pool.category}
                        </span>
                      </div>

                      {/* Pool Info */}
                      <h3 className="font-semibold text-[var(--color-text)] mb-2 line-clamp-2">
                        {pool.title}
                      </h3>

                      {/* Progress */}
                      <div className="mb-3">
                        <div className="flex justify-between items-center mb-2">
                          <span className="text-sm text-[var(--color-text-secondary)]">
                            Progress
                          </span>
                          <span className="text-sm font-semibold text-[var(--color-text)]">
                            {Math.min(
                              100,
                              Math.round((pool.raised / pool.target) * 100)
                            )}
                            %
                          </span>
                        </div>
                        <ProgressBar
                          value={pool.raised}
                          max={pool.target}
                          size="sm"
                        />
                      </div>

                      {/* Quick Stats */}
                      <div className="text-xs text-[var(--color-text-secondary)] space-y-1">
                        <div>
                          Raised: ${pool.raised.toLocaleString()} / $
                          {pool.target.toLocaleString()}
                        </div>
                        <div>Status: {pool.status}</div>
                      </div>
                    </div>
                  ))}
                </div>

                {/* Action Buttons */}
                <div className="flex gap-3 justify-center">
                  <Button
                    onClick={() => {
                      setIsComparing(true);
                    }}
                    disabled={selectedPools.length === 0}
                    className="px-8"
                  >
                    Compare {selectedPools.length} Pool
                    {selectedPools.length !== 1 ? 's' : ''}
                  </Button>
                  {selectedPools.length > 0 && (
                    <Button
                      onClick={clearSelection}
                      variant="secondary"
                      className="px-8"
                    >
                      Clear Selection
                    </Button>
                  )}
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    );
  }

  // Comparison View
  return (
    <div className="min-h-screen bg-[var(--color-background)]">
      <div className="container mx-auto px-4 py-8">
        {/* Header */}
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4 mb-8">
          <div>
            <h1 className="text-3xl font-bold text-[var(--color-text)]">
              Pool Comparison
            </h1>
            <p className="text-[var(--color-text-secondary)] mt-1">
              {sortedComparisonPools.length} pool
              {sortedComparisonPools.length !== 1 ? 's' : ''} selected
            </p>
          </div>

          <Button
            onClick={() => {
              setIsComparing(false);
            }}
            variant="secondary"
            className="w-full md:w-auto"
          >
            Change Selection
          </Button>
        </div>

        {/* Sort Controls */}
        <div className="flex flex-wrap gap-2 mb-8">
          <span className="text-sm text-[var(--color-text-secondary)] font-semibold self-center">
            Sort by:
          </span>
          {(['raised', 'target', 'progress', 'donors'] as const).map(
            (metric) => (
              <button
                key={metric}
                onClick={() => setSortBy(metric)}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-all ${
                  sortBy === metric
                    ? 'bg-brand-500 text-white'
                    : 'bg-[var(--color-surface)] text-[var(--color-text)] border border-[var(--color-border)] hover:border-brand-300'
                }`}
              >
                {metric === 'raised'
                  ? 'Amount Raised'
                  : metric === 'target'
                    ? 'Goal'
                    : metric === 'progress'
                      ? 'Progress %'
                      : 'Donors'}
              </button>
            )
          )}
        </div>

        {/* Comparison Table - Desktop View */}
        <div className="hidden md:block overflow-x-auto bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)]">
          <table className="w-full">
            <thead>
              <tr className="border-b border-[var(--color-border)] bg-[var(--color-background)]">
                <th className="px-6 py-4 text-left text-sm font-semibold text-[var(--color-text)]">
                  Metric
                </th>
                {sortedComparisonPools.map((pool) => (
                  <th
                    key={pool.id}
                    className="px-6 py-4 text-left text-sm font-semibold text-[var(--color-text)]"
                  >
                    <Link
                      href={`/pools/${pool.id}`}
                      className="hover:text-brand-500 transition-colors truncate block"
                    >
                      {pool.title}
                    </Link>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {/* Category Row */}
              <tr className="border-b border-[var(--color-border)] hover:bg-[var(--color-background)]/50 transition-colors">
                <td className="px-6 py-4 text-sm font-semibold text-[var(--color-text)]">
                  Category
                </td>
                {sortedComparisonPools.map((pool) => (
                  <td key={pool.id} className="px-6 py-4 text-sm">
                    <span
                      className="inline-block px-2.5 py-1 rounded-full text-xs font-semibold"
                      style={{
                        backgroundColor: pool.imageColor + '20',
                        color: pool.imageColor,
                      }}
                    >
                      {pool.category}
                    </span>
                  </td>
                ))}
              </tr>

              {/* Status Row */}
              <tr className="border-b border-[var(--color-border)] hover:bg-[var(--color-background)]/50 transition-colors">
                <td className="px-6 py-4 text-sm font-semibold text-[var(--color-text)]">
                  Status
                </td>
                {sortedComparisonPools.map((pool) => (
                  <td key={pool.id} className="px-6 py-4 text-sm">
                    <span
                      className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${
                        pool.status === 'Active'
                          ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400'
                          : 'bg-zinc-500/10 text-zinc-600 dark:text-zinc-400'
                      }`}
                    >
                      <span
                        className={`h-1.5 w-1.5 rounded-full mr-1.5 ${
                          pool.status === 'Active'
                            ? 'bg-emerald-500'
                            : 'bg-zinc-500'
                        }`}
                      />
                      {pool.status}
                    </span>
                  </td>
                ))}
              </tr>

              {/* Goal Row */}
              <tr className="border-b border-[var(--color-border)] hover:bg-[var(--color-background)]/50 transition-colors">
                <td className="px-6 py-4 text-sm font-semibold text-[var(--color-text)]">
                  Goal
                </td>
                {sortedComparisonPools.map((pool) => (
                  <td
                    key={pool.id}
                    className={`px-6 py-4 text-sm font-semibold ${highlightClass(pool, 'target')}`}
                  >
                    ${pool.target.toLocaleString()}
                  </td>
                ))}
              </tr>

              {/* Raised Row */}
              <tr className="border-b border-[var(--color-border)] hover:bg-[var(--color-background)]/50 transition-colors">
                <td className="px-6 py-4 text-sm font-semibold text-[var(--color-text)]">
                  Raised
                </td>
                {sortedComparisonPools.map((pool) => (
                  <td
                    key={pool.id}
                    className={`px-6 py-4 text-sm font-semibold ${highlightClass(pool, 'raised')}`}
                  >
                    ${pool.raised.toLocaleString()}
                  </td>
                ))}
              </tr>

              {/* Progress Row */}
              <tr className="border-b border-[var(--color-border)] hover:bg-[var(--color-background)]/50 transition-colors">
                <td className="px-6 py-4 text-sm font-semibold text-[var(--color-text)]">
                  Progress
                </td>
                {sortedComparisonPools.map((pool) => {
                  const progress = Math.min(
                    100,
                    Math.round((pool.raised / pool.target) * 100)
                  );
                  return (
                    <td
                      key={pool.id}
                      className={`px-6 py-4 ${highlightClass(pool, 'progress')}`}
                    >
                      <div className="space-y-2">
                        <ProgressBar
                          value={pool.raised}
                          max={pool.target}
                          size="sm"
                        />
                        <span className="text-sm font-semibold text-[var(--color-text)]">
                          {progress}%
                        </span>
                      </div>
                    </td>
                  );
                })}
              </tr>

              {/* Donors Row */}
              <tr className="border-b border-[var(--color-border)] hover:bg-[var(--color-background)]/50 transition-colors">
                <td className="px-6 py-4 text-sm font-semibold text-[var(--color-text)]">
                  Donors
                </td>
                {sortedComparisonPools.map((pool) => (
                  <td
                    key={pool.id}
                    className={`px-6 py-4 text-sm font-semibold ${highlightClass(pool, 'donors')}`}
                  >
                    {getDonorCount(pool)}
                  </td>
                ))}
              </tr>

              {/* Created Date Row */}
              <tr className="border-b border-[var(--color-border)] hover:bg-[var(--color-background)]/50 transition-colors">
                <td className="px-6 py-4 text-sm font-semibold text-[var(--color-text)]">
                  Created
                </td>
                {sortedComparisonPools.map((pool) => (
                  <td key={pool.id} className="px-6 py-4 text-sm">
                    {pool.createdAt
                      ? new Date(pool.createdAt).toLocaleDateString()
                      : 'N/A'}
                  </td>
                ))}
              </tr>

              {/* Creator Row */}
              <tr className="hover:bg-[var(--color-background)]/50 transition-colors">
                <td className="px-6 py-4 text-sm font-semibold text-[var(--color-text)]">
                  Creator
                </td>
                {sortedComparisonPools.map((pool) => (
                  <td key={pool.id} className="px-6 py-4">
                    <div className="flex items-center gap-2">
                      <Avatar name={pool.creator || 'Anonymous'} size="sm" />
                      <span className="text-sm text-[var(--color-text)]">
                        {pool.creator
                          ? `${pool.creator.slice(0, 6)}...${pool.creator.slice(-4)}`
                          : 'Anonymous'}
                      </span>
                    </div>
                  </td>
                ))}
              </tr>
            </tbody>
          </table>
        </div>

        {/* Comparison Cards - Mobile View */}
        <div className="md:hidden space-y-6 overflow-x-auto">
          <div className="flex gap-4 pb-4">
            {sortedComparisonPools.map((pool) => (
              <div
                key={pool.id}
                className="flex-shrink-0 w-80 bg-[var(--color-surface)] rounded-lg border border-[var(--color-border)] p-4"
              >
                {/* Pool Header */}
                <div className="mb-4">
                  <Link
                    href={`/pools/${pool.id}`}
                    className="text-lg font-bold text-brand-500 hover:text-brand-600 transition-colors mb-2 line-clamp-2 block"
                  >
                    {pool.title}
                  </Link>
                  <span
                    className="inline-block px-2.5 py-1 rounded-full text-xs font-semibold"
                    style={{
                      backgroundColor: pool.imageColor + '20',
                      color: pool.imageColor,
                    }}
                  >
                    {pool.category}
                  </span>
                </div>

                {/* Status */}
                <div className="mb-4">
                  <span
                    className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold ${
                      pool.status === 'Active'
                        ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400'
                        : 'bg-zinc-500/10 text-zinc-600 dark:text-zinc-400'
                    }`}
                  >
                    <span
                      className={`h-1.5 w-1.5 rounded-full mr-1.5 ${
                        pool.status === 'Active'
                          ? 'bg-emerald-500'
                          : 'bg-zinc-500'
                      }`}
                    />
                    {pool.status}
                  </span>
                </div>

                {/* Metrics */}
                <div className="space-y-3 mb-4 text-sm">
                  <div>
                    <span className="text-[var(--color-text-secondary)]">
                      Goal:
                    </span>
                    <span className="font-semibold text-[var(--color-text)] float-right">
                      ${pool.target.toLocaleString()}
                    </span>
                  </div>
                  <div>
                    <span className="text-[var(--color-text-secondary)]">
                      Raised:
                    </span>
                    <span className="font-semibold text-brand-600 dark:text-brand-400 float-right">
                      ${pool.raised.toLocaleString()}
                    </span>
                  </div>
                  <div>
                    <span className="text-[var(--color-text-secondary)]">
                      Donors:
                    </span>
                    <span className="font-semibold text-[var(--color-text)] float-right">
                      {getDonorCount(pool)}
                    </span>
                  </div>
                </div>

                {/* Progress */}
                <div className="mb-4">
                  <div className="flex justify-between items-center mb-2">
                    <span className="text-sm text-[var(--color-text-secondary)]">
                      Progress
                    </span>
                    <span className="text-sm font-semibold text-[var(--color-text)]">
                      {Math.min(
                        100,
                        Math.round((pool.raised / pool.target) * 100)
                      )}
                      %
                    </span>
                  </div>
                  <ProgressBar
                    value={pool.raised}
                    max={pool.target}
                    size="sm"
                  />
                </div>

                {/* Creator & Date */}
                <div className="pt-4 border-t border-[var(--color-border)] space-y-3 text-xs">
                  <div className="flex items-center gap-2">
                    <span className="text-[var(--color-text-secondary)]">
                      Creator:
                    </span>
                    <Avatar name={pool.creator || 'Anonymous'} size="sm" />
                    <span className="text-[var(--color-text)]">
                      {pool.creator
                        ? `${pool.creator.slice(0, 6)}...${pool.creator.slice(-4)}`
                        : 'Anonymous'}
                    </span>
                  </div>
                  <div>
                    <span className="text-[var(--color-text-secondary)]">
                      Created:
                    </span>
                    <span className="float-right text-[var(--color-text)]">
                      {pool.createdAt
                        ? new Date(pool.createdAt).toLocaleDateString()
                        : 'N/A'}
                    </span>
                  </div>
                </div>

                {/* View Pool Link */}
                <Link
                  href={`/pools/${pool.id}`}
                  className="block mt-4 text-center px-4 py-2 bg-brand-500/10 text-brand-600 dark:text-brand-400 rounded-lg font-medium hover:bg-brand-500/20 transition-colors text-sm"
                >
                  View Pool Details →
                </Link>
              </div>
            ))}
          </div>
        </div>

        {/* Differences Highlight */}
        <div className="mt-12 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-lg p-6">
          <h3 className="text-lg font-semibold text-[var(--color-text)] mb-4">
            Key Differences
          </h3>

          {(() => {
            const pools = sortedComparisonPools;
            if (pools.length < 2) return null;

            const maxRaised = Math.max(...pools.map((p) => p.raised));
            const maxTarget = Math.max(...pools.map((p) => p.target));
            const maxProgress = Math.max(
              ...pools.map((p) => (p.raised / p.target) * 100)
            );

            return (
              <div className="space-y-3 text-sm">
                <div>
                  <span className="font-semibold text-[var(--color-text)]">
                    Highest Raised:
                  </span>
                  <span className="float-right text-brand-600 dark:text-brand-400">
                    {pools.find((p) => p.raised === maxRaised)?.title} ($
                    {maxRaised.toLocaleString()})
                  </span>
                </div>
                <div>
                  <span className="font-semibold text-[var(--color-text)]">
                    Largest Goal:
                  </span>
                  <span className="float-right">
                    {pools.find((p) => p.target === maxTarget)?.title} ($
                    {maxTarget.toLocaleString()})
                  </span>
                </div>
                <div>
                  <span className="font-semibold text-[var(--color-text)]">
                    Highest Progress:
                  </span>
                  <span className="float-right">
                    {
                      pools.find(
                        (p) => (p.raised / p.target) * 100 === maxProgress
                      )?.title
                    }{' '}
                    ({Math.round(maxProgress)}%)
                  </span>
                </div>
                <div>
                  <span className="font-semibold text-[var(--color-text)]">
                    Most Donors:
                  </span>
                  <span className="float-right">
                    {
                      pools.reduce((max, pool) =>
                        getDonorCount(pool) > getDonorCount(max) ? pool : max
                      )?.title
                    }{' '}
                    (
                    {getDonorCount(
                      pools.reduce((max, pool) =>
                        getDonorCount(pool) > getDonorCount(max) ? pool : max
                      )
                    )}
                    )
                  </span>
                </div>
              </div>
            );
          })()}
        </div>

        {/* Footer Actions */}
        <div className="mt-8 flex flex-col md:flex-row gap-3 justify-center">
          <Button
            onClick={() => setIsComparing(false)}
            variant="secondary"
            className="md:w-auto"
          >
            Select Different Pools
          </Button>
          <Link href="/pools" className="inline-block">
            <Button className="w-full md:w-auto">Browse All Pools</Button>
          </Link>
        </div>
      </div>
    </div>
  );
}
