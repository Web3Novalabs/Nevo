import {
  getToken as getStoredToken,
  setToken as setStoredToken,
  clearToken as clearStoredToken,
} from './jwt-storage';

interface JwtPayload {
  exp?: number;
  [key: string]: unknown;
}

function decodeBase64Url(input: string): string | null {
  if (typeof atob === 'undefined') {
    return null;
  }

  try {
    const base64 = input.replace(/-/g, '+').replace(/_/g, '/');
    const padded = base64.padEnd(base64.length + ((4 - (base64.length % 4)) % 4), '=');
    return atob(padded);
  } catch {
    return null;
  }
}

export function decodeTokenPayload(token: string | null): JwtPayload | null {
  if (!token) {
    return null;
  }

  const parts = token.split('.');
  if (parts.length < 2) {
    return null;
  }

  const decoded = decodeBase64Url(parts[1]);
  if (decoded === null) {
    return null;
  }

  try {
    const payload = JSON.parse(decoded);
    if (payload && typeof payload === 'object') {
      return payload as JwtPayload;
    }
    return null;
  } catch {
    return null;
  }
}

export function getTokenExpiryMs(token: string | null): number | null {
  const payload = decodeTokenPayload(token);
  if (!payload || typeof payload.exp !== 'number' || !Number.isFinite(payload.exp)) {
    return null;
  }

  return payload.exp * 1000;
}

export function isTokenExpired(token: string | null, now: number = Date.now()): boolean {
  if (!token) {
    return true;
  }

  const expiryMs = getTokenExpiryMs(token);
  if (expiryMs === null) {
    return false;
  }

  return expiryMs <= now;
}

export function getToken(): string | null {
  return getStoredToken();
}

export function setToken(token: string): void {
  setStoredToken(token);
}

export function clearToken(): void {
  clearStoredToken();
  if (typeof window === 'undefined') {
    return null;
  }

  const token = window.localStorage.getItem(TOKEN_KEY);
  if (!token) {
    return null;
  }

  if (isTokenExpired(token)) {
    window.localStorage.removeItem(TOKEN_KEY);
    return null;
  }

  return token;
}

export function setToken(token: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  if (isTokenExpired(token)) {
    window.localStorage.removeItem(TOKEN_KEY);
    return;
  }

  window.localStorage.setItem(TOKEN_KEY, token);
}

export function clearToken(): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.localStorage.removeItem(TOKEN_KEY);
}