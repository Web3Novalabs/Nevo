import { IsHexadecimal, IsNotEmpty, IsString, Matches } from 'class-validator';
import { STELLAR_PUBLIC_KEY } from '../../common/stellar-public-key.js';

/**
 * Body of `POST /auth/verify`: the wallet's answer to the challenge issued by
 * `GET /auth/challenge`.
 *
 * This is a class rather than an interface on purpose — the global
 * ValidationPipe (`whitelist`/`forbidNonWhitelisted`/`transform`) reads the
 * runtime metatype, so an interface here would be erased at runtime and the
 * body would go entirely unvalidated.
 */
export class VerifyAuthDto {
  /** Stellar public key (G...) of the wallet signing in. */
  @IsString()
  @IsNotEmpty()
  @Matches(STELLAR_PUBLIC_KEY, {
    message: 'publicKey must be a valid Stellar public key (G...)',
  })
  publicKey: string;

  /** Hex-encoded Ed25519 signature over `message`. */
  @IsString()
  @IsNotEmpty()
  @IsHexadecimal()
  signature: string;

  /** The nonce previously returned by `GET /auth/challenge`. */
  @IsString()
  @IsNotEmpty()
  message: string;
}
