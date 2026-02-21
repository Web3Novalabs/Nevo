# Donation Transaction & Receipt Implementation

## Overview
Complete implementation of donation transaction execution and shareable receipt generation for the crowdfunding platform.

## Components

### 1. Transaction Execution (`lib/stellar-transaction.ts`)
Handles Stellar blockchain transactions:
- Supports XLM and USDC payments
- Connects to Stellar Horizon mainnet
- Signs transactions via wallet
- Returns transaction hash on success

### 2. Donation Modal (`components/DonationModal.tsx`)
Enhanced with transaction execution:
- Asset selection (XLM/USDC)
- Amount input with quick select
- Fee breakdown display
- Loading state during transaction
- Automatic receipt display on success

### 3. Donation Receipt (`components/DonationReceipt.tsx`)
Shareable receipt with:
- Transaction confirmation UI
- Pool and donor details
- Transaction hash with Stellar Expert link
- Social share functionality
- Download as image (PNG)

## Usage

```tsx
import { DonationModal } from "@/components";

<DonationModal
  isOpen={isOpen}
  onClose={() => setIsOpen(false)}
  poolTitle="Save the Ocean"
  poolAddress="GCZYLNGU4CA5NAWBAVTHMZH4JKXRCN3XWWIL3KIJMZ6QDMRKFZYQUD7Z"
/>
```

## Flow

1. User opens donation modal
2. Selects asset (XLM/USDC) and amount
3. Clicks "Confirm Donation"
4. Wallet prompts for signature
5. Transaction submitted to Stellar network
6. Receipt displayed with transaction details
7. User can share or download receipt

## Dependencies

- `@stellar/stellar-sdk` - Stellar blockchain SDK
- `html2canvas` - Receipt image generation
- `@creit.tech/stellar-wallets-kit` - Wallet integration

## Features

- Real-time transaction execution
- Network fee estimation
- Transaction status tracking
- Shareable social cards
- Downloadable receipts
- Stellar Expert integration
