import React, { FC } from 'react';

export type SkeletonVariant = 'text' | 'card' | 'image';

export interface SkeletonProps {
  variant?: SkeletonVariant;
  /** Number of text lines (only used when variant="text") */
  lines?: number;
  className?: string;
  width?: string;
  height?: string;
}

const base =
  'bg-gray-200 dark:bg-gray-700 rounded animate-pulse motion-reduce:animate-none';

const TextSkeleton: FC<{ lines: number; className: string }> = ({
  lines,
  className,
}) => (
  <div className={`space-y-2 ${className}`} aria-hidden="true">
    {Array.from({ length: lines }).map((_, i) => (
      <div
        key={i}
        className={`${base} h-4 ${i === lines - 1 && lines > 1 ? 'w-3/4' : 'w-full'}`}
      />
    ))}
  </div>
);

const CardSkeleton: FC<{ className: string }> = ({ className }) => (
  <div
    className={`rounded-lg border border-gray-200 dark:border-gray-700 p-4 space-y-3 ${className}`}
    aria-hidden="true"
  >
    <div className={`${base} h-40 w-full rounded-md`} />
    <div className={`${base} h-4 w-3/4`} />
    <div className={`${base} h-4 w-1/2`} />
  </div>
);

const ImageSkeleton: FC<{
  className: string;
  width?: string;
  height?: string;
}> = ({ className, width, height }) => (
  <div
    className={`${base} rounded-md ${className}`}
    style={{ width: width ?? '100%', height: height ?? '200px' }}
    aria-hidden="true"
  />
);

export const Skeleton: FC<SkeletonProps> = ({
  variant = 'text',
  lines = 3,
  className = '',
  width,
  height,
}) => {
  if (variant === 'card') return <CardSkeleton className={className} />;
  if (variant === 'image')
    return (
      <ImageSkeleton className={className} width={width} height={height} />
    );
  return <TextSkeleton lines={lines} className={className} />;
};
