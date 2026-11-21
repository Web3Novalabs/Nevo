import { ApiError } from './api-client';

const FALLBACK_MESSAGE = 'Something went wrong. Please try again.';

/**
 * Maps on-chain `ContractError` numeric codes to user-facing messages.
 *
 * Codes must stay in sync with `nevo_contract/ERRORS.md`. Each variant has a
 * unique discriminant; `TokenTransferFailed` (28) and `InvalidTokenAddress`
 * (29) were reassigned to resolve duplicate discriminants 21 and 24.
 */
export const CONTRACT_ERROR_MESSAGES: Record<number, string> = {
  1: 'Unauthorized: you are not allowed to perform this action.',
  2: 'The contract is paused.',
  3: 'Invalid input provided.',
  4: 'The requested resource was not found.',
  5: 'Insufficient balance.',
  6: 'Insufficient allowance.',
  7: 'Token not found.',
  8: 'Application not found.',
  9: 'Application already exists.',
  10: 'Invalid application status.',
  11: 'Application is not pending.',
  12: 'Application is not approved.',
  13: 'Application is not rejected.',
  14: 'Withdrawal not found.',
  15: 'Withdrawal already processed.',
  16: 'Withdrawal not approved.',
  17: 'Emergency withdrawal not found.',
  18: 'Emergency withdrawal already processed.',
  19: 'Emergency withdrawal not approved.',
  20: 'Emergency withdrawal already requested.',
  21: 'Emergency withdrawal was not requested.',
  22: 'Invalid threshold configuration.',
  23: 'Invalid signer configuration.',
  24: 'Application status not found.',
  25: 'Invalid contract address.',
  26: 'Invalid amount.',
  27: 'Arithmetic overflow.',
  28: 'Token transfer failed.',
  29: 'Invalid token address.',
};

/**
 * Returns the user-facing message for a `ContractError` code, or `undefined`
 * when the code is unknown.
 */
export function getContractErrorMessage(code: number): string | undefined {
  return CONTRACT_ERROR_MESSAGES[code];
}

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

/**
 * Extracts a human-readable message from an error of unknown shape.
 *
 * **Precedence order** (first match wins):
 * 1. Axios-shaped error (`err.response.data.message` or `.error`)
 * 2. `ApiError` instance (`err.data.message` or `.error`)
 * 3. Standard `Error` instance (`err.message`)
 * 4. Generic object with a `message` property
 * 5. Fallback string: `"Something went wrong. Please try again."`
 *
 * This order ensures server-provided messages are preferred over generic stack
 * traces, and structured API errors are preferred over plain `Error` objects.
 */
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
