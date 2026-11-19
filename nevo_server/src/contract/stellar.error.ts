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
};

export class StellarError extends HttpException {
  constructor(codeOrMessage: string) {
    const mapped = STELLAR_CODE_MAP[codeOrMessage];
    super(
      mapped?.message ?? codeOrMessage,
      mapped?.status ?? HttpStatus.INTERNAL_SERVER_ERROR,
    );
  }
}
