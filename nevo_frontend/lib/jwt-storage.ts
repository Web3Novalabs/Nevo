import { clearToken, getToken } from './auth-storage';

export function getStoredAccessToken(): string | null {
  return getToken();
}

export function clearJwt(): void {
  clearToken();
}
