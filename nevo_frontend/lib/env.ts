const REQUIRED_PUBLIC_ENV_VARS = [
  'NEXT_PUBLIC_API_BASE_URL',
  'NEXT_PUBLIC_STELLAR_NETWORK',
] as const;

type RequiredPublicEnvVar = (typeof REQUIRED_PUBLIC_ENV_VARS)[number];

function getRequiredPublicEnvVar(name: RequiredPublicEnvVar): string {
  const value = process.env[name];
  if (!value) {
    throw new Error(`Missing required env var: ${name}`);
  }
  return value;
}

export function validatePublicEnv(): void {
  REQUIRED_PUBLIC_ENV_VARS.forEach((name) => {
    getRequiredPublicEnvVar(name);
  });
}

export const env = {
  NEXT_PUBLIC_API_BASE_URL: getRequiredPublicEnvVar('NEXT_PUBLIC_API_BASE_URL'),
  NEXT_PUBLIC_STELLAR_NETWORK: getRequiredPublicEnvVar(
    'NEXT_PUBLIC_STELLAR_NETWORK'
  ),
} as const;
