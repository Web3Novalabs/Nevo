'use client';

import React, { useState, useCallback, KeyboardEvent, useId } from 'react';
import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
} from 'lucide-react';

// ─── Types ───

export interface PaginationProps {
  totalItems: number;
  itemsPerPage?: number;
  currentPage?: number;
  defaultPage?: number;
  onPageChange?: (page: number) => void;
  showGoToPage?: boolean;
  showItemsPerPage?: boolean;
  itemsPerPageOptions?: number[];
  onItemsPerPageChange?: (itemsPerPage: number) => void;
  maxVisiblePages?: number;
  className?: string;
  ariaLabel?: string;
}

// ─── Helpers ───

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max);
}

function buildPageRange(
  current: number,
  total: number,
  maxVisible: number
): (number | '…')[] {
  if (total <= maxVisible) {
    return Array.from({ length: total }, (_, i) => i + 1);
  }

  const sideCount = Math.max(1, Math.floor((maxVisible - 3) / 2));
  const pages: (number | '…')[] = [];

  // Always show first page
  pages.push(1);

  const leftEdge = current - sideCount;
  const rightEdge = current + sideCount;

  if (leftEdge > 2) {
    pages.push('…');
  }

  const rangeStart = clamp(leftEdge, 2, total - 1);
  const rangeEnd = clamp(rightEdge, 2, total - 1);

  for (let p = rangeStart; p <= rangeEnd; p++) {
    pages.push(p);
  }

  if (rightEdge < total - 1) {
    pages.push('…');
  }

  // Always show last page
  if (total > 1) pages.push(total);

  return pages;
}

// ─── Sub-components ───

interface PageButtonProps {
  page: number | '…';
  isCurrent?: boolean;
  disabled?: boolean;
  onClick?: () => void;
  tabIndex?: number;
}

function PageButton({
  page,
  isCurrent,
  disabled,
  onClick,
  tabIndex,
}: PageButtonProps) {
  const isEllipsis = page === '…';

  if (isEllipsis) {
    return (
      <span
        className="
          inline-flex items-end justify-center
          w-9 h-9 pb-1
          text-[var(--color-text-muted)] select-none
          text-sm font-medium tracking-wide
        "
        aria-hidden="true"
      >
        …
      </span>
    );
  }

  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled || isCurrent}
      tabIndex={tabIndex}
      aria-label={`Page ${page}`}
      aria-current={isCurrent ? 'page' : undefined}
      className={`
        relative inline-flex items-center justify-center
        w-9 h-9 rounded-lg
        text-sm font-semibold
        transition-all duration-150 ease-out
        focus-visible:outline-none focus-visible:ring-2
        focus-visible:ring-[var(--color-brand-500)] focus-visible:ring-offset-1
        focus-visible:ring-offset-[var(--color-surface)]
        ${
          isCurrent
            ? `bg-[var(--color-brand-500)] text-white
               shadow-[0_2px_8px_rgba(39,146,110,0.4)]
               cursor-default`
            : `text-[var(--color-text-muted)]
               hover:bg-[var(--color-surface-raised)]
               hover:text-[var(--color-text)]
               active:scale-95`
        }
        disabled:opacity-40 disabled:cursor-not-allowed disabled:active:scale-100
      `}
    >
      {page}
    </button>
  );
}

interface NavButtonProps {
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
  disabled: boolean;
}

function NavButton({ label, icon, onClick, disabled }: NavButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      aria-label={label}
      className={`
        inline-flex items-center justify-center
        w-9 h-9 rounded-lg
        text-[var(--color-text-muted)]
        transition-all duration-150 ease-out
        hover:bg-[var(--color-surface-raised)] hover:text-[var(--color-text)]
        active:scale-95
        focus-visible:outline-none focus-visible:ring-2
        focus-visible:ring-[var(--color-brand-500)] focus-visible:ring-offset-1
        focus-visible:ring-offset-[var(--color-surface)]
        disabled:opacity-30 disabled:cursor-not-allowed disabled:active:scale-100
      `}
    >
      {icon}
    </button>
  );
}

// ─── Main Component ───

export function Pagination({
  totalItems,
  itemsPerPage: itemsPerPageProp = 10,
  currentPage: currentPageProp,
  defaultPage = 1,
  onPageChange,
  showGoToPage = true,
  showItemsPerPage = false,
  itemsPerPageOptions = [10, 25, 50, 100],
  onItemsPerPageChange,
  maxVisiblePages = 7,
  className = '',
  ariaLabel = 'Pagination',
}: PaginationProps) {
  const uid = useId();

  // ── Items-per-page state
  const [localItemsPerPage, setLocalItemsPerPage] = useState(itemsPerPageProp);
  const effectiveItemsPerPage = localItemsPerPage;

  const totalPages = Math.max(1, Math.ceil(totalItems / effectiveItemsPerPage));

  // ── Page state
  const isControlled = currentPageProp !== undefined;
  const [localPage, setLocalPage] = useState(clamp(defaultPage, 1, totalPages));
  const currentPage = isControlled
    ? clamp(currentPageProp, 1, totalPages)
    : clamp(localPage, 1, totalPages);

  const setPage = useCallback(
    (page: number) => {
      const clamped = clamp(page, 1, totalPages);
      if (!isControlled) setLocalPage(clamped);
      onPageChange?.(clamped);
    },
    [isControlled, onPageChange, totalPages]
  );

  // ── Go-to-page ──
  const [goInput, setGoInput] = useState('');

  const commitGoToPage = () => {
    const parsed = parseInt(goInput, 10);
    if (!isNaN(parsed)) setPage(parsed);
    setGoInput('');
  };

  const handleGoKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') commitGoToPage();
  };

  // ── Page range ──
  const pageRange = buildPageRange(currentPage, totalPages, maxVisiblePages);

  // ── Items-per-page handler ──
  const handleItemsPerPageChange = (val: number) => {
    setLocalItemsPerPage(val);
    onItemsPerPageChange?.(val);
    // Reset to page 1 when page size changes
    setPage(1);
  };

  // ── Range label ──
  const rangeStart =
    totalItems === 0 ? 0 : (currentPage - 1) * effectiveItemsPerPage + 1;
  const rangeEnd = Math.min(currentPage * effectiveItemsPerPage, totalItems);

  if (totalItems === 0) {
    return null;
  }

  return (
    <nav
      aria-label={ariaLabel}
      className={`
        flex flex-col gap-3
        sm:flex-row sm:items-center sm:justify-between
        text-sm
        ${className}
      `}
    >
      {/* ── Left: range info + optional items-per-page ── */}
      <div className="flex items-center gap-3 flex-wrap">
        <span className="text-[var(--color-text-muted)] whitespace-nowrap tabular-nums">
          <span className="text-[var(--color-text)] font-medium">
            {rangeStart}–{rangeEnd}
          </span>{' '}
          of{' '}
          <span className="text-[var(--color-text)] font-medium">
            {totalItems.toLocaleString()}
          </span>
        </span>

        {showItemsPerPage && (
          <div className="flex items-center gap-1.5">
            <label
              htmlFor={`${uid}-per-page`}
              className="text-[var(--color-text-muted)] whitespace-nowrap"
            >
              Per page:
            </label>
            <select
              id={`${uid}-per-page`}
              value={effectiveItemsPerPage}
              onChange={(e) => handleItemsPerPageChange(Number(e.target.value))}
              className="
                h-8 pl-2 pr-7 rounded-lg border border-[var(--color-border)]
                bg-[var(--color-surface)] text-[var(--color-text)]
                text-sm font-medium
                appearance-none cursor-pointer
                focus-visible:outline-none focus-visible:ring-2
                focus-visible:ring-[var(--color-brand-500)]
                transition-colors
              "
              style={{
                backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236b7280' stroke-width='2.5'%3E%3Cpath d='M6 9l6 6 6-6'/%3E%3C/svg%3E")`,
                backgroundRepeat: 'no-repeat',
                backgroundPosition: 'right 8px center',
              }}
            >
              {itemsPerPageOptions.map((opt) => (
                <option key={opt} value={opt}>
                  {opt}
                </option>
              ))}
            </select>
          </div>
        )}
      </div>

      {/* ── Center + Right: page controls ── */}
      <div className="flex items-center gap-1 flex-wrap">
        {/* First page */}
        <NavButton
          label="First page"
          icon={<ChevronsLeft size={16} strokeWidth={2.5} />}
          onClick={() => setPage(1)}
          disabled={currentPage === 1}
        />

        {/* Previous page */}
        <NavButton
          label="Previous page"
          icon={<ChevronLeft size={16} strokeWidth={2.5} />}
          onClick={() => setPage(currentPage - 1)}
          disabled={currentPage === 1}
        />

        {/* Page number buttons — hidden on xs, visible sm+ */}
        <div
          className="hidden sm:flex items-center gap-0.5"
          role="group"
          aria-label="Page numbers"
        >
          {pageRange.map((page, idx) => (
            <PageButton
              key={`${page}-${idx}`}
              page={page}
              isCurrent={page === currentPage}
              onClick={page !== '…' ? () => setPage(page as number) : undefined}
            />
          ))}
        </div>

        {/* Mobile: just show "3 / 18" */}
        <span
          className="sm:hidden px-3 py-1.5 rounded-lg bg-[var(--color-surface-raised)]
            border border-[var(--color-border)] text-[var(--color-text)] font-semibold
            tabular-nums text-sm whitespace-nowrap"
          aria-live="polite"
          aria-atomic="true"
        >
          {currentPage} / {totalPages}
        </span>

        {/* Next page */}
        <NavButton
          label="Next page"
          icon={<ChevronRight size={16} strokeWidth={2.5} />}
          onClick={() => setPage(currentPage + 1)}
          disabled={currentPage === totalPages}
        />

        {/* Last page */}
        <NavButton
          label="Last page"
          icon={<ChevronsRight size={16} strokeWidth={2.5} />}
          onClick={() => setPage(totalPages)}
          disabled={currentPage === totalPages}
        />

        {/* Go-to-page input */}
        {showGoToPage && totalPages > 5 && (
          <div className="flex items-center gap-1.5 ml-2 pl-2 border-l border-[var(--color-border)]">
            <label
              htmlFor={`${uid}-goto`}
              className="text-[var(--color-text-muted)] whitespace-nowrap hidden sm:block"
            >
              Go to
            </label>
            <input
              id={`${uid}-goto`}
              type="number"
              min={1}
              max={totalPages}
              value={goInput}
              onChange={(e) => setGoInput(e.target.value)}
              onKeyDown={handleGoKeyDown}
              onBlur={commitGoToPage}
              placeholder="…"
              aria-label={`Go to page, 1 to ${totalPages}`}
              className="
                w-14 h-8 px-2 rounded-lg text-center
                border border-[var(--color-border)]
                bg-[var(--color-surface)] text-[var(--color-text)]
                text-sm font-medium tabular-nums
                placeholder:text-[var(--color-text-muted)]
                focus-visible:outline-none focus-visible:ring-2
                focus-visible:ring-[var(--color-brand-500)]
                [appearance:textfield]
                [&::-webkit-inner-spin-button]:appearance-none
                [&::-webkit-outer-spin-button]:appearance-none
                transition-colors
              "
            />
          </div>
        )}
      </div>
    </nav>
  );
}

export default Pagination;
