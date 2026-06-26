'use client';

import React, { useEffect, useMemo, useRef, useState } from 'react';
import Link from 'next/link';
import { EmptyState } from '@/components/EmptyState';
import { Pagination, PoolCard, Skeleton } from '@/components';
import {
  usePoolsStore,
  type Pool,
  type PoolStatus,
} from '@/src/store/poolsStore';
import { usePoolsStore } from '@/src/store/poolsStore';

type SortOption = 'newest' | 'most-funded' | 'close-to-goal' | 'trending';

interface FilterState {
  search: string;
  categories: string[];
  statuses: PoolStatus[];
  minPrice: number;
  maxPrice: number;
  startDate: string;
  endDate: string;
  sortBy: SortOption;
  page: number;
}

interface ActiveFilter {
  key: string;
  label: string;
  remove: () => void;
}

const ITEMS_PER_PAGE = 6;
const DEFAULT_SORT: SortOption = 'newest';
const STATUS_OPTIONS: PoolStatus[] = ['Active', 'Completed'];
const PARAM_KEYS = {
  search: 'q',
  categories: 'category',
  statuses: 'status',
  minPrice: 'min',
  maxPrice: 'max',
  startDate: 'from',
  endDate: 'to',
  sortBy: 'sort',
  page: 'page',
} as const;

const CATEGORY_STYLES: Record<string, { inactive: string; active: string }> = {
  Education: {
    inactive: 'border-sky-200 bg-sky-50 text-sky-700',
    active: 'border-sky-500 bg-sky-100 text-sky-800',
  },
  Healthcare: {
    inactive: 'border-rose-200 bg-rose-50 text-rose-700',
    active: 'border-rose-500 bg-rose-100 text-rose-800',
  },
  Emergency: {
    inactive: 'border-red-200 bg-red-50 text-red-700',
    active: 'border-red-500 bg-red-100 text-red-800',
  },
  Humanitarian: {
    inactive: 'border-emerald-200 bg-emerald-50 text-emerald-700',
    active: 'border-emerald-500 bg-emerald-100 text-emerald-800',
  },
  Technology: {
    inactive: 'border-indigo-200 bg-indigo-50 text-indigo-700',
    active: 'border-indigo-500 bg-indigo-100 text-indigo-800',
  },
  Environment: {
    inactive: 'border-lime-200 bg-lime-50 text-lime-700',
    active: 'border-lime-500 bg-lime-100 text-lime-800',
  },
  'Animal Welfare': {
    inactive: 'border-amber-200 bg-amber-50 text-amber-700',
    active: 'border-amber-500 bg-amber-100 text-amber-800',
  },
  Community: {
    inactive: 'border-teal-200 bg-teal-50 text-teal-700',
    active: 'border-teal-500 bg-teal-100 text-teal-800',
  },
  'Art & Culture': {
    inactive: 'border-fuchsia-200 bg-fuchsia-50 text-fuchsia-700',
    active: 'border-fuchsia-500 bg-fuchsia-100 text-fuchsia-800',
  },
};

function getDonorCount(pool: Pool): number {
  if (pool.id === '1') return 42;
  if (pool.id === '2') return 87;
  if (pool.id === '3') return 31;
  return Math.floor((pool.raised * 7.3) / 100) + 1;
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

function parseCsvParam(value: string | null) {
  return value
    ? value
        .split(',')
        .map((item) => item.trim())
        .filter(Boolean)
    : [];
}

function getBounds(pools: Pool[]) {
  const targets = pools.map((pool) => pool.target);
  return {
    min: targets.length ? Math.min(...targets) : 0,
    max: targets.length ? Math.max(...targets) : 0,
  };
}

function isPoolStatus(value: string): value is PoolStatus {
  return STATUS_OPTIONS.includes(value as PoolStatus);
}

function isSortOption(value: string | null): value is SortOption {
  return (
    value === 'newest' ||
    value === 'most-funded' ||
    value === 'close-to-goal' ||
    value === 'trending'
  );
}

function normalizeFilters(filters: FilterState, pools: Pool[]): FilterState {
  const bounds = getBounds(pools);
  const minPrice = clamp(filters.minPrice, bounds.min, bounds.max);
  const maxPrice = clamp(filters.maxPrice, minPrice, bounds.max);

  return {
    ...filters,
    categories: filters.categories.filter((category) =>
      pools.some((pool) => pool.category === category)
    ),
    statuses: filters.statuses.filter(isPoolStatus),
    minPrice,
    maxPrice,
    page: Math.max(1, filters.page),
  };
}

function parseFiltersFromUrl(
  pools: Pool[],
  fallback: FilterState
): FilterState {
  if (typeof window === 'undefined') return fallback;

  const params = new URLSearchParams(window.location.search);
  const bounds = getBounds(pools);
  const parsedMin = Number(params.get(PARAM_KEYS.minPrice));
  const parsedMax = Number(params.get(PARAM_KEYS.maxPrice));
  const parsedPage = Number(params.get(PARAM_KEYS.page));
  const sortParam = params.get(PARAM_KEYS.sortBy);

  return normalizeFilters(
    {
      search: params.get(PARAM_KEYS.search) ?? fallback.search,
      categories: parseCsvParam(params.get(PARAM_KEYS.categories)),
      statuses: parseCsvParam(params.get(PARAM_KEYS.statuses)).filter(
        isPoolStatus
      ),
      minPrice: Number.isFinite(parsedMin) ? parsedMin : bounds.min,
      maxPrice: Number.isFinite(parsedMax) ? parsedMax : bounds.max,
      startDate: params.get(PARAM_KEYS.startDate) ?? '',
      endDate: params.get(PARAM_KEYS.endDate) ?? '',
      sortBy: isSortOption(sortParam) ? sortParam : DEFAULT_SORT,
      page: Number.isFinite(parsedPage) ? parsedPage : 1,
    },
    pools
  );
}

function writeFiltersToUrl(
  filters: FilterState,
  bounds: { min: number; max: number }
) {
  if (typeof window === 'undefined') return;

  const params = new URLSearchParams();
  if (filters.search) params.set(PARAM_KEYS.search, filters.search);
  if (filters.categories.length) {
    params.set(PARAM_KEYS.categories, filters.categories.join(','));
  }
  if (filters.statuses.length) {
    params.set(PARAM_KEYS.statuses, filters.statuses.join(','));
  }
  if (filters.minPrice !== bounds.min) {
    params.set(PARAM_KEYS.minPrice, String(filters.minPrice));
  }
  if (filters.maxPrice !== bounds.max) {
    params.set(PARAM_KEYS.maxPrice, String(filters.maxPrice));
  }
  if (filters.startDate) params.set(PARAM_KEYS.startDate, filters.startDate);
  if (filters.endDate) params.set(PARAM_KEYS.endDate, filters.endDate);
  if (filters.sortBy !== DEFAULT_SORT) {
    params.set(PARAM_KEYS.sortBy, filters.sortBy);
  }
  if (filters.page > 1) params.set(PARAM_KEYS.page, String(filters.page));

  const query = params.toString();
  const nextUrl = query
    ? `${window.location.pathname}?${query}`
    : window.location.pathname;
  window.history.replaceState(null, '', nextUrl);
}

function poolMatchesFilters(pool: Pool, filters: FilterState) {
  const search = filters.search.trim().toLowerCase();
  const createdAt = pool.createdAt ?? '';
  const matchesSearch =
    !search ||
    pool.title.toLowerCase().includes(search) ||
    pool.description.toLowerCase().includes(search) ||
    pool.category.toLowerCase().includes(search) ||
    pool.creator?.toLowerCase().includes(search);
  const matchesCategory =
    filters.categories.length === 0 ||
    filters.categories.includes(pool.category);
  const matchesStatus =
    filters.statuses.length === 0 || filters.statuses.includes(pool.status);
  const matchesPrice =
    pool.target >= filters.minPrice && pool.target <= filters.maxPrice;
  const matchesStartDate = !filters.startDate || createdAt >= filters.startDate;
  const matchesEndDate = !filters.endDate || createdAt <= filters.endDate;

  return (
    matchesSearch &&
    matchesCategory &&
    matchesStatus &&
    matchesPrice &&
    matchesStartDate &&
    matchesEndDate
  );
}

function filterPools(pools: Pool[], filters: FilterState) {
  return pools.filter((pool) => poolMatchesFilters(pool, filters));
}

function sortPools(pools: Pool[], sortBy: SortOption) {
  return [...pools].sort((a, b) => {
    if (sortBy === 'most-funded') return b.raised - a.raised;
    if (sortBy === 'close-to-goal') {
      return b.raised / b.target - a.raised / a.target;
    }
    if (sortBy === 'trending') return getDonorCount(b) - getDonorCount(a);
    return (b.createdAt ?? '').localeCompare(a.createdAt ?? '');
  });
}

function buildDefaultFilters(pools: Pool[]): FilterState {
  const bounds = getBounds(pools);

  return {
    search: '',
    categories: [],
    statuses: [],
    minPrice: bounds.min,
    maxPrice: bounds.max,
    startDate: '',
    endDate: '',
    sortBy: DEFAULT_SORT,
    page: 1,
  };
}

export default function BrowsePoolsPage() {
  const { pools, loading, error, fetchPools } = usePoolsStore();
  const hasFetched = useRef(false);
  useEffect(() => {
    fetchPools();
  }, [fetchPools]);

  const bounds = useMemo(() => getBounds(pools), [pools]);
  const categories = useMemo(
    () => Array.from(new Set(pools.map((pool) => pool.category))).sort(),
    [pools]
  );
  const defaultFilters = useMemo(() => buildDefaultFilters(pools), [pools]);
  const hasHydrated = useRef(false);
  const [filters, setFilters] = useState<FilterState>(defaultFilters);
  const [searchInput, setSearchInput] = useState(defaultFilters.search);

  useEffect(() => {
    if (hasFetched.current) {
      fetchPools(filters);
    } else {
      hasFetched.current = true;
    }
  }, [filters, fetchPools]);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      const nextFilters = parseFiltersFromUrl(pools, defaultFilters);
      setFilters(nextFilters);
      setSearchInput(nextFilters.search);
      hasHydrated.current = true;
    }, 0);

    return () => window.clearTimeout(timeout);
  }, [defaultFilters, pools]);

  useEffect(() => {
    if (!hasHydrated.current) return;
    writeFiltersToUrl(filters, bounds);
  }, [bounds, filters]);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setFilters((current) => ({ ...current, search: searchInput, page: 1 }));
    }, 250);

    return () => window.clearTimeout(timeout);
  }, [searchInput]);

  const filteredPools = useMemo(
    () => filterPools(pools, filters),
    [filters, pools]
  );
  const sortedPools = useMemo(
    () => sortPools(filteredPools, filters.sortBy),
    [filteredPools, filters.sortBy]
  );
  const totalPages = Math.max(
    1,
    Math.ceil(sortedPools.length / ITEMS_PER_PAGE)
  );
  const currentPage = clamp(filters.page, 1, totalPages);
  const paginatedPools = useMemo(() => {
    const startIndex = (currentPage - 1) * ITEMS_PER_PAGE;
    return sortedPools.slice(startIndex, startIndex + ITEMS_PER_PAGE);
  }, [currentPage, sortedPools]);

  const categoryCounts = useMemo(
    () =>
      Object.fromEntries(
        categories.map((category) => [
          category,
          pools.filter((pool) =>
            poolMatchesFilters(pool, {
              ...filters,
              categories: [category],
              page: 1,
            })
          ).length,
        ])
      ) as Record<string, number>,
    [categories, filters, pools]
  );

  const statusCounts = useMemo(
    () =>
      Object.fromEntries(
        STATUS_OPTIONS.map((status) => [
          status,
          pools.filter((pool) =>
            poolMatchesFilters(pool, {
              ...filters,
              statuses: [status],
              page: 1,
            })
          ).length,
        ])
      ) as Record<PoolStatus, number>,
    [filters, pools]
  );

  function updateFilters(update: Partial<FilterState>) {
    setFilters((current) => ({
      ...current,
      ...update,
      page: update.page ?? 1,
    }));
  }

  function toggleCategory(category: string) {
    updateFilters({
      categories: filters.categories.includes(category)
        ? filters.categories.filter((item) => item !== category)
        : [...filters.categories, category],
    });
  }

  function toggleStatus(status: PoolStatus) {
    updateFilters({
      statuses: filters.statuses.includes(status)
        ? filters.statuses.filter((item) => item !== status)
        : [...filters.statuses, status],
    });
  }

  function updateMinPrice(value: number) {
    updateFilters({
      minPrice: Math.min(value, filters.maxPrice),
      maxPrice: filters.maxPrice,
    });
  }

  function updateMaxPrice(value: number) {
    updateFilters({
      minPrice: filters.minPrice,
      maxPrice: Math.max(value, filters.minPrice),
    });
  }

  function clearAllFilters() {
    setSearchInput('');
    setFilters(defaultFilters);
  }

  const activeFilters: ActiveFilter[] = [
    ...(filters.search
      ? [
          {
            key: 'search',
            label: `Search: ${filters.search}`,
            remove: () => {
              setSearchInput('');
              updateFilters({ search: '' });
            },
          },
        ]
      : []),
    ...filters.categories.map((category) => ({
      key: `category-${category}`,
      label: category,
      remove: () =>
        updateFilters({
          categories: filters.categories.filter((item) => item !== category),
        }),
    })),
    ...filters.statuses.map((status) => ({
      key: `status-${status}`,
      label: `Status: ${status}`,
      remove: () =>
        updateFilters({
          statuses: filters.statuses.filter((item) => item !== status),
        }),
    })),
    ...(filters.minPrice !== bounds.min || filters.maxPrice !== bounds.max
      ? [
          {
            key: 'price',
            label: `${filters.minPrice.toLocaleString()}-${filters.maxPrice.toLocaleString()} XLM`,
            remove: () =>
              updateFilters({ minPrice: bounds.min, maxPrice: bounds.max }),
          },
        ]
      : []),
    ...(filters.startDate || filters.endDate
      ? [
          {
            key: 'date',
            label: `${filters.startDate || 'Any'} to ${filters.endDate || 'Any'}`,
            remove: () => updateFilters({ startDate: '', endDate: '' }),
          },
        ]
      : []),
  ];

  if (error !== null) {
    return (
      <main className="mx-auto flex w-full max-w-7xl flex-1 flex-col px-6 py-10">
        <EmptyState
          variant="bordered"
          iconTone="muted"
          icon="not-found"
          title="Failed to load pools"
          description={error}
          action={{
            label: 'Try again',
            onClick: fetchPools,
            variant: 'primary',
          }}
        />
      </main>
    );
  }

  if (loading) {
    return (
      <main className="mx-auto flex w-full max-w-7xl flex-1 flex-col px-6 py-10">
        <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
          <Skeleton variant="text" lines={2} className="w-64" />
        </div>
        <div className="grid gap-6 sm:grid-cols-2 xl:grid-cols-3">
          {Array.from({ length: 8 }).map((_, i) => (
            <Skeleton key={i} variant="card" />
          ))}
        </div>
      </main>
    );
  }

  return (
    <main className="mx-auto flex w-full max-w-7xl flex-1 flex-col px-6 py-10">
      <div className="mb-8 flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-3xl font-black text-[var(--color-text)]">
            Browse Donation Pools
          </h1>
          <p className="mt-2 max-w-2xl text-sm leading-relaxed text-[var(--color-text-muted)]">
            Discover, audit, and fund verified Web3 donation pools transparently
            powered by Stellar smart contracts.
          </p>
        </div>
        <div className="flex gap-2">
          <Link
            href="/pools/compare"
            className="inline-flex items-center justify-center rounded-xl border border-brand-600 bg-transparent px-4 py-2 text-sm font-semibold text-brand-600 transition-colors hover:bg-brand-50 dark:hover:bg-brand-950/30"
          >
            Compare Pools
          </Link>
          <Link
            href="/pools/new"
            className="inline-flex items-center justify-center rounded-xl bg-brand-600 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-brand-700"
          >
            Create Pool
          </Link>
        </div>
      </div>

      <div className="flex flex-col gap-6 lg:flex-row lg:items-start">
        <aside className="w-full flex-shrink-0 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface-raised)]/30 p-5 lg:sticky lg:top-24 lg:w-72">
          <div className="space-y-6">
            <div>
              <label
                htmlFor="search-pools"
                className="mb-2 block text-xs font-bold uppercase text-[var(--color-text-muted)]"
              >
                Search campaigns
              </label>
              <div className="relative">
                <span className="pointer-events-none absolute inset-y-0 left-3 flex items-center text-[var(--color-text-muted)]">
                  <SearchIcon />
                </span>
                <input
                  id="search-pools"
                  type="search"
                  value={searchInput}
                  onChange={(event) => setSearchInput(event.target.value)}
                  placeholder="Search title, category, creator"
                  className="w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] py-3 pl-10 pr-3 text-sm text-[var(--color-text)] outline-none transition-colors focus:border-brand-500"
                />
              </div>
            </div>

            <FilterSection title="Goal range">
              <div className="space-y-3">
                <div className="flex items-center justify-between text-xs font-semibold text-[var(--color-text)]">
                  <span>{filters.minPrice.toLocaleString()} XLM</span>
                  <span>{filters.maxPrice.toLocaleString()} XLM</span>
                </div>
                <input
                  type="range"
                  min={bounds.min}
                  max={bounds.max}
                  step={100}
                  value={filters.minPrice}
                  onChange={(event) =>
                    updateMinPrice(Number(event.target.value))
                  }
                  className="w-full accent-brand-600"
                  aria-label="Minimum goal amount"
                />
                <input
                  type="range"
                  min={bounds.min}
                  max={bounds.max}
                  step={100}
                  value={filters.maxPrice}
                  onChange={(event) =>
                    updateMaxPrice(Number(event.target.value))
                  }
                  className="w-full accent-brand-600"
                  aria-label="Maximum goal amount"
                />
              </div>
            </FilterSection>

            <FilterSection title="Date range">
              <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-1">
                <DateInput
                  id="start-date"
                  label="From"
                  value={filters.startDate}
                  max={filters.endDate || undefined}
                  onChange={(startDate) => updateFilters({ startDate })}
                />
                <DateInput
                  id="end-date"
                  label="To"
                  value={filters.endDate}
                  min={filters.startDate || undefined}
                  onChange={(endDate) => updateFilters({ endDate })}
                />
              </div>
            </FilterSection>

            <FilterSection title="Status">
              <div className="space-y-2">
                {STATUS_OPTIONS.map((status) => (
                  <CheckboxFilter
                    key={status}
                    checked={filters.statuses.includes(status)}
                    count={statusCounts[status]}
                    label={status}
                    onChange={() => toggleStatus(status)}
                  />
                ))}
              </div>
            </FilterSection>

            <FilterSection title="Category">
              <div className="space-y-2">
                {categories.map((category) => (
                  <CheckboxFilter
                    key={category}
                    checked={filters.categories.includes(category)}
                    count={categoryCounts[category] ?? 0}
                    label={category}
                    labelClassName={
                      filters.categories.includes(category)
                        ? CATEGORY_STYLES[category]?.active
                        : CATEGORY_STYLES[category]?.inactive
                    }
                    onChange={() => toggleCategory(category)}
                  />
                ))}
              </div>
            </FilterSection>

            <button
              type="button"
              onClick={clearAllFilters}
              disabled={activeFilters.length === 0}
              className="w-full rounded-xl border border-dashed border-[var(--color-border)] px-4 py-2.5 text-sm font-semibold text-[var(--color-text-muted)] transition-colors hover:border-brand-500 hover:text-brand-600 disabled:cursor-not-allowed disabled:opacity-50"
            >
              Clear all filters
            </button>
          </div>
        </aside>

        <section className="min-w-0 flex-1">
          <div className="mb-5 flex flex-col gap-3 border-b border-[var(--color-border)] pb-4 md:flex-row md:items-center md:justify-between">
            <div>
              <p className="text-sm font-semibold text-[var(--color-text)]">
                {sortedPools.length} result{sortedPools.length === 1 ? '' : 's'}
              </p>
            </div>

            <label className="flex items-center gap-2 text-xs font-bold uppercase text-[var(--color-text-muted)]">
              Sort
              <select
                value={filters.sortBy}
                onChange={(event) =>
                  updateFilters({ sortBy: event.target.value as SortOption })
                }
                className="rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-xs font-semibold text-[var(--color-text)] outline-none focus:border-brand-500"
              >
                <option value="newest">Newest</option>
                <option value="most-funded">Most funded</option>
                <option value="close-to-goal">Closest to goal</option>
                <option value="trending">Trending</option>
              </select>
            </label>
          </div>

          {activeFilters.length > 0 && (
            <div className="mb-5 flex flex-wrap items-center gap-2">
              {activeFilters.map((filter) => (
                <button
                  key={filter.key}
                  type="button"
                  onClick={filter.remove}
                  className="inline-flex max-w-full items-center gap-2 rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-1.5 text-xs font-semibold text-[var(--color-text)] transition-colors hover:border-brand-500 hover:text-brand-600"
                  aria-label={`Remove ${filter.label} filter`}
                >
                  <span className="truncate">{filter.label}</span>
                  <span aria-hidden="true">x</span>
                </button>
              ))}
            </div>
          )}

          {sortedPools.length === 0 ? (
            <EmptyState
              variant="bordered"
              icon="search"
              iconTone="muted"
              title="No results found"
              description="No pools match the current filter combination."
              action={{
                label: 'Clear all filters',
                onClick: clearAllFilters,
                variant: 'primary',
              }}
              secondaryAction={{
                label: 'Create a Pool',
                href: '/pools/new',
                variant: 'link',
              }}
            />
          ) : (
            <div className="space-y-8">
              <div className="grid gap-6 sm:grid-cols-2 xl:grid-cols-3">
                {paginatedPools.map((pool) => (
                  <PoolCard
                    key={pool.id}
                    pool={pool}
                    donorCount={getDonorCount(pool)}
                  />
                ))}
              </div>

              {sortedPools.length > ITEMS_PER_PAGE && (
                <div className="border-t border-[var(--color-border)] pt-6">
                  <Pagination
                    totalItems={sortedPools.length}
                    itemsPerPage={ITEMS_PER_PAGE}
                    currentPage={currentPage}
                    onPageChange={(page) => updateFilters({ page })}
                    showGoToPage={sortedPools.length > ITEMS_PER_PAGE * 5}
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

function FilterSection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="border-t border-[var(--color-border)] pt-5">
      <h2 className="mb-3 text-xs font-bold uppercase text-[var(--color-text-muted)]">
        {title}
      </h2>
      {children}
    </section>
  );
}

function CheckboxFilter({
  checked,
  count,
  label,
  labelClassName,
  onChange,
}: {
  checked: boolean;
  count: number;
  label: string;
  labelClassName?: string;
  onChange: () => void;
}) {
  return (
    <label className="flex cursor-pointer items-center justify-between gap-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm transition-colors hover:border-brand-500">
      <span className="flex min-w-0 items-center gap-2">
        <input
          type="checkbox"
          checked={checked}
          onChange={onChange}
          className="size-4 rounded border-[var(--color-border)] accent-brand-600"
        />
        <span
          className={`truncate rounded-full px-2 py-0.5 text-xs font-semibold ${
            labelClassName ?? 'text-[var(--color-text)]'
          }`}
        >
          {label}
        </span>
      </span>
      <span className="shrink-0 text-xs font-semibold text-[var(--color-text-muted)]">
        {count}
      </span>
    </label>
  );
}

function DateInput({
  id,
  label,
  value,
  min,
  max,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  min?: string;
  max?: string;
  onChange: (value: string) => void;
}) {
  return (
    <label htmlFor={id} className="block">
      <span className="mb-1 block text-xs font-medium text-[var(--color-text-muted)]">
        {label}
      </span>
      <input
        id={id}
        type="date"
        value={value}
        min={min}
        max={max}
        onChange={(event) => onChange(event.target.value)}
        className="w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm text-[var(--color-text)] outline-none focus:border-brand-500"
      />
    </label>
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
      aria-hidden="true"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
      />
    </svg>
  );
}