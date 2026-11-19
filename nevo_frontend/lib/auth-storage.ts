import {
  getToken as getStoredToken,
  setToken as setStoredToken,
  clearToken as clearStoredToken,
} from './jwt-storage';

export function getToken(): string | null {
  return getStoredToken();
}

export function setToken(token: string): void {
  setStoredToken(token);
}

export function clearToken(): void {
  clearStoredToken();
}