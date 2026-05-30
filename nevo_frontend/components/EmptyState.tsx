'use client';

import Link from 'next/link';
import type { ReactNode } from 'react';

export type EmptyStateIcon =
  | 'pool'
  | 'search'
  | 'transaction'
  | 'wallet'
  | 'contributors'
  | 'history'
  | 'not-found';

export type EmptyStateVariant = 'default' | 'bordered' | 'compact';

export interface EmptyStateAction {
  label: string;
  href?: string;
  onClick?: () => void;
  variant?: 'primary' | 'secondary' | 'link';
}

export interface EmptyStateStep {
  text: string;
}

export interface EmptyStateProps {
  title: string;
  description?: string;
  icon?: EmptyStateIcon;
  iconTone?: 'brand' | 'muted';
  variant?: EmptyStateVariant;
  action?: EmptyStateAction;
  secondaryAction?: EmptyStateAction;
  steps?: EmptyStateStep[];
  children?: ReactNode;
  className?: string;
}

const variantStyles: Record<EmptyStateVariant, string> = {
  default:
    'flex flex-col items-center gap-4 px-4 py-12 sm:py-16 text-center',
  bordered:
    'flex flex-col items-center justify-center gap-4 rounded-2xl border border-dashed border-[var(--color-border)] bg-[var(--color-surface-raised)] px-4 py-16 sm:py-20 text-center',
  compact:
    'flex flex-col items-center gap-3 px-2 py-8 text-center',
};

const iconToneStyles = {
  brand: 'bg-brand-100 text-brand-600',
  muted: 'bg-[var(--color-surface-raised)] text-[var(--color-text-muted)]',
};

function EmptyStateIconGraphic({
  name,
  className = 'size-7',
}: {
  name: EmptyStateIcon;
  className?: string;
}) {
  const props = {
    xmlns: 'http://www.w3.org/2000/svg',
    fill: 'none' as const,
    viewBox: '0 0 24 24',
    strokeWidth: 1.5,
    stroke: 'currentColor',
    className,
    'aria-hidden': true,
  };

  switch (name) {
    case 'search':
      return (
        <svg {...props}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
          />
        </svg>
      );
    case 'wallet':
      return (
        <svg {...props}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M16.5 10.5V6.75a4.5 4.5 0 1 0-9 0v3.75m-.75 11.25h10.5a2.25 2.25 0 0 0 2.25-2.25v-6.75a2.25 2.25 0 0 0-2.25-2.25H6.75a2.25 2.25 0 0 0-2.25 2.25v6.75a2.25 2.25 0 0 0 2.25 2.25Z"
          />
        </svg>
      );
    case 'transaction':
      return (
        <svg {...props}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M9 12h3.75M9 15h3.75M9 18h3.75m3 .75H18a2.25 2.25 0 0 0 2.25-2.25V6.108c0-1.135-.845-2.098-1.976-2.192a48.424 48.424 0 0 0-1.123-.08m-5.801 0c-.065.21-.1.433-.1.664 0 .414.336.75.75.75h4.5a.75.75 0 0 0 .75-.75 2.25 2.25 0 0 0-.1-.664m-5.8 0A2.251 2.251 0 0 1 13.5 2.25H15c1.012 0 1.867.668 2.15 1.586m-5.8 0c-.376.023-.75.05-1.124.08C9.095 4.01 8.25 4.973 8.25 6.108V8.25m0 0H4.875c-.621 0-1.125.504-1.125 1.125v11.25c0 .621.504 1.125 1.125 1.125h9.75c.621 0 1.125-.504 1.125-1.125V9.375c0-.621-.504-1.125-1.125-1.125H8.25ZM6.75 12h.008v.008H6.75V12Zm0 3h.008v.008H6.75V15Zm0 3h.008v.008H6.75V18Z"
          />
        </svg>
      );
    case 'contributors':
      return (
        <svg {...props}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M15 19.128a9.38 9.38 0 0 0 2.625.372 9.337 9.337 0 0 0 4.121-.952 4.125 4.125 0 0 0-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 0 1 8.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.21a6.375 6.375 0 0 1 11.964-3.07M12 6.375a3.375 3.375 0 1 1-6.75 0 3.375 3.375 0 0 1 6.75 0Zm8.25 2.25a2.625 2.625 0 1 1-5.25 0 2.625 2.625 0 0 1 5.25 0Z"
          />
        </svg>
      );
    case 'history':
      return (
        <svg {...props}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
          />
        </svg>
      );
    case 'not-found':
      return (
        <svg {...props}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M9.879 7.519c1.171-1.025 3.071-1.025 4.242 0 1.172 1.025 1.172 2.687 0 3.712-.203.179-.43.326-.67.442-.745.361-1.45.999-1.45 1.827v.75M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 5.25h.008v.008H12v-.008Z"
          />
        </svg>
      );
    case 'pool':
    default:
      return (
        <svg {...props}>
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M12 6v12m-3-2.818.879.659c1.171.879 3.07.879 4.242 0 1.172-.879 1.172-2.303 0-3.182C13.536 12.219 12.768 12 12 12c-.725 0-1.45-.22-2.003-.659-1.106-.879-1.106-2.303 0-3.182s2.9-.879 4.006 0l.415.33M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
          />
        </svg>
      );
  }
}

function ActionButton({ action }: { action: EmptyStateAction }) {
  const base =
    action.variant === 'link'
      ? 'text-sm font-medium text-brand-600 hover:text-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600'
      : action.variant === 'secondary'
        ? 'rounded-full border border-[var(--color-border)] px-5 py-2 text-sm font-medium hover:bg-[var(--color-surface-raised)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600'
        : 'rounded-full bg-brand-600 px-6 py-2 text-sm font-semibold text-white hover:bg-brand-700 transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600';

  if (action.href) {
    return (
      <Link href={action.href} className={base}>
        {action.label}
      </Link>
    );
  }

  return (
    <button type="button" onClick={action.onClick} className={base}>
      {action.label}
    </button>
  );
}

export function EmptyState({
  title,
  description,
  icon = 'pool',
  iconTone = 'brand',
  variant = 'default',
  action,
  secondaryAction,
  steps,
  children,
  className = '',
}: EmptyStateProps) {
  const iconSize = variant === 'compact' ? 'size-12' : 'size-14';
  const iconGraphicSize = variant === 'compact' ? 'size-6' : 'size-7';

  return (
    <div
      role="status"
      className={`${variantStyles[variant]} ${className}`.trim()}
    >
      <div
        className={`flex ${iconSize} items-center justify-center rounded-full ${iconToneStyles[iconTone]}`}
      >
        <EmptyStateIconGraphic name={icon} className={iconGraphicSize} />
      </div>

      <div className="flex max-w-md flex-col gap-2">
        <h3
          className={
            variant === 'compact'
              ? 'text-sm font-semibold'
              : 'text-base font-semibold sm:text-lg'
          }
        >
          {title}
        </h3>
        {description && (
          <p className="text-sm text-[var(--color-text-muted)]">{description}</p>
        )}
      </div>

      {steps && steps.length > 0 && (
        <ol className="mt-1 flex w-full max-w-sm flex-col gap-2 text-left text-sm text-[var(--color-text-muted)]">
          {steps.map((step, index) => (
            <li key={index} className="flex gap-3">
              <span
                className="flex size-6 shrink-0 items-center justify-center rounded-full bg-brand-100 text-xs font-semibold text-brand-700"
                aria-hidden="true"
              >
                {index + 1}
              </span>
              <span className="pt-0.5">{step.text}</span>
            </li>
          ))}
        </ol>
      )}

      {children}

      {(action || secondaryAction) && (
        <div className="mt-2 flex flex-col items-center gap-3 sm:flex-row sm:justify-center">
          {action && <ActionButton action={action} />}
          {secondaryAction && <ActionButton action={secondaryAction} />}
        </div>
      )}
    </div>
  );
}
