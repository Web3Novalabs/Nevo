import { HttpException, HttpStatus } from '@nestjs/common';

const STELLAR_CODE_MAP: Record<
  string,
  { status: HttpStatus; message: string }
> = {
  tx_bad_auth: {
    status: HttpStatus.UNAUTHORIZED,
    message: 'Bad authentication',
  },
  op_underfunded: {
    status: HttpStatus.BAD_REQUEST,
    message: 'Insufficient balance',
  },
  op_no_source_account: {
    status: HttpStatus.NOT_FOUND,
    message: 'Source account does not exist on the network',
  },
  timeout: {
    status: HttpStatus.REQUEST_TIMEOUT,
    message: 'Transaction timed out — the network did not confirm it in time',
  },
  invalid_xdr: {
    status: HttpStatus.BAD_REQUEST,
    message: 'Invalid transaction XDR',
  },
  tx_submit_failed: {
    status: HttpStatus.BAD_REQUEST,
    message: 'Transaction rejected by the network',
  },
  tx_retry_later: {
    status: HttpStatus.TOO_MANY_REQUESTS,
    message: 'Transaction could not be processed yet — retry later',
  },
  account_not_found: {
    status: HttpStatus.NOT_FOUND,
    message: 'Source account does not exist on the network',
  },
  simulation_failed: {
    status: HttpStatus.BAD_REQUEST,
    message: 'Soroban transaction simulation failed',
  },
  prepare_failed: {
    status: HttpStatus.BAD_GATEWAY,
    message: 'Failed to prepare Soroban transaction',
  },
};

const CONTRACT_CODE_MAP: Record<
  string,
  { status: HttpStatus; message: string }
> = {
  '21': {
    status: HttpStatus.BAD_REQUEST,
    message: 'Emergency withdrawal was not requested',
  },
  '24': {
    status: HttpStatus.NOT_FOUND,
    message: 'Application status not found',
  },
  '28': {
    status: HttpStatus.BAD_GATEWAY,
    message: 'Token transfer failed',
  },
  '29': {
    status: HttpStatus.BAD_REQUEST,
    message: 'Invalid token address',
  },
};

export class StellarError extends HttpException {
  constructor(codeOrMessage: string) {
    const mapped =
      STELLAR_CODE_MAP[codeOrMessage] ?? CONTRACT_CODE_MAP[codeOrMessage];
    super(
      mapped?.message ?? codeOrMessage,
      mapped?.status ?? HttpStatus.INTERNAL_SERVER_ERROR,
    );
  }
}