import { StrKey } from '@stellar/stellar-sdk';
import { IsNotEmpty, IsString, Matches, ValidateBy } from 'class-validator';
import { STELLAR_PUBLIC_KEY } from '../../common/stellar-public-key.js';

/**
 * Query of `GET /auth/challenge`.
 *
 * Validated before a nonce is stored, so a malformed or oversized key is
 * rejected with 400 and never reaches the `nonces` table.
 */
export class ChallengeQueryDto {
  /** Stellar public key (G...) of the wallet signing in. */
  @IsString()
  @IsNotEmpty()
  @Matches(STELLAR_PUBLIC_KEY, {
    message: 'publicKey must be a valid Stellar public key (G...)',
  })
  @ValidateBy({
    name: 'isValidEd25519PublicKey',
    validator: {
      validate: (value: unknown) =>
        typeof value === 'string' && StrKey.isValidEd25519PublicKey(value),
      defaultMessage: () =>
        'publicKey must be a valid Stellar public key (G...)',
    },
  })
  publicKey: string;
}
