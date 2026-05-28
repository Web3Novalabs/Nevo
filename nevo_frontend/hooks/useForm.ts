import { useState, useCallback, useRef } from 'react';
import { ValidationRule, runValidation } from '../lib/validation';

export type FieldConfig<T> = {
  initialValue: T;
  rules?: ValidationRule<T>[];
};

export type FormConfig<T extends Record<string, unknown>> = {
  [K in keyof T]: FieldConfig<T[K]>;
};

export function useForm<T extends Record<string, unknown>>(
  initialConfig: FormConfig<T>
) {
  const configRef = useRef(initialConfig);

  const [initialValues] = useState(() =>
    Object.keys(initialConfig).reduce((acc, key) => {
      acc[key as keyof T] = initialConfig[key as keyof T].initialValue;
      return acc;
    }, {} as T)
  );

  const [values, setValues] = useState<T>(initialValues);
  const [errors, setErrors] = useState<Record<keyof T, string | null>>(() =>
    Object.keys(initialConfig).reduce(
      (acc, key) => {
        acc[key as keyof T] = null;
        return acc;
      },
      {} as Record<keyof T, string | null>
    )
  );
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [touched, setTouched] = useState<Record<keyof T, boolean>>(() =>
    Object.keys(initialConfig).reduce(
      (acc, key) => {
        acc[key as keyof T] = false;
        return acc;
      },
      {} as Record<keyof T, boolean>
    )
  );

  const validateField = useCallback(async (name: keyof T, value: unknown) => {
    const rules = configRef.current[name]?.rules || [];
    const error = await runValidation(value as T[keyof T], rules);
    setErrors((prev) => ({ ...prev, [name]: error }));
    return error === null;
  }, []);

  const handleChange = useCallback(
    (name: keyof T, value: unknown) => {
      setValues((prev) => ({ ...prev, [name]: value as T[keyof T] }));
      if (touched[name]) {
        validateField(name, value);
      }
    },
    [touched, validateField]
  );

  const handleBlur = useCallback(
    (name: keyof T) => {
      setTouched((prev) => ({ ...prev, [name]: true }));
      validateField(name, values[name]);
    },
    [values, validateField]
  );

  const validateAll = useCallback(async () => {
    const newErrors: Record<keyof T, string | null> = {} as Record<
      keyof T,
      string | null
    >;
    let isValid = true;

    // Mark all as touched
    const allTouched = Object.keys(configRef.current).reduce(
      (acc, key) => {
        acc[key as keyof T] = true;
        return acc;
      },
      {} as Record<keyof T, boolean>
    );
    setTouched(allTouched);

    for (const key of Object.keys(configRef.current)) {
      const name = key as keyof T;
      const rules = configRef.current[name]?.rules || [];
      const error = await runValidation(values[name], rules);
      newErrors[name] = error;
      if (error) {
        isValid = false;
      }
    }
    setErrors(newErrors);
    return isValid;
  }, [values]);

  const handleSubmit = (onSubmit: (values: T) => void | Promise<void>) => {
    return async (e?: React.FormEvent) => {
      if (e && e.preventDefault) {
        e.preventDefault();
      }
      setIsSubmitting(true);
      const isValid = await validateAll();
      if (isValid) {
        await onSubmit(values);
      }
      setIsSubmitting(false);
    };
  };

  const reset = useCallback(() => {
    setValues(initialValues);
    setErrors(
      Object.keys(configRef.current).reduce(
        (acc, key) => {
          acc[key as keyof T] = null;
          return acc;
        },
        {} as Record<keyof T, string | null>
      )
    );
    setTouched(
      Object.keys(configRef.current).reduce(
        (acc, key) => {
          acc[key as keyof T] = false;
          return acc;
        },
        {} as Record<keyof T, boolean>
      )
    );
  }, [initialValues]);

  return {
    values,
    errors,
    touched,
    isSubmitting,
    handleChange,
    handleBlur,
    handleSubmit,
    validateField,
    reset,
    setValues,
  };
}
