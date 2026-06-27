import React, { forwardRef, InputHTMLAttributes, useId } from 'react';
import { FormField } from './FormField';
import { getControlClassName } from './form-styles';

export type InputType = 'text' | 'email' | 'number' | 'password';

export interface InputProps extends Omit<
  InputHTMLAttributes<HTMLInputElement>,
  'type'
> {
  type?: InputType;
  label?: string;
  helperText?: string;
  error?: string;
  required?: boolean;
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  (
    {
      type = 'text',
      label,
      helperText,
      error,
      required = false,
      className = '',
      id: idProp,
      ...props
    },
    ref
  ) => {
    const generatedId = useId();
    const id = idProp ?? generatedId;
    const hasError = Boolean(error);

    const input = (
      <input
        ref={ref}
        id={id}
        type={type}
        required={required}
        aria-invalid={hasError || undefined}
        aria-required={required || undefined}
        className={getControlClassName(hasError, className)}
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
          {input}
        </FormField>
      );
    }

    return input;
  }
);

Input.displayName = 'Input';
