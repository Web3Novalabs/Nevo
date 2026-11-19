import { clearToken, getToken, getTokenExpiry, isTokenExpired } from './auth-storage';

export function getStoredAccessToken(): string | null {
  return getToken();
}

export function getStoredAccessTokenExpiry(): number | null {
  return getTokenExpiry();
}

export function isStoredTokenExpired(): boolean {
  return isTokenExpired();
}

export function clearJwt(): void {
  clearToken();
}