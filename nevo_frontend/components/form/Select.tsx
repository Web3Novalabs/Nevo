import React, { forwardRef, SelectHTMLAttributes, useId } from 'react';
import { FormField } from './FormField';
import { getSelectClassName } from './form-styles';

export interface SelectOption {
  label: string;
  value: string;
  disabled?: boolean;
}

export interface SelectProps extends Omit<
  SelectHTMLAttributes<HTMLSelectElement>,
  'children'
> {
  options: SelectOption[];
  placeholder?: string;
  label?: string;
  helperText?: string;
  error?: string;
  required?: boolean;
}

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  (
    {
      options,
      placeholder,
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

    const select = (
      <select
        ref={ref}
        id={id}
        required={required}
        aria-invalid={hasError || undefined}
        aria-required={required || undefined}
        className={getSelectClassName(hasError, className)}
        {...props}
      >
        {placeholder && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((option) => (
          <option
            key={option.value}
            value={option.value}
            disabled={option.disabled}
          >
            {option.label}
          </option>
        ))}
      </select>
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
          {select}
        </FormField>
      );
    }

    return select;
  }
);

Select.displayName = 'Select';
