export type ValidationRule<T = unknown> = {
  validate: (value: T) => boolean | Promise<boolean>;
  message: string;
};

export const runValidation = async <T>(
  value: T,
  rules: ValidationRule<T>[]
): Promise<string | null> => {
  for (const rule of rules) {
    const isValid = await rule.validate(value);
    if (!isValid) {
      return rule.message;
    }
  }
  return null;
};

export const isRequired = (
  message = 'This field is required'
): ValidationRule => ({
  validate: (value) => {
    if (value === null || value === undefined) return false;
    if (typeof value === 'string') return value.trim().length > 0;
    if (Array.isArray(value)) return value.length > 0;
    return true;
  },
  message,
});

export const isEmail = (
  message = 'Invalid email format'
): ValidationRule<string> => ({
  validate: (value) => {
    if (!value) return true; // Let required rule handle empty values
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(value);
  },
  message,
});

export const isPhone = (
  message = 'Invalid phone number format'
): ValidationRule<string> => ({
  validate: (value) => {
    if (!value) return true;
    const phoneRegex = /^\+?[\d\s-]{10,}$/;
    return phoneRegex.test(value);
  },
  message,
});

export const isUrl = (
  message = 'Invalid URL format'
): ValidationRule<string> => ({
  validate: (value) => {
    if (!value) return true;
    try {
      new URL(value);
      return true;
    } catch {
      return false;
    }
  },
  message,
});

export const isCurrencyAmount = (
  message = 'Invalid currency amount'
): ValidationRule<string | number> => ({
  validate: (value) => {
    if (value === null || value === undefined || value === '') return true;
    // Allow positive numbers with up to 2 decimal places
    const amountRegex = /^\d+(\.\d{1,2})?$/;
    return amountRegex.test(String(value));
  },
  message,
});

export const minLength = (
  min: number,
  message?: string
): ValidationRule<string> => ({
  validate: (value) => {
    if (!value) return true;
    return value.length >= min;
  },
  message: message || `Minimum length is ${min} characters`,
});

export const maxLength = (
  max: number,
  message?: string
): ValidationRule<string> => ({
  validate: (value) => {
    if (!value) return true;
    return value.length <= max;
  },
  message: message || `Maximum length is ${max} characters`,
});
