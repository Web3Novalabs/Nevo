// API response interfaces — these match the shapes returned by the NestJS server.
// Keep these separate from Zustand store types (src/store/*).

export interface ApiPool {
  id: string;
  contractPoolId: string;
  title: string;
  description: string;
  category: string;
  creatorWallet: string;
  status: string;
  goal: string;
  raised: string;
  imageUrl: string | null;
  raisedOnChain: string;
  donorCount: number;
  createdAt: string;
}

export interface ApiDonation {
  id: string;
  txHash: string;
  poolId: string;
  poolTitle: string;
  donorWallet: string;
  amount: string;
  asset: string;
  createdAt: string;
}

export interface ApiUser {
  id: string;
  publicKey: string;
  displayName: string | null;
  createdAt: string;
}

export interface ApiTransaction {
  id: string;
  txHash: string;
  poolId: string;
  poolTitle: string;
  type: string;
  status: string;
  amount: string;
  asset: string;
  createdAt: string;
}

export interface TransactionListParams {
  page?: number;
  limit?: number;
  type?: string;
  status?: string;
  poolId?: string;
  search?: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
}

export type PaginatedTransactionsResponse = PaginatedResponse<ApiTransaction>;