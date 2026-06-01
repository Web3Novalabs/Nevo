export type TxType = 'donation' | 'pool_creation' | 'withdrawal';
export type TxStatus = 'completed' | 'pending' | 'failed';

export interface Transaction {
  id: string;
  type: TxType;
  amount: string;
  asset: string;
  recipient: string;
  date: string;
  status: TxStatus;
  txHash: string;
}

export const MOCK_TRANSACTIONS: Transaction[] = [
  {
    id: '1',
    type: 'donation',
    amount: '250',
    asset: 'XLM',
    recipient: 'Clean Water Initiative',
    date: '2025-05-20T14:32:00Z',
    status: 'completed',
    txHash: 'abc123def456',
  },
  {
    id: '2',
    type: 'pool_creation',
    amount: '0',
    asset: 'XLM',
    recipient: 'Open Source Dev Fund',
    date: '2025-05-18T09:15:00Z',
    status: 'completed',
    txHash: 'bcd234efg567',
  },
  {
    id: '3',
    type: 'withdrawal',
    amount: '3200',
    asset: 'XLM',
    recipient: 'Community Garden Project',
    date: '2025-05-15T17:45:00Z',
    status: 'completed',
    txHash: 'cde345fgh678',
  },
  {
    id: '4',
    type: 'donation',
    amount: '100',
    asset: 'USDC',
    recipient: 'Education for All',
    date: '2025-05-12T11:20:00Z',
    status: 'pending',
    txHash: 'def456ghi789',
  },
  {
    id: '5',
    type: 'donation',
    amount: '500',
    asset: 'XLM',
    recipient: 'Medical Relief Fund',
    date: '2025-05-10T08:05:00Z',
    status: 'failed',
    txHash: 'efg567hij890',
  },
  {
    id: '6',
    type: 'pool_creation',
    amount: '0',
    asset: 'XLM',
    recipient: 'Tech Scholarship Pool',
    date: '2025-05-08T13:30:00Z',
    status: 'completed',
    txHash: 'fgh678ijk901',
  },
  {
    id: '7',
    type: 'donation',
    amount: '75',
    asset: 'XLM',
    recipient: 'Clean Water Initiative',
    date: '2025-05-05T16:00:00Z',
    status: 'completed',
    txHash: 'ghi789jkl012',
  },
  {
    id: '8',
    type: 'withdrawal',
    amount: '1500',
    asset: 'XLM',
    recipient: 'Open Source Dev Fund',
    date: '2025-04-28T10:10:00Z',
    status: 'completed',
    txHash: 'hij890klm123',
  },
  {
    id: '9',
    type: 'donation',
    amount: '200',
    asset: 'USDC',
    recipient: 'Education for All',
    date: '2025-04-20T14:00:00Z',
    status: 'completed',
    txHash: 'ijk901lmn234',
  },
  {
    id: '10',
    type: 'donation',
    amount: '50',
    asset: 'XLM',
    recipient: 'Medical Relief Fund',
    date: '2025-04-15T09:45:00Z',
    status: 'failed',
    txHash: 'jkl012mno345',
  },
  {
    id: '11',
    type: 'pool_creation',
    amount: '0',
    asset: 'XLM',
    recipient: 'Climate Action Fund',
    date: '2025-04-10T11:00:00Z',
    status: 'completed',
    txHash: 'klm123nop456',
  },
  {
    id: '12',
    type: 'donation',
    amount: '300',
    asset: 'XLM',
    recipient: 'Tech Scholarship Pool',
    date: '2025-04-05T15:30:00Z',
    status: 'completed',
    txHash: 'lmn234opq567',
  },
];
