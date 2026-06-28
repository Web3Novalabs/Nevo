export function getStoredAccessToken(): string | null {
  if (typeof window === 'undefined') {
    return null;
  }

  const stored = window.localStorage.getItem('nevo-wallet');
  if (!stored) {
    return null;
  }

  try {
    const parsed = JSON.parse(stored);
    const state = parsed?.state ?? parsed;
    const accessToken =
      typeof state?.accessToken === 'string' ? state.accessToken : null;
    return accessToken;
  } catch {
    return null;
  }
}

export function clearJwt(): void {
  if (typeof window === 'undefined') {
    return;
  }

  window.localStorage.removeItem('nevo-wallet');
}
