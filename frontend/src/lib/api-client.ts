export const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || '';

export const getApiUrl = (path: string): string => {
  return `${API_BASE_URL}${path}`;
};

const JWT_KEY = 'nevo_jwt';

export const getJwt = (): string | null => {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem(JWT_KEY);
};

export const setJwt = (token: string): void => {
  if (typeof window === 'undefined') return;
  localStorage.setItem(JWT_KEY, token);
};

export const clearJwt = (): void => {
  if (typeof window === 'undefined') return;
  localStorage.removeItem(JWT_KEY);
};