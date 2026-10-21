# Nevo Frontend

The frontend for Nevo — an open-source on-chain donation platform built on the Stellar blockchain. This is a Next.js 15 application using the App Router, Tailwind CSS, Zustand for state management, and TypeScript.

## Monorepo Context

Nevo is organized as a monorepo with three layers:

- `nevo_frontend/` — This directory (Next.js frontend)
- `nevo_server/` — NestJS backend API
- `nevo_contract/` — Soroban smart contract (Rust)

This layer handles the user interface, wallet connections via Freighter, and interactions with the backend API and Stellar network.

## Prerequisites

- Node.js 18+ 
- npm
- [Freighter wallet extension](https://www.freighter.app/) (for local development with Stellar testnet)

## Environment Variables

Create a `.env.local` file in the `nevo_frontend/` directory with the following variables:

```bash
NEXT_PUBLIC_API_BASE_URL=http://localhost:3001
NEXT_PUBLIC_STELLAR_NETWORK=testnet
```

- `NEXT_PUBLIC_API_BASE_URL` — URL of the Nevo backend API (default: `http://localhost:3001` for local development)
- `NEXT_PUBLIC_STELLAR_NETWORK` — Stellar network to connect to (`testnet` for development, `pubnet` for production)

## Setting Up Freighter Wallet for Local Development

1. Install the [Freighter browser extension](https://www.freighter.app/)
2. Create a new wallet or import an existing one
3. Switch to **Testnet** mode in Freighter settings
4. Fund your testnet wallet with XLM using the [Stellar Testnet Faucet](https://friendbot.stellar.org/)
5. Ensure Freighter is unlocked before running the app

## Available Scripts

### `npm run dev`

Starts the development server on [http://localhost:3000](http://localhost:3000). The app hot-reloads as you make changes.

### `npm run build`

Creates an optimized production build. This must pass before pushing changes (enforced by pre-push git hook).

### `npm test`

Runs unit tests using Jest. Tests are located in `__tests__/` and use React Testing Library.

### `npm run test:e2e`

Runs end-to-end tests using Playwright. Tests are located in `__tests__/e2e/`.

### `npm run test:e2e:ui`

Runs Playwright E2E tests with the interactive UI for debugging.

## Project Structure

- `app/` — Next.js App Router pages
- `components/` — Reusable UI components
- `hooks/` — Custom React hooks
- `lib/` — Utilities (api-client, stellar, validation, etc.)
- `src/store/` — Zustand stores (pools, donations, wallet, ui, theme)
- `__tests__/` — Unit and E2E tests

## Browser Compatibility

Supported browsers are defined in `package.json` under `browserslist`:
- Last 2 versions of Chrome, Firefox, Safari, Edge
- Last 2 versions of iOS and Android
- Excludes dead browsers and Opera Mini

## Learn More

- [Next.js Documentation](https://nextjs.org/docs)
- [Stellar Documentation](https://developers.stellar.org/)
- [Freighter Documentation](https://docs.freighter.app/)
