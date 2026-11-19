import { defineConfig, globalIgnores } from 'eslint/config';
import nextVitals from 'eslint-config-next/core-web-vitals';
import nextTs from 'eslint-config-next/typescript';
import prettierConfig from 'eslint-config-prettier';

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  prettierConfig,
  {
    rules: {
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: '@stellar/freighter-api',
              message:
                'Import from the wallet abstraction (useXdrTransaction / lib/stellar) instead of @stellar/freighter-api directly.',
            },
          ],
        },
      ],
    },
  },
  {
    // The wallet module is the only place allowed to talk to Freighter directly.
    files: ['app/stellar-wallets-kit/**', 'lib/stellar.ts'],
    rules: {
      'no-restricted-imports': 'off',
    },
  },
  // Override default ignores of eslint-config-next.
  globalIgnores([
    // Default ignores of eslint-config-next:
    '.next/**',
    'out/**',
    'build/**',
    'next-env.d.ts',
  ]),
]);

export default eslintConfig;