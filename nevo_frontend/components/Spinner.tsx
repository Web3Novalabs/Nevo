import React, { FC } from 'react';

export type SpinnerSize = 'sm' | 'md' | 'lg' | 'xl';

export interface SpinnerProps {
  size?: SpinnerSize;
  color?: string;
  className?: string;
  label?: string;
}

const sizes: Record<SpinnerSize, string> = {
  sm: 'h-4 w-4',
  md: 'h-6 w-6',
  lg: 'h-8 w-8',
  xl: 'h-12 w-12',
};

export const Spinner: FC<SpinnerProps> = ({
  size = 'md',
  color = 'currentColor',
  className = '',
  label = 'Loading…',
}) => (
  <svg
    className={`animate-spin ${sizes[size]} ${className}`}
    viewBox="0 0 24 24"
    fill="none"
    aria-label={label}
    role="status"
    style={{ color }}
  >
    <circle
      className="opacity-25"
      cx="12"
      cy="12"
      r="10"
      stroke="currentColor"
      strokeWidth="4"
    />
    <path
      className="opacity-75"
      fill="currentColor"
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
    />
  </svg>
);
