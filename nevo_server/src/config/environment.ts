import { plainToInstance } from 'class-transformer';
import {
  IsIn,
  IsNotEmpty,
  IsPort,
  ValidateIf,
  validateSync,
} from 'class-validator';
import { isIP } from 'node:net';

function usesDatabaseFields(config: Environment): boolean {
  return !config.DATABASE_URL?.trim();
}

class Environment {
  @IsNotEmpty()
  CONTRACT_ID?: string;

  @IsNotEmpty()
  JWT_SECRET?: string;

  @IsIn(['testnet', 'mainnet'])
  STELLAR_NETWORK?: string;

  DATABASE_URL?: string;

  @ValidateIf(usesDatabaseFields)
  @IsNotEmpty()
  DB_HOST?: string;

  @ValidateIf(usesDatabaseFields)
  @IsPort()
  DB_PORT?: string;

  @ValidateIf(usesDatabaseFields)
  @IsNotEmpty()
  DB_USER?: string;

  @ValidateIf(usesDatabaseFields)
  @IsNotEmpty()
  DB_PASSWORD?: string;

  @ValidateIf(usesDatabaseFields)
  @IsNotEmpty()
  DB_NAME?: string;
}

export function validateEnvironment(config: Record<string, unknown>) {
  const errors: string[] = [];
  if (config.NODE_ENV !== 'test') {
    const normalized = Object.fromEntries(
      Object.entries(config).map(([key, value]) => [
        key,
        typeof value === 'string' && !value.trim() ? undefined : value,
      ]),
    );
    errors.push(
      ...validateSync(plainToInstance(Environment, normalized)).flatMap(
        ({ constraints }) => Object.values(constraints ?? {}),
      ),
    );
  }

  try {
    parseTrustProxy(config.TRUST_PROXY as string | undefined);
  } catch (error) {
    if (error instanceof Error) errors.push(error.message);
  }

  if (errors.length) {
    throw new Error(`Invalid server configuration: ${errors.join(', ')}`);
  }
  return config;
}

export function parseTrustProxy(
  value: string | undefined,
): false | number | string[] {
  const input = value?.trim();
  if (!input || input === 'false' || input === '0') return false;

  if (/^[1-9]\d*$/.test(input) && Number.isSafeInteger(Number(input))) {
    return Number(input);
  }

  const entries = input.split(',').map((entry) => entry.trim());
  const valid = entries.every((entry) => {
    if (['loopback', 'linklocal', 'uniquelocal'].includes(entry)) return true;
    const [address, prefix, extra] = entry.split('/');
    const family = isIP(address);
    if (!family || extra !== undefined) return false;
    return (
      prefix === undefined ||
      (/^\d{1,3}$/.test(prefix) && Number(prefix) <= (family === 4 ? 32 : 128))
    );
  });
  if (!valid) {
    throw new Error(
      'TRUST_PROXY must be false, a proxy hop count, or a comma-separated list of proxy IP addresses or CIDRs',
    );
  }
  return entries;
}
