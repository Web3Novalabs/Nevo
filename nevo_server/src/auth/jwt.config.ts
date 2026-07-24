import { Logger } from '@nestjs/common';

const logger = new Logger('JwtConfig');

export const JWT_SECRET_FALLBACK = 'dev-secret';

export function getJwtSecret(): string {
  const secret = process.env.JWT_SECRET;
  if (!secret) {
    logger.warn(
      'JWT_SECRET is not set in the environment. Falling back to insecure default. ' +
        'This is unsafe in production.',
    );
    return JWT_SECRET_FALLBACK;
  }
  return secret;
}
