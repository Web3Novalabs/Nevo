import React, { ReactNode } from 'react';

export interface FormFieldProps {
  id: string;
  label?: string;
  required?: boolean;
  error?: string;
  helperText?: string;
  children: ReactNode;
  className?: string;
}

export function FormField({
  id,
  label,
  required = false,
  error,
  helperText,
  children,
  className = '',
}: FormFieldProps) {
  const helperId = helperText ? `${id}-helper` : undefined;
  const errorId = error ? `${id}-error` : undefined;
  const describedBy =
    [helperId, errorId].filter(Boolean).join(' ') || undefined;

  return (
    <div className={className}>
      {label && (
        <label
          htmlFor={id}
          className="mb-1.5 block text-sm font-medium text-[var(--color-text)]"
        >
          {label}
          {required && (
            <span className="ml-1 text-[var(--color-error)]" aria-hidden="true">
              *
            </span>
          )}
        </label>
      )}
      {helperText && (
        <p
          id={helperId}
          className="mb-1.5 text-xs text-[var(--color-text-muted)]"
        >
          {helperText}
        </p>
      )}
      {React.isValidElement(children)
        ? describedBy
          ? React.cloneElement(
              children as React.ReactElement<{ 'aria-describedby'?: string }>,
              {
                'aria-describedby': describedBy,
              }
            )
          : children
        : children}
      {error && (
        <p
          id={errorId}
          role="alert"
          className="mt-1 text-xs text-[var(--color-error)]"
        >
          {error}
        </p>
      )}
    </div>
  );
}
