import React, { forwardRef, TextareaHTMLAttributes, useId } from 'react';
import { FormField } from './FormField';
import { getControlClassName } from './form-styles';

export interface TextareaProps extends TextareaHTMLAttributes<HTMLTextAreaElement> {
  label?: string;
  helperText?: string;
  error?: string;
  required?: boolean;
}

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  (
    {
      label,
      helperText,
      error,
      required = false,
      className = '',
      id: idProp,
      rows = 4,
      ...props
    },
    ref
  ) => {
    const generatedId = useId();
    const id = idProp ?? generatedId;
    const hasError = Boolean(error);

    const textarea = (
      <textarea
        ref={ref}
        id={id}
        rows={rows}
        required={required}
        aria-invalid={hasError || undefined}
        aria-required={required || undefined}
        className={getControlClassName(
          hasError,
          `${className} resize-y`.trim()
        )}
        {...props}
      />
    );

    if (label || helperText || error) {
      return (
        <FormField
          id={id}
          label={label}
          helperText={helperText}
          error={error}
          required={required}
        >
          {textarea}
        </FormField>
      );
    }

    return textarea;
  }
);

Textarea.displayName = 'Textarea';
