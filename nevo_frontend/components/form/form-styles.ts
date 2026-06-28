export function getControlClassName(hasError: boolean, className = ''): string {
  return [
    'w-full rounded-xl border px-3.5 py-2.5 text-sm',
    'bg-[var(--color-surface)] text-[var(--color-text)]',
    'placeholder:text-[var(--color-text-muted)]',
    'focus:outline-none focus:ring-2 transition-colors',
    'disabled:cursor-not-allowed disabled:opacity-50',
    hasError
      ? 'border-[var(--color-error)] focus:ring-[var(--color-error)]'
      : 'border-[var(--color-border)] focus:ring-brand-500',
    className,
  ]
    .filter(Boolean)
    .join(' ');
}

export function getSelectClassName(hasError: boolean, className = ''): string {
  return [
    getControlClassName(hasError),
    'appearance-none bg-[length:1rem] bg-[right_0.75rem_center] bg-no-repeat pr-10',
    "bg-[url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 24 24' stroke='%236b7280' stroke-width='2'%3E%3Cpath stroke-linecap='round' stroke-linejoin='round' d='M19 9l-7 7-7-7'/%3E%3C/svg%3E\")]",
    className,
  ]
    .filter(Boolean)
    .join(' ');
}

export function getCheckboxClassName(hasError: boolean): string {
  return [
    'h-4 w-4 shrink-0 rounded border',
    'text-brand-600 focus:ring-2 focus:ring-offset-0',
    'disabled:cursor-not-allowed disabled:opacity-50',
    hasError
      ? 'border-[var(--color-error)] focus:ring-[var(--color-error)]'
      : 'border-[var(--color-border)] focus:ring-brand-500',
  ].join(' ');
}
