import { HttpException, HttpStatus } from '@nestjs/common';

const CODE_MAP: Record<string, { status: HttpStatus; message: string }> = {
  tx_bad_auth: { status: HttpStatus.UNAUTHORIZED, message: 'tx_bad_auth' },
  op_underfunded: {
    status: HttpStatus.BAD_REQUEST,
    message: 'Insufficient balance',
  },
};

export class StellarError extends HttpException {
  constructor(code: string) {
    const mapped = CODE_MAP[code] ?? {
      status: HttpStatus.INTERNAL_SERVER_ERROR,
      message: code,
    };
    super(mapped.message, mapped.status);
  }
}
