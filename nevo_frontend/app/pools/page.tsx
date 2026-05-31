'use client';

import React, { useEffect, useState, useMemo } from 'react';
import Link from 'next/link';
import { EmptyState } from '@/components/EmptyState';
import { PoolCard } from '@/components';
import {
  usePoolsStore,
  type Pool,
  type PoolStatus,
  type SortOption,
} from '@/src/store/poolsStore';

const CATEGORY_CONFIG = {
  Education: {
    label: 'Education',
    className: 'border-sky-200 bg-sky-50 text-sky-700',
    activeClassName: 'border-sky-500 bg-sky-100 text-sky-800',
  },
  Healthcare: {
    label: 'Healthcare',
    className: 'border-rose-200 bg-rose-50 text-rose-700',
    activeClassName: 'border-rose-500 bg-rose-100 text-rose-800',
  },
  Emergency: {
    label: 'Emergency',
    className: 'border-red-200 bg-red-50 text-red-700',
    activeClassName: 'border-red-500 bg-red-100 text-red-800',
  },
  Humanitarian: {
    label: 'Humanitarian',
    className: 'border-emerald-200 bg-emerald-50 text-emerald-700',
    activeClassName: 'border-emerald-500 bg-emerald-100 text-emerald-800',
  },
  Technology: {
    label: 'Technology',
    className: 'border-indigo-200 bg-indigo-50 text-indigo-700',
    activeClassName: 'border-indigo-500 bg-indigo-100 text-indigo-800',
  },
  Environment: {
    label: 'Environment',
    className: 'border-lime-200 bg-lime-50 text-lime-700',
    activeClassName: 'border-lime-500 bg-lime-100 text-lime-800',
  },
  'Animal Welfare': {
    label: 'Animal Welfare',
    className: 'border-amber-200 bg-amber-50 text-amber-700',
    activeClassName: 'border-amber-500 bg-amber-100 text-amber-800',
  },
  Community: {
    label: 'Community',
    className: 'border-teal-200 bg-teal-50 text-teal-700',
    activeClassName: 'border-teal-500 bg-teal-100 text-teal-800',
  },
  'Art & Culture': {
    label: 'Art & Culture',
    className: 'border-fuchsia-200 bg-fuchsia-50 text-fuchsia-700',
    activeClassName: 'border-fuchsia-500 bg-fuchsia-100 text-fuchsia-800',
  },
} as const;

type PoolCategory = keyof typeof CATEGORY_CONFIG;

const CATEGORIES = Object.keys(CATEGORY_CONFIG) as PoolCategory[];

const POOL_TAGS: Record<string, PoolCategory[]> = {
  '1': ['Humanitarian', 'Emergency', 'Community'],
  '2': ['Technology', 'Education'],
  '3': ['Environment', 'Community'],
  '4': ['Animal Welfare', 'Emergency'],
  '5': ['Education', 'Technology'],
};
// We extract categories from MOCK_POOLS dynamically or define them statically
import { usePoolsStore } from '@/src/store/poolsStore';
import { PoolCard, Pagination } from '@/components';

// Categories matching standard list
const CATEGORIES = [
  'Humanitarian',
  'Technology',
  'Environment',
  'Animal Welfare',
  'Education',
  'Art & Culture',
];

type SortOption = 'newest' | 'most-funded' | 'close-to-goal' | 'trending';
type StatusFilter = 'All' | 'Active' | 'Completed';

export default function BrowsePoolsPage() {
  const {
    pools,
    filters,
    setSearch,
    toggleCategory,
  } = usePoolsStore();
  const [searchInput, setSearchInput] = useState(filters.search);

  // Additional local filter and sort states
  const [statusFilter, setStatusFilter] = useState<StatusFilter>('All');
  const [startDate, setStartDate] = useState<string>('');
  const [endDate, setEndDate] = useState<string>('');
  const [sortBy, setSortBy] = useState<SortOption>('newest');

  // Pagination states
  const [currentPage, setCurrentPage] = useState<number>(1);
  const itemsPerPage = 6;

  // Debounce search input
  useEffect(() => {
    const handler = setTimeout(() => {
      setSearch(searchInput);
      setCurrentPage(1); // Reset page on search
    }, 300);

    return () => clearTimeout(handler);
  }, [searchInput, setSearch]);

  const displayedPools = getDisplayedPools(pools, filters, sortBy);
  // Helper function to calculate donor counts consistently
  const getDonorCount = (id: string, raised: number): number => {
    if (id === '1') return 42;
    if (id === '2') return 87;
    if (id === '3') return 31;
    return Math.floor((raised * 7.3) / 100) + 1;
  };

  // Process pools client-side (Filter -> Sort -> Paginate)
  const processedPools = useMemo(() => {
    // 1. Start with the base list from store (already filters search & categories)
    let list = filteredPools();

    // 2. Filter by status
    if (statusFilter !== 'All') {
      list = list.filter((pool) => pool.status === statusFilter);
    }

    // 3. Filter by date range
    if (startDate) {
      list = list.filter(
        (pool) => pool.createdAt && pool.createdAt >= startDate
      );
    }
    if (endDate) {
      list = list.filter((pool) => pool.createdAt && pool.createdAt <= endDate);
    }

    // 4. Sort the pools
    list = [...list].sort((a, b) => {
      switch (sortBy) {
        case 'most-funded':
          return b.raised - a.raised;
        case 'close-to-goal':
          const pctA = a.raised / a.target;
          const pctB = b.raised / b.target;
          return pctB - pctA;
        case 'trending':
          return getDonorCount(b.id, b.raised) - getDonorCount(a.id, a.raised);
        case 'newest':
        default:
          const dateA = a.createdAt || '';
          const dateB = b.createdAt || '';
          return dateB.localeCompare(dateA);
      }
    });

    return list;
  }, [filteredPools, statusFilter, startDate, endDate, sortBy]);

  // Paginated chunk
  const paginatedPools = useMemo(() => {
    const startIndex = (currentPage - 1) * itemsPerPage;
    return processedPools.slice(startIndex, startIndex + itemsPerPage);
  }, [processedPools, currentPage, itemsPerPage]);

  const handleClearAllFilters = () => {
    setSearchInput('');
    setSearch('');
    setStatusFilter('All');
    setStartDate('');
    setEndDate('');
    setSortBy('newest');
    setCurrentPage(1);
    // Clear store categories
    if (filters.categories.length > 0) {
      filters.categories.forEach((cat) => toggleCategory(cat));
    }
  };

  const activeFilterCount = (searchInput ? 1 : 0) + 
    (statusFilter !== 'All' ? 1 : 0) +
    filters.categories.length +
    (startDate ? 1 : 0) +
    (endDate ? 1 : 0);

  return (
    <main className="mx-auto max-w-7xl px-6 py-10 flex-1 w-full">
      {/* Page Header */}
      <div className="mb-10">
        <h1 className="text-3.5xl font-black tracking-tight text-[var(--color-text)]">
          Browse Donation Pools
        </h1>
        <p className="mt-2 text-sm text-[var(--color-text-muted)] max-w-2xl leading-relaxed">
          Discover, audit, and fund verified Web3 donation pools transparently
          powered by Stellar smart contracts.
        </p>
      </div>

      <div className="flex flex-col gap-8 lg:flex-row items-start">
        {/* Sidebar / Filters */}
        <aside className="w-full lg:w-68 flex-shrink-0 bg-[var(--color-surface-raised)]/20 border border-[var(--color-border)] rounded-2xl p-6 sticky top-24">
          <div className="space-y-6">
            {/* Search Input */}
            <div>
              <label
                htmlFor="search-pools"
                className="block text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] mb-2"
              >
                Search campaigns
              </label>
              <div className="relative">
                <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3.5 text-[var(--color-text-muted)]">
                  <SearchIcon />
                </div>
                <input
                  type="text"
                  id="search-pools"
                  className="block w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] py-3 pl-11 pr-4 text-sm text-[var(--color-text)] outline-none transition-all focus:border-brand-500 focus:ring-1 focus:ring-brand-500 placeholder-zinc-400 dark:placeholder-zinc-500"
                  placeholder="Search title, creator..."
                  value={searchInput}
                  onChange={(e) => setSearchInput(e.target.value)}
                />
              </div>
            </div>

            <hr className="border-[var(--color-border)]" />

            {/* Status Segmented Control */}
            <div>
              <span className="block text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] mb-3">
                Campaign Status
              </span>
              <div className="grid grid-cols-3 gap-1 bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-xl p-1">
                {(['All', 'Active', 'Completed'] as StatusFilter[]).map(
                  (st) => {
                    const isActive = statusFilter === st;
                    const label =
                      st === 'Active'
                        ? 'Open'
                        : st === 'Completed'
                          ? 'Closed'
                          : 'All';
                    return (
                      <button
                        key={st}
                        type="button"
                        onClick={() => {
                          setStatusFilter(st);
                          setCurrentPage(1);
                        }}
                        className={`py-2 rounded-lg text-xs font-semibold transition-all ${
                          isActive
                            ? 'bg-white dark:bg-zinc-800 text-[var(--color-text)] shadow-sm'
                            : 'text-[var(--color-text-muted)] hover:text-[var(--color-text)]'
                        }`}
                      >
                        {label}
                      </button>
                    );
                  }
                )}
              </div>
            </div>

            <hr className="border-[var(--color-border)]" />

            {/* Categories */}
            <div>
              <h3 className="text-sm font-semibold mb-3">Categories</h3>
              <div className="flex flex-wrap gap-2 lg:flex-col">
                {CATEGORIES.map((category) => {
                  const isActive = filters.categories.includes(category);
                  return (
                    <button
                      key={category}
                      onClick={() => toggleCategory(category)}
                      aria-pressed={isActive}
                      className={`rounded-full lg:rounded-lg border px-3 py-1.5 text-left text-sm transition-colors ${
                        isActive
                          ? CATEGORY_CONFIG[category].activeClassName
                          : 'border-[var(--color-border)] bg-transparent text-[var(--color-text-muted)] hover:bg-[var(--color-surface-raised)]'
                      }`}
                    >
                      {CATEGORY_CONFIG[category].label}
              <h3 className="block text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] mb-3">
                Categories
              </h3>
              <div className="flex flex-wrap gap-1.5 lg:flex-col lg:gap-1">
                {CATEGORIES.map((cat) => {
                  const isActive = filters.categories.includes(cat);
                  return (
                    <button
                      key={cat}
                      onClick={() => {
                        toggleCategory(cat);
                        setCurrentPage(1);
                      }}
                      className={`rounded-xl border px-3.5 py-2 text-left text-xs font-semibold transition-all w-full flex items-center justify-between ${
                        isActive
                          ? 'border-brand-500 bg-brand-500/10 text-brand-600 dark:text-brand-400'
                          : 'border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-muted)] hover:bg-[var(--color-surface-raised)] hover:text-[var(--color-text)]'
                      }`}
                    >
                      <span>{cat}</span>
                      {isActive && (
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          viewBox="0 0 20 20"
                          fill="currentColor"
                          className="w-3.5 h-3.5"
                        >
                          <path
                            fillRule="evenodd"
                            d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z"
                            clipRule="evenodd"
                          />
                        </svg>
                      )}
                    </button>
                  );
                })}
              </div>
            </div>

            <hr className="border-[var(--color-border)]" />

            {/* Date Range Inputs */}
            <div>
              <span className="block text-xs font-bold uppercase tracking-wider text-[var(--color-text-muted)] mb-3">
                Creation Date Range
              </span>
              <div className="space-y-3">
                <div>
                  <label
                    htmlFor="start-date"
                    className="block text-[11px] text-[var(--color-text-muted)] mb-1"
                  >
                    From date
                  </label>
                  <input
                    type="date"
                    id="start-date"
                    value={startDate}
                    onChange={(e) => {
                      setStartDate(e.target.value);
                      setCurrentPage(1);
                    }}
                    className="block w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-xs text-[var(--color-text)] outline-none transition-colors focus:border-brand-500"
                  />
                </div>
                <div>
                  <label
                    htmlFor="end-date"
                    className="block text-[11px] text-[var(--color-text-muted)] mb-1"
                  >
                    To date
                  </label>
                  <input
                    type="date"
                    id="end-date"
                    value={endDate}
                    onChange={(e) => {
                      setEndDate(e.target.value);
                      setCurrentPage(1);
                    }}
                    className="block w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-xs text-[var(--color-text)] outline-none transition-colors focus:border-brand-500"
                  />
                </div>
              </div>
            </div>

            <hr className="border-[var(--color-border)]" />

            {/* Clear All Button */}
            <button
              onClick={handleClearAllFilters}
              className="w-full py-2.5 rounded-xl border border-dashed border-[var(--color-border)] text-xs font-semibold text-[var(--color-text-muted)] hover:text-brand-500 hover:border-brand-500 hover:bg-brand-500/5 transition-all text-center"
            >
              Reset All Filters
            </button>
          </div>
        </aside>

        {/* Results */}
        <section className="flex-1 w-full">
          {/* Controls Bar */}
          <div className="mb-6 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between border-b border-[var(--color-border)] pb-4">
            <div className="text-sm font-semibold text-[var(--color-text-muted)]">
              Showing {processedPools.length} pool
              {processedPools.length !== 1 ? 's' : ''}
            </div>

            {/* Sorting Dropdown */}
            <div className="flex items-center gap-2">
              <label
                htmlFor="sort-pools"
                className="text-xs font-bold text-[var(--color-text-muted)] uppercase tracking-wider"
              >
                Sort by:
              </label>
              <select
                id="sort-pools"
                value={sortBy}
                onChange={(e) => {
                  setSortBy(e.target.value as SortOption);
                  setCurrentPage(1);
                }}
                className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-xs font-semibold text-[var(--color-text)] focus-visible:outline-brand-500 cursor-pointer"
              >
                <option value="newest">Newest Campaigns</option>
                <option value="most-funded">Most Funded (XLM)</option>
                <option value="close-to-goal">Close to Goal (%)</option>
                <option value="trending">Popularity / Trending</option>
              </select>
            </div>
          </div>

          {/* Applied Filters Display */}
          {activeFilterCount > 0 && (
            <div className="mb-4 flex flex-wrap items-center gap-2 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)] p-3 text-sm">
              <span className="text-[var(--color-text-muted)]">
                Applied filters:
              </span>
              {selectedFilters.map((filter) => (
                <span
                  key={filter.key}
                  className={`rounded-full border px-3 py-1 text-xs font-medium ${
                    isPoolCategory(filter.label)
                      ? getCategoryClassName(filter.label, true)
                      : 'border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text)]'
                  }`}
                >
                  {filter.label}
              {searchInput && (
                <span className="rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1 text-xs font-medium text-[var(--color-text)]">
                  Search: {searchInput}
                </span>
              )}
              {statusFilter !== 'All' && (
                <span className="rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1 text-xs font-medium text-[var(--color-text)]">
                  Status: {statusFilter}
                </span>
              )}
              {filters.categories.map((cat) => (
                <span key={cat} className="rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1 text-xs font-medium text-[var(--color-text)]">
                  {cat}
                </span>
              ))}
              {startDate && (
                <span className="rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1 text-xs font-medium text-[var(--color-text)]">
                  From: {startDate}
                </span>
              )}
              {endDate && (
                <span className="rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1 text-xs font-medium text-[var(--color-text)]">
                  To: {endDate}
                </span>
              )}
            </div>
          )}

          {displayedPools.length === 0 ? (
            <>
              <EmptyState
                variant="bordered"
                icon="search"
                iconTone="muted"
                title="No results found"
                description="We couldn't find any pools matching your search criteria. Try adjusting your filters or search term."
                action={{
                  label: 'Clear search',
                  onClick: () => {
                    setSearchInput('');
                    setSearch('');
                  },
                  variant: 'link',
                }}
                secondaryAction={{
                  label: 'Create a Pool',
                  href: '/pools/new',
                  variant: 'primary',
                }}
              />
              <div className="flex flex-col items-center justify-center rounded-2xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-raised)] py-24 text-center">
                <div className="flex size-12 items-center justify-center rounded-full bg-[var(--color-border)] text-[var(--color-text-muted)] mb-4">
                  <SearchIcon />
                </div>
                <h3 className="text-base font-semibold">No results found</h3>
                <p className="mt-1 text-sm text-[var(--color-text-muted)] max-w-sm">
                  We couldn&apos;t find any pools matching your search criteria.
                  Try adjusting your filters or search term.
                </p>
                <button
                  onClick={handleClearFilters}
                  className="mt-6 text-sm font-medium text-brand-600 hover:text-brand-700"
                >
                  Clear filters
                </button>
              </div>
            </>
          {/* Grid or Empty State */}
          {processedPools.length === 0 ? (
            <EmptyState
              variant="bordered"
              icon="search"
              iconTone="muted"
              title="No results found"
              description="We couldn't find any pools matching your search criteria. Try adjusting your filters or search term."
              action={{
                label: 'Clear all filters',
                onClick: handleClearAllFilters,
                variant: 'primary',
              }}
              secondaryAction={{
                label: 'Create a Pool',
                href: '/pools/new',
                variant: 'link',
              }}
            />
          ) : (
            <div className="grid gap-6 sm:grid-cols-2 xl:grid-cols-3">
              {displayedPools.map((pool) => (
                <PoolCard
                  key={pool.id}
                  pool={pool}
                  activeCategories={filters.categories}
                  onCategoryClick={toggleCategory}
                />
              ))}
            <div className="space-y-10">
              {/* Grid of Pool Cards */}
              <div className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
                {paginatedPools.map((pool) => (
                  <PoolCard key={pool.id} pool={pool} />
                ))}
              </div>

              {/* Pagination Controls */}
              {processedPools.length > itemsPerPage && (
                <div className="border-t border-[var(--color-border)] pt-6">
                  <Pagination
                    totalItems={processedPools.length}
                    itemsPerPage={itemsPerPage}
                    currentPage={currentPage}
                    onPageChange={(page) => {
                      setCurrentPage(page);
                      // Smooth scroll back to top of section on page switch
                      window.scrollTo({ top: 0, behavior: 'smooth' });
                    }}
                    showGoToPage={processedPools.length > itemsPerPage * 5}
                  />
                </div>
              )}
            </div>
          )}
        </section>
      </div>
    </main>
  );
}

function getPoolTags(pool: Pool): PoolCategory[] {
  const category = isPoolCategory(pool.category) ? pool.category : null;
  return Array.from(new Set([category, ...(POOL_TAGS[pool.id] ?? [])])).filter(
    (tag): tag is PoolCategory => Boolean(tag)
  );
}

function isPoolCategory(value: string): value is PoolCategory {
  return value in CATEGORY_CONFIG;
}

function getCategoryClassName(category: PoolCategory, active = false) {
  return active
    ? CATEGORY_CONFIG[category].activeClassName
    : CATEGORY_CONFIG[category].className;
}

function getDisplayedPools(
  pools: Pool[],
  filters: {
    search: string;
    categories: string[];
    statuses: PoolStatus[];
  },
  sortBy: SortOption
) {
  const searchLower = filters.search.toLowerCase();

  return pools
    .filter((pool) => {
      const tags = getPoolTags(pool);
      const tagText = tags.join(' ').toLowerCase();
      const matchSearch =
        !filters.search ||
        pool.title.toLowerCase().includes(searchLower) ||
        pool.description.toLowerCase().includes(searchLower) ||
        pool.category.toLowerCase().includes(searchLower) ||
        tagText.includes(searchLower) ||
        (pool.creator && pool.creator.toLowerCase().includes(searchLower));
      const matchCategory =
        filters.categories.length === 0 ||
        filters.categories.some(
          (category) => isPoolCategory(category) && tags.includes(category)
        );
      const matchStatus =
        filters.statuses.length === 0 || filters.statuses.includes(pool.status);
      return matchSearch && matchCategory && matchStatus;
    })
    .sort((a, b) => {
      if (sortBy === 'most_raised') {
        return b.raised - a.raised;
      }
      if (sortBy === 'goal_low') {
        return a.target - b.target;
      }
      const dateA = new Date(a.createdAt ?? '1970-01-01').getTime();
      const dateB = new Date(b.createdAt ?? '1970-01-01').getTime();
      return dateB - dateA;
    });
}

function PoolCard({
  pool,
  activeCategories,
  onCategoryClick,
}: {
  pool: Pool;
  activeCategories: string[];
  onCategoryClick: (category: string) => void;
}) {
  const pct = Math.min(100, Math.round((pool.raised / pool.target) * 100));
  const tags = getPoolTags(pool);

  return (
    <article className="flex flex-col overflow-hidden rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] transition-all hover:-translate-y-1 hover:shadow-md">
      <Link
        href={`/pools/${pool.id}`}
        className="block h-24 w-full focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
        style={{ backgroundColor: pool.imageColor || '#e5e7eb' }}
        aria-label={`View ${pool.title}`}
      />
      <div className="flex flex-1 flex-col p-5">
        <div className="mb-3 flex flex-wrap items-center gap-2">
          {tags.map((tag) => {
            const isActive = activeCategories.includes(tag);
            return (
              <button
                type="button"
                key={tag}
                onClick={() => onCategoryClick(tag)}
                aria-pressed={isActive}
                className={`max-w-full rounded-full border px-2.5 py-1 text-xs font-semibold leading-tight transition-colors hover:brightness-95 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600 ${getCategoryClassName(
                  tag,
                  isActive
                )}`}
              >
                {CATEGORY_CONFIG[tag].label}
              </button>
            );
          })}
          <span
            className={`ml-auto inline-flex rounded-full px-2 py-0.5 text-xs font-medium ${
              pool.status === 'Active'
                ? 'bg-success-light text-success-dark'
                : 'bg-[var(--color-border)] text-[var(--color-text-muted)]'
            }`}
          >
            {pool.status}
          </span>
        </div>
        <Link
          href={`/pools/${pool.id}`}
          className="group/title focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
        >
          <h3 className="font-bold text-lg leading-tight transition-colors line-clamp-1 group-hover/title:text-brand-600">
            {pool.title}
          </h3>
        </Link>
        <p className="mt-2 text-sm text-[var(--color-text-muted)] line-clamp-2 flex-1">
          {pool.description}
        </p>

        <div className="mt-6">
          <div className="mb-1.5 flex items-center justify-between text-xs font-medium">
            <span className="text-[var(--color-text)]">
              {pool.raised.toLocaleString()} XLM raised
            </span>
            <span className="text-[var(--color-text-muted)]">{pct}%</span>
          </div>
          <div className="h-1.5 w-full overflow-hidden rounded-full bg-[var(--color-surface-raised)]">
            <div
              className="h-full rounded-full bg-brand-500 transition-all duration-500 ease-out"
              style={{ width: `${pct}%` }}
            />
          </div>
          <div className="mt-2 text-xs text-[var(--color-text-muted)]">
            Goal: {pool.target.toLocaleString()} XLM
          </div>
        </div>
      </div>
    </article>
  );
}

function SearchIcon() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
      strokeWidth={2.5}
      stroke="currentColor"
      className="size-4"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
      />
    </svg>
  );
}