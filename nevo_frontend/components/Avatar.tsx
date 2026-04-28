import React, { FC, useState } from 'react';

export interface AvatarProps {
  name?: string;
  src?: string;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

const sizes = {
  sm: 'h-8 w-8 text-xs',
  md: 'h-10 w-10 text-sm',
  lg: 'h-14 w-14 text-base',
};

function getInitials(name?: string): string {
  if (!name) return '?';
  const parts = name.trim().split(/\s+/);
  return parts.length === 1
    ? parts[0][0].toUpperCase()
    : (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}

export const Avatar: FC<AvatarProps> = ({
  name,
  src,
  size = 'md',
  className = '',
}) => {
  const [status, setStatus] = useState<'loading' | 'loaded' | 'error'>(
    src ? 'loading' : 'error'
  );
  const base = `inline-flex items-center justify-center rounded-full overflow-hidden shrink-0 ${sizes[size]} ${className}`;

  if (status === 'loading') {
    return (
      <span
        className={`${base} bg-gray-200 animate-pulse`}
        aria-label="Loading avatar"
        role="img"
      >
        {src && (
          <img
            src={src}
            alt={name ?? 'User avatar'}
            className="h-full w-full object-cover"
            onLoad={() => setStatus('loaded')}
            onError={() => setStatus('error')}
            style={{ display: 'none' }}
          />
        )}
      </span>
    );
  }

  if (src && status === 'loaded') {
    return (
      <span className={base}>
        <img
          src={src}
          alt={name ?? 'User avatar'}
          className="h-full w-full object-cover"
          onError={() => setStatus('error')}
        />
      </span>
    );
  }

  // fallback: initials
  return (
    <span
      className={`${base} bg-blue-600 text-white font-medium select-none`}
      role="img"
      aria-label={name ? `Avatar for ${name}` : 'User avatar'}
    >
      {getInitials(name)}
    </span>
  );
};
