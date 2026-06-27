import React, { forwardRef, InputHTMLAttributes, useId } from 'react';
import { getCheckboxClassName } from './form-styles';

export interface CheckboxProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  'type'
> {
  label: string;
  helperText?: string;
  error?: string;
}

export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(
  (
    {
      label,
      helperText,
      error,
      className = '',
      id: idProp,
      required = false,
      ...props
    },
    ref
  ) => {
    const generatedId = useId();
    const id = idProp ?? generatedId;
    const hasError = Boolean(error);
    const helperId = helperText ? `${id}-helper` : undefined;
    const errorId = error ? `${id}-error` : undefined;
    const describedBy =
      [helperId, errorId].filter(Boolean).join(' ') || undefined;

    return (
      <div className={className}>
        <div className="flex items-start gap-2.5">
          <input
            ref={ref}
            id={id}
            type="checkbox"
            required={required}
            aria-invalid={hasError || undefined}
            aria-required={required || undefined}
            aria-describedby={describedBy}
            className={getCheckboxClassName(hasError)}
            {...props}
          />
          <div className="min-w-0 flex-1">
            <label
              htmlFor={id}
              className="text-sm font-medium text-[var(--color-text)] cursor-pointer"
            >
              {label}
              {required && (
                <span
                  className="ml-1 text-[var(--color-error)]"
                  aria-hidden="true"
                >
                  *
                </span>
              )}
            </label>
            {helperText && (
              <p
                id={helperId}
                className="mt-0.5 text-xs text-[var(--color-text-muted)]"
              >
                {helperText}
              </p>
            )}
          </div>
        </div>
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
);

Checkbox.displayName = 'Checkbox';
