import React, { FC } from 'react';

export interface ProgressBarProps {
  /** Current value (e.g. amount raised) */
  value: number;
  /** Maximum value (e.g. funding goal) */
  max: number;
  /** Optional label shown above the bar */
  label?: string;
  /** Optional formatted value string (e.g. "500 XLM") */
  valueLabel?: string;
  /** Optional formatted max string (e.g. "1,000 XLM") */
  maxLabel?: string;
  /** Color variant */
  variant?: 'brand' | 'success' | 'warning' | 'error';
  /** Bar height */
  size?: 'sm' | 'md' | 'lg';
  /** Show percentage text inside/beside bar */
  showPercent?: boolean;
  className?: string;
}

const variantTrack: Record<NonNullable<ProgressBarProps['variant']>, string> = {
  brand: 'bg-brand-100',
  success: 'bg-success-light',
  warning: 'bg-warning-light',
  error: 'bg-error-light',
};

const variantFill: Record<NonNullable<ProgressBarProps['variant']>, string> = {
  brand: 'bg-brand-500',
  success: 'bg-success',
  warning: 'bg-warning',
  error: 'bg-error',
};

const sizeClass: Record<NonNullable<ProgressBarProps['size']>, string> = {
  sm: 'h-1.5',
  md: 'h-2.5',
  lg: 'h-4',
};

export const ProgressBar: FC<ProgressBarProps> = ({
  value,
  max,
  label,
  valueLabel,
  maxLabel,
  variant = 'brand',
  size = 'md',
  showPercent = false,
  className = '',
}) => {
  const pct = max > 0 ? Math.min(100, Math.round((value / max) * 100)) : 0;

  return (
    <div className={`w-full ${className}`}>
      {(label || showPercent) && (
        <div className="flex justify-between items-baseline mb-1">
          {label && (
            <span className="text-sm font-medium text-[var(--color-text)]">
              {label}
            </span>
          )}
          {showPercent && (
            <span className="text-xs text-[var(--color-text-muted)]">
              {pct}%
            </span>
          )}
        </div>
      )}

      <div
        className={`w-full rounded-full overflow-hidden ${sizeClass[size]} ${variantTrack[variant]}`}
        role="progressbar"
        aria-valuenow={value}
        aria-valuemin={0}
        aria-valuemax={max}
        aria-label={label ?? 'Progress'}
      >
        <div
          className={`h-full rounded-full transition-all duration-500 ${variantFill[variant]}`}
          style={{ width: `${pct}%` }}
        />
      </div>

      {(valueLabel || maxLabel) && (
        <div className="flex justify-between mt-1">
          <span className="text-xs text-[var(--color-text-muted)]">
            {valueLabel ?? value}
          </span>
          <span className="text-xs text-[var(--color-text-muted)]">
            {maxLabel ?? max}
          </span>
        </div>
      )}
    </div>
  );
};
