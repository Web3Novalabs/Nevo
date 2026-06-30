import { ApiError } from './api-client';

const FALLBACK_MESSAGE = 'Something went wrong. Please try again.';

interface AxiosErrorResponseData {
  message?: unknown;
  error?: unknown;
}

interface AxiosLikeError {
  isAxiosError?: boolean;
  response?: {
    data?: AxiosErrorResponseData;
  };
  message?: unknown;
}

function isAxiosError(err: unknown): err is AxiosLikeError {
  if (typeof err !== 'object' || err === null) {
    return false;
  }

  const candidate = err as AxiosLikeError;
  if (candidate.isAxiosError === true) {
    return true;
  }

  return (
    'response' in candidate &&
    typeof candidate.response === 'object' &&
    candidate.response !== null
  );
}

function extractString(value: unknown): string | undefined {
  return typeof value === 'string' && value.length > 0 ? value : undefined;
}

function extractFromPayload(payload: unknown): string | undefined {
  if (typeof payload !== 'object' || payload === null) {
    return undefined;
  }

  const record = payload as AxiosErrorResponseData;
  return extractString(record.message) ?? extractString(record.error);
}

export function parseApiError(err: unknown): string {
  if (isAxiosError(err)) {
    const fromResponse = extractFromPayload(err.response?.data);
    if (fromResponse) {
      return fromResponse;
    }
  }

  if (err instanceof ApiError) {
    const fromData = extractFromPayload(err.data);
    if (fromData) {
      return fromData;
    }
  }

  if (err instanceof Error && err.message) {
    return err.message;
  }

  if (
    typeof err === 'object' &&
    err !== null &&
    'message' in err &&
    typeof (err as { message: unknown }).message === 'string'
  ) {
    const message = (err as { message: string }).message;
    if (message.length > 0) {
      return message;
    }
  }

  return FALLBACK_MESSAGE;
}
