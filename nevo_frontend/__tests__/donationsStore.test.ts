import { useDonationsStore, Donation } from '@/src/store/donationsStore';

const mockDonation: Donation = {
  id: '1',
  poolId: 'p1',
  poolName: 'Test Pool',
  amount: '10',
  asset: 'XLM',
  txHash: 'abc123',
  timestamp: '2024-01-01T00:00:00Z',
  status: 'pending',
};

beforeEach(() =>
  useDonationsStore.setState({ history: [], activeDonation: null })
);

describe('donationsStore', () => {
  it('adds a donation to history', () => {
    useDonationsStore.getState().addDonation(mockDonation);
    expect(useDonationsStore.getState().history).toHaveLength(1);
  });

  it('prepends new donations', () => {
    useDonationsStore.getState().addDonation(mockDonation);
    useDonationsStore.getState().addDonation({ ...mockDonation, id: '2' });
    expect(useDonationsStore.getState().history[0].id).toBe('2');
  });

  it('updates donation status', () => {
    useDonationsStore.getState().addDonation(mockDonation);
    useDonationsStore.getState().updateDonationStatus('1', 'confirmed');
    expect(useDonationsStore.getState().history[0].status).toBe('confirmed');
  });

  it('clears history', () => {
    useDonationsStore.getState().addDonation(mockDonation);
    useDonationsStore.getState().clearHistory();
    expect(useDonationsStore.getState().history).toHaveLength(0);
  });

  it('sets active donation', () => {
    useDonationsStore.getState().setActiveDonation(mockDonation);
    expect(useDonationsStore.getState().activeDonation).toEqual(mockDonation);
  });
});
