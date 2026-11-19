import { HttpException, HttpStatus } from '@nestjs/common';
import { StellarError } from './stellar.error.js';

describe('StellarError', () => {
  describe('known Stellar codes', () => {
    it.each([
      ['tx_bad_auth', HttpStatus.UNAUTHORIZED, 'Bad authentication'],
      ['op_underfunded', HttpStatus.BAD_REQUEST, 'Insufficient balance'],
      [
        'op_no_source_account',
        HttpStatus.NOT_FOUND,
        'Source account does not exist on the network',
      ],
      [
        'timeout',
        HttpStatus.REQUEST_TIMEOUT,
        'Transaction timed out — the network did not confirm it in time',
      ],
    ])('maps %s to its documented status and message', (code, status, message) => {
      const err = new StellarError(code);

      expect(err).toBeInstanceOf(HttpException);
      expect(err.getStatus()).toBe(status);
      expect(err.getResponse()).toBe(message);
    });
  });

  describe('unmapped codes', () => {
    it('falls back to INTERNAL_SERVER_ERROR with the raw code as the message', () => {
      const err = new StellarError('tx_unknown_code');

      expect(err.getStatus()).toBe(HttpStatus.INTERNAL_SERVER_ERROR);
      expect(err.getResponse()).toBe('tx_unknown_code');
    });

    it('does not partially match a known code embedded in a longer string', () => {
      const err = new StellarError('op_underfunded: extra detail');

      expect(err.getStatus()).toBe(HttpStatus.INTERNAL_SERVER_ERROR);
      expect(err.getResponse()).toBe('op_underfunded: extra detail');
    });

    it('does not match a known code with different casing', () => {
      const err = new StellarError('TX_BAD_AUTH');

      expect(err.getStatus()).toBe(HttpStatus.INTERNAL_SERVER_ERROR);
      expect(err.getResponse()).toBe('TX_BAD_AUTH');
    });
  });
});
