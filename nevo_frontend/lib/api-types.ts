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
  imageUrl: string;
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

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  limit: number;
}
