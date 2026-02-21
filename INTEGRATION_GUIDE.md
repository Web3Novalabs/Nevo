# Frontend-Contract Integration Guide

This guide will help you integrate the Nevo frontend with the Soroban smart contract.

## Overview

The integration involves:
1. Setting up contract client
2. Creating contract interaction hooks
3. Implementing transaction flows
4. Handling responses and errors

## Step 1: Install Dependencies

The frontend already has `@creit.tech/stellar-wallets-kit` installed. You may need additional packages:

```bash
cd frontend
npm install @stellar/stellar-sdk soroban-client
```

## Step 2: Create Contract Client

Create `frontend/src/lib/contract/client.ts`:

```typescript
import { Contract, SorobanRpc, TransactionBuilder, Networks } from '@stellar/stellar-sdk';

const RPC_URL = process.env.NEXT_PUBLIC_STELLAR_RPC_URL || 'https://soroban-testnet.stellar.org';
const CONTRACT_ID = process.env.NEXT_PUBLIC_CROWDFUNDING_CONTRACT_ID || '';
const NETWORK_PASSPHRASE = process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE || 'Test SDF Network ; September 2015';

export class CrowdfundingClient {
  private contract: Contract;
  private server: SorobanRpc.Server;

  constructor() {
    this.contract = new Contract(CONTRACT_ID);
    this.server = new SorobanRpc.Server(RPC_URL);
  }

  async createCampaign(params: {
    id: string;
    title: string;
    creator: string;
    goal: bigint;
    deadline: number;
    token: string;
  }) {
    // Implementation here
  }

  async createPool(params: {
    name: string;
    description: string;
    creator: string;
    target: bigint;
    deadline: number;
  }) {
    // Implementation here
  }

  async contribute(params: {
    poolId: number;
    contributor: string;
    token: string;
    amount: bigint;
    anonymous: boolean;
  }) {
    // Implementation here
  }

  async getCampaign(id: string) {
    // Implementation here
  }

  async getPool(poolId: number) {
    // Implementation here
  }

  async getAllCampaigns() {
    // Implementation here
  }
}

export const contractClient = new CrowdfundingClient();
```

## Step 3: Create React Hooks

Create `frontend/src/lib/hooks/useContract.ts`:

```typescript
import { useState, useCallback } from 'react';
import { contractClient } from '../contract/client';
import { toast } from 'sonner';

export function useCreateCampaign() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const createCampaign = useCallback(async (params: {
    title: string;
    goal: number;
    deadline: Date;
  }) => {
    setLoading(true);
    setError(null);

    try {
      // Get wallet address
      const walletAddress = ''; // Get from wallet context

      const result = await contractClient.createCampaign({
        id: generateCampaignId(),
        title: params.title,
        creator: walletAddress,
        goal: BigInt(params.goal),
        deadline: Math.floor(params.deadline.getTime() / 1000),
        token: process.env.NEXT_PUBLIC_TOKEN_CONTRACT_ID || '',
      });

      toast.success('Campaign created successfully!');
      return result;
    } catch (err) {
      const error = err as Error;
      setError(error);
      toast.error(`Failed to create campaign: ${error.message}`);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  return { createCampaign, loading, error };
}

export function useCreatePool() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const createPool = useCallback(async (params: {
    name: string;
    description: string;
    target: number;
    deadline: Date;
  }) => {
    setLoading(true);
    setError(null);

    try {
      const walletAddress = ''; // Get from wallet context

      const result = await contractClient.createPool({
        name: params.name,
        description: params.description,
        creator: walletAddress,
        target: BigInt(params.target),
        deadline: Math.floor(params.deadline.getTime() / 1000),
      });

      toast.success('Pool created successfully!');
      return result;
    } catch (err) {
      const error = err as Error;
      setError(error);
      toast.error(`Failed to create pool: ${error.message}`);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  return { createPool, loading, error };
}

export function useContribute() {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const contribute = useCallback(async (params: {
    poolId: number;
    amount: number;
    anonymous?: boolean;
  }) => {
    setLoading(true);
    setError(null);

    try {
      const walletAddress = ''; // Get from wallet context

      const result = await contractClient.contribute({
        poolId: params.poolId,
        contributor: walletAddress,
        token: process.env.NEXT_PUBLIC_TOKEN_CONTRACT_ID || '',
        amount: BigInt(params.amount),
        anonymous: params.anonymous || false,
      });

      toast.success('Contribution successful!');
      return result;
    } catch (err) {
      const error = err as Error;
      setError(error);
      toast.error(`Failed to contribute: ${error.message}`);
      throw error;
    } finally {
      setLoading(false);
    }
  }, []);

  return { contribute, loading, error };
}

export function usePools() {
  const [pools, setPools] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchPools = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      // Fetch all pools from contract
      const poolData = await contractClient.getAllCampaigns();
      setPools(poolData);
    } catch (err) {
      const error = err as Error;
      setError(error);
      toast.error(`Failed to fetch pools: ${error.message}`);
    } finally {
      setLoading(false);
    }
  }, []);

  return { pools, loading, error, fetchPools };
}

function generateCampaignId(): string {
  // Generate a unique campaign ID
  return crypto.randomUUID();
}
```

## Step 4: Create Wallet Context

Create `frontend/src/lib/context/WalletContext.tsx`:

```typescript
'use client';

import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { StellarWalletsKit, WalletNetwork, ISupportedWallet } from '@creit.tech/stellar-wallets-kit';

interface WalletContextType {
  address: string | null;
  connected: boolean;
  connect: () => Promise<void>;
  disconnect: () => void;
  kit: StellarWalletsKit | null;
}

const WalletContext = createContext<WalletContextType>({
  address: null,
  connected: false,
  connect: async () => {},
  disconnect: () => {},
  kit: null,
});

export function WalletProvider({ children }: { children: ReactNode }) {
  const [address, setAddress] = useState<string | null>(null);
  const [kit, setKit] = useState<StellarWalletsKit | null>(null);

  useEffect(() => {
    const walletKit = new StellarWalletsKit({
      network: process.env.NEXT_PUBLIC_STELLAR_NETWORK as WalletNetwork || WalletNetwork.TESTNET,
      selectedWalletId: 'freighter',
      modules: [],
    });
    setKit(walletKit);
  }, []);

  const connect = async () => {
    if (!kit) return;

    try {
      await kit.openModal({
        onWalletSelected: async (option: ISupportedWallet) => {
          kit.setWallet(option.id);
          const { address } = await kit.getAddress();
          setAddress(address);
        },
      });
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    }
  };

  const disconnect = () => {
    setAddress(null);
  };

  return (
    <WalletContext.Provider
      value={{
        address,
        connected: !!address,
        connect,
        disconnect,
        kit,
      }}
    >
      {children}
    </WalletContext.Provider>
  );
}

export const useWallet = () => useContext(WalletContext);
```

## Step 5: Update Layout to Include Wallet Provider

Update `frontend/src/app/layout.tsx`:

```typescript
import { WalletProvider } from '@/lib/context/WalletContext';

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>
        <WalletProvider>
          <main>{children}</main>
          <Toaster />
        </WalletProvider>
      </body>
    </html>
  );
}
```

## Step 6: Update ExplorePools Component

Replace mock data with real contract data:

```typescript
'use client';

import { useEffect } from 'react';
import { usePools } from '@/lib/hooks/useContract';

export const ExplorePools = () => {
  const { pools, loading, error, fetchPools } = usePools();

  useEffect(() => {
    fetchPools();
  }, [fetchPools]);

  if (loading) {
    return <div>Loading pools...</div>;
  }

  if (error) {
    return <div>Error loading pools: {error.message}</div>;
  }

  // Render pools...
};
```

## Step 7: Create Pool Creation Form

Create `frontend/src/components/CreatePoolForm.tsx`:

```typescript
'use client';

import { useState } from 'react';
import { useCreatePool } from '@/lib/hooks/useContract';
import { useWallet } from '@/lib/context/WalletContext';

export function CreatePoolForm() {
  const { connected, connect } = useWallet();
  const { createPool, loading } = useCreatePool();
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    target: '',
    deadline: '',
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!connected) {
      await connect();
      return;
    }

    await createPool({
      name: formData.name,
      description: formData.description,
      target: parseFloat(formData.target),
      deadline: new Date(formData.deadline),
    });
  };

  return (
    <form onSubmit={handleSubmit}>
      {/* Form fields */}
      <button type="submit" disabled={loading}>
        {loading ? 'Creating...' : 'Create Pool'}
      </button>
    </form>
  );
}
```

## Step 8: Environment Variables

Create `frontend/.env.local`:

```env
NEXT_PUBLIC_STELLAR_NETWORK=testnet
NEXT_PUBLIC_STELLAR_RPC_URL=https://soroban-testnet.stellar.org
NEXT_PUBLIC_NETWORK_PASSPHRASE=Test SDF Network ; September 2015
NEXT_PUBLIC_CROWDFUNDING_CONTRACT_ID=your_contract_id_here
NEXT_PUBLIC_TOKEN_CONTRACT_ID=your_token_id_here
```

## Step 9: Testing

1. Deploy contract to testnet
2. Update contract ID in `.env.local`
3. Run frontend: `npm run dev`
4. Connect wallet
5. Test creating a pool
6. Test contributing to a pool
7. Test viewing pools

## Common Issues

### Issue: Wallet not connecting
**Solution**: Ensure Freighter or another Stellar wallet is installed

### Issue: Transaction fails
**Solution**: 
- Check wallet has sufficient XLM
- Verify contract ID is correct
- Check network configuration

### Issue: Data not loading
**Solution**:
- Verify RPC URL is accessible
- Check contract is deployed
- Verify contract ID

## Next Steps

1. Implement all contract methods in client
2. Create hooks for all user actions
3. Add loading and error states
4. Implement transaction history
5. Add user dashboard
6. Test thoroughly on testnet

## Resources

- [Stellar SDK Documentation](https://stellar.github.io/js-stellar-sdk/)
- [Soroban Documentation](https://soroban.stellar.org/docs)
- [Stellar Wallets Kit](https://github.com/Creit-Tech/Stellar-Wallets-Kit)

---

**Note**: This is a starting point. You'll need to implement the actual contract interaction logic based on the Soroban SDK and your specific contract interface.
