'use client';

import React, { useEffect, useState, useMemo } from 'react';
import Link from 'next/link';
import { EmptyState } from '@/components/EmptyState';
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
    filteredPools,
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