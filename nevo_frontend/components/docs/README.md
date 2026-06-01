# Nevo Frontend — Component Documentation

This directory contains usage guides for every reusable component in `nevo_frontend/components/`.

## Component Index

| Component                           | File                | Description                                 |
| ----------------------------------- | ------------------- | ------------------------------------------- |
| [Button](./Button.md)               | `Button.tsx`        | Multi-variant button with loading state     |
| [Avatar](./Avatar.md)               | `Avatar.tsx`        | Profile image with initials fallback        |
| [Navbar](./Navbar.md)               | `Navbar.tsx`        | Slim top nav bar (Server Component)         |
| [Header](./Header.md)               | `Header.tsx`        | Sticky header with mobile drawer            |
| [ConnectWallet](./ConnectWallet.md) | `ConnectWallet.tsx` | Freighter wallet connect/disconnect widget  |
| [WalletAddress](./WalletAddress.md) | `WalletAddress.tsx` | Responsive address display with copy button |
| [Footer](./Footer.md)               | `Footer.tsx`        | Site footer with nav and social links       |

## Importing Components

All components are re-exported from the barrel file:

```tsx
import { Button, Avatar, Header, Footer, WalletAddress } from '@/components';

// Default exports (Navbar, ConnectWallet) must be imported directly:
import Navbar from '@/components/Navbar';
import ConnectWallet from '@/components/ConnectWallet';
```

## Typical Page Shell

```tsx
// app/layout.tsx
import { Header, Footer } from '@/components';

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="flex flex-col min-h-screen">
        <Header />
        <main className="flex-1">{children}</main>
        <Footer />
      </body>
    </html>
  );
}
```

## Design Tokens

Components use Tailwind CSS v4 utility classes and the following CSS custom properties for theming:

| Token                     | Usage                           |
| ------------------------- | ------------------------------- |
| `--color-text`            | Primary text colour             |
| `--color-text-muted`      | Secondary / muted text          |
| `--color-surface`         | Page background                 |
| `--color-surface-raised`  | Elevated surface (cards, pills) |
| `--color-border`          | Border colour                   |
| `brand-600` / `brand-700` | Primary brand colour (blue)     |

## Tooling

- **Framework:** Next.js 16 (App Router)
- **Styling:** Tailwind CSS v4
- **State:** Zustand (`useWalletStore`)
- **Wallet:** `@stellar/freighter-api`
- **No Storybook** — documentation is maintained as Markdown in this directory.
