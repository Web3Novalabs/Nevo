# Nevo Custom UI Components Documentation

This document provides an overview of all custom UI components and page sections built for the Nevo frontend (`frontend/src/components/`).

## 1. UI Kit Components (`src/components/ui/`)
These are foundational, reusable building blocks typically based on Radix UI primitives and styled with Tailwind CSS.

* **`button.tsx`**: A highly versatile Button component built with `class-variance-authority` (cva). It supports multiple variants (`default`, `destructive`, `outline`, `secondary`, `ghost`, `link`), sizes, and a built-in `isLoading` state that renders an animated spinner.
* **`checkbox.tsx`**: A custom-styled checkbox using `@radix-ui/react-checkbox`. It supports an `indeterminate` state in addition to checked/unchecked.
* **`checkbox-group.tsx`**: A wrapper component (`CheckboxGroup` and `CheckboxGroupItem`) that groups multiple checkboxes, providing shared labels, descriptions, and error text management for reliable accessibility.
* **`input.tsx`**: A standard text input field. For `type="password"`, it automatically utilizes a `usePasswordToggle` hook to embed an eye icon that toggles password visibility.
* **`label.tsx`**: A standardized label component based on `@radix-ui/react-label` for forms and inputs.
* **`sonner.tsx`**: A global toast notification wrapper customized around the `sonner` library, incorporating specific Lucide icons for various states (success, info, warning, error, loading).
* **`tooltip.tsx`**: A tooltip implementation (`Tooltip`, `TooltipTrigger`, `TooltipContent`, etc.) powered by Radix UI, featuring smooth fade/zoom animations and proper accessibility positioning.

---

## 2. Core Functional Components (`src/components/`)
These are business logic or complex visual components central to the application's functionality.

* **`Button.tsx`**: A secondary, simpler custom button implementation using `React.forwardRef`. Provides `primary`, `secondary`, and `outline` variants without relying on Radix UI or CVA.
* **`ConnectWallet.tsx`**: A crucial integration component that uses `stellar-wallets-kit` to connect and disconnect user wallets. It displays the connected public key when active.
* **`DonationModal.tsx`**: A modal interface triggered when users wish to donate to a pool. It allows users to select an asset (XLM or USDC), input a contribution amount (or choose quick amounts), and previews network fees/total deductions.
* **`ExplorePools.tsx`**: A full-page discovery interface providing a grid of pools with advanced filtering capabilities (by status and category) and text-based search.
* **`Navigation.tsx`**: The main top navigation bar. It is fully responsive, including a mobile hamburger menu. It houses page section anchors (`#features`, `#how-it-works`) and embeds the `ConnectWallet` component.
* **`PoolCard.tsx`**: A visually rich card displaying the details of an individual donation pool. It showcases a cover image, category badge, goal vs. raised mapping, an animated progress bar, and contributor counts. It also renders the "Donate Now" button that opens `DonationModal`.
* **`PoolGrid.tsx`**: A responsive, CSS grid-based layout container that maps over a list of pool data items and renders them as `PoolCard`s.
* **`waitlist/signup-form.tsx`**: A specialized form component for a waitlist page. Features client-side validation for email, name, and terms acceptance, along with loading spinners and a success state layout upon submission.

---

## 3. Landing Page Sections (`src/components/`)
These components represent the distinct, full-width sections of the main landing page.

* **`HeroSection.tsx`**: The introductory section of the site featuring the main value proposition, primary CTA buttons ("Start Creating Pools", "Discover Pools"), and trust metrics (100% Transparent, 0.1% Avg Fee).
* **`FeaturesSection.tsx`**: Highlights the platform's core offerings (Multi-Asset Support, DeFi Yield, etc.) arranged in a grid of `FeatureCard`s with distinct icons.
* **`HowItWorksSection.tsx`**: An enumerated, 4-step flow section explaining how to create a pool, share it, collect funds, and earn impact.
* **`SecuritySection.tsx`**: A split-layout section detailing the platform's Stellar blockchain security, smart contract audits, and real-time monitoring, accompanied by visual "Trust Indicators".
* **`CTASection.tsx`**: A bottom-page Call to Action block with a vibrant gradient background inviting users to launch the application.
* **`Footer.tsx`**: The site footer housing links to Product features, Resources (Documentation/API), and Legal pages.
