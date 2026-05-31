import React, { FC } from 'react';
import Link from 'next/link';
import { Card, CardBody } from './Card';
import { Avatar } from './Avatar';
import type { Pool } from '@/src/store/poolsStore';

export interface PoolCardProps {
  pool: Pool;
  donorCount?: number;
  className?: string;
}

export const PoolCard: FC<PoolCardProps> = ({
  pool,
  donorCount,
  className = '',
}) => {
  const {
    id,
    title,
    description,
    category,
    status,
    target,
    raised,
    imageColor,
    creator,
  } = pool;

  // Calculate progress percentage
  const progressPercent = Math.min(100, Math.round((raised / target) * 100));

  // Determine open/closed display text & styling
  const isOpen = status === 'Active';
  const statusLabel = isOpen ? 'Open' : 'Closed';
  const statusBadgeStyle = isOpen
    ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20'
    : 'bg-zinc-500/10 text-zinc-600 dark:text-zinc-400 border border-zinc-500/20';

  // Format creator address
  const truncatedCreator = creator
    ? `${creator.slice(0, 6)}...${creator.slice(-4)}`
    : 'Anonymous';

  // Mock donor count if not provided
  const displayDonorCount =
    donorCount ??
    (id === '1'
      ? 42
      : id === '2'
        ? 87
        : id === '3'
          ? 31
          : Math.floor((raised * 7.3) / 100) + 1);

  return (
    <Link href={`/pools/${id}`} className="block group no-underline">
      <Card
        variant="elevated"
        hoverable
        className={`flex flex-col h-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded-2xl overflow-hidden transition-all duration-300 group-hover:border-brand-500/30 ${className}`}
      >
        {/* Banner Section */}
        <div
          className="h-28 w-full relative transition-all duration-500 group-hover:saturate-110"
          style={{ backgroundColor: imageColor || '#e5e7eb' }}
        >
          {/* Decorative glassmorphic gradient overlay */}
          <div className="absolute inset-0 bg-gradient-to-t from-black/20 to-transparent" />

          {/* Badges overlay */}
          <div className="absolute top-4 left-4 right-4 flex items-center justify-between z-10">
            <span className="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold bg-white/90 dark:bg-zinc-900/90 backdrop-blur-sm text-zinc-800 dark:text-zinc-200 shadow-sm border border-white/20">
              {category}
            </span>
            <span
              className={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-semibold backdrop-blur-sm shadow-sm ${statusBadgeStyle}`}
            >
              <span
                className={`h-1.5 w-1.5 rounded-full mr-1.5 animate-pulse ${isOpen ? 'bg-emerald-500' : 'bg-zinc-500'}`}
              />
              {statusLabel}
            </span>
          </div>
        </div>

        {/* Content Section */}
        <CardBody className="flex flex-col flex-1 p-5">
          {/* Title */}
          <h3 className="font-bold text-lg text-[var(--color-text)] leading-snug group-hover:text-brand-500 transition-colors duration-200 line-clamp-1 mb-2">
            {title}
          </h3>

          {/* Creator Information Section */}
          <div className="flex items-center gap-2 mb-4">
            <Avatar
              name={creator || 'Anonymous'}
              size="sm"
              className="ring-2 ring-[var(--color-border)]"
            />
            <div className="min-w-0">
              <span className="block text-xs font-semibold text-[var(--color-text-muted)]">
                Creator
              </span>
              <span
                className="block text-xs font-mono text-[var(--color-text)] truncate"
                title={creator}
              >
                {truncatedCreator}
              </span>
            </div>
          </div>

          {/* Description */}
          <p className="text-sm text-[var(--color-text-muted)] leading-relaxed line-clamp-2 mb-5 flex-1">
            {description}
          </p>

          {/* Progress Section */}
          <div className="mt-auto space-y-2">
            <div className="flex items-center justify-between text-xs font-semibold">
              <span className="text-zinc-500 dark:text-zinc-400">Progress</span>
              <span className="text-brand-600 dark:text-brand-400">
                {progressPercent}%
              </span>
            </div>

            {/* Progress Bar Container */}
            <div className="h-2 w-full bg-zinc-100 dark:bg-zinc-800 rounded-full overflow-hidden">
              <div
                className="h-full rounded-full bg-gradient-to-r from-brand-500 to-brand-600 transition-all duration-500 ease-out"
                style={{ width: `${progressPercent}%` }}
              />
            </div>

            {/* Raised & Goal Details */}
            <div className="flex items-center justify-between pt-1 text-xs text-[var(--color-text-muted)]">
              <span className="font-medium text-[var(--color-text)]">
                {raised.toLocaleString()} XLM raised
              </span>
              <span>Goal: {target.toLocaleString()} XLM</span>
            </div>
          </div>
        </CardBody>

        {/* Footer Metrics Section */}
        <div className="px-5 py-3 border-t border-[var(--color-border)] bg-[var(--color-surface-raised)]/50 flex items-center justify-between text-xs">
          {/* Donor Count */}
          <div className="flex items-center gap-1.5 text-[var(--color-text-muted)] font-medium">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={2}
              stroke="currentColor"
              className="w-4 h-4 text-brand-500"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M18 18.72a9.094 9.094 0 003.741-.479 3 3 0 00-4.682-2.72m.94 3.198l.001.031c0 .225-.012.447-.037.666A11.944 11.944 0 0112 21c-2.17 0-4.207-.576-5.963-1.584A6.062 6.062 0 016 18.719m12 0a5.971 5.971 0 00-.941-3.197m0 0A5.995 5.995 0 0012 12.75a5.995 5.995 0 00-5.058 2.772m0 0a3 3 0 00-4.681 2.72 8.986 8.986 0 003.74.477m.94-3.197a5.971 5.971 0 00-.94 3.197M15 6.75a3 3 0 11-6 0 3 3 0 016 0zm6 3a2.25 2.25 0 11-4.5 0 2.25 2.25 0 014.5 0zm-13.5 0a2.25 2.25 0 11-4.5 0 2.25 2.25 0 014.5 0z"
              />
            </svg>
            <span>
              {displayDonorCount} donor{displayDonorCount === 1 ? '' : 's'}
            </span>
          </div>

          {/* Quick link action text */}
          <span className="font-semibold text-brand-600 dark:text-brand-400 group-hover:translate-x-0.5 transition-transform duration-200 inline-flex items-center gap-1">
            Donate
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={2.5}
              stroke="currentColor"
              className="w-3 h-3"
              aria-hidden="true"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M8.25 4.5l7.5 7.5-7.5 7.5"
              />
            </svg>
          </span>
        </div>
      </Card>
    </Link>
  );
};
