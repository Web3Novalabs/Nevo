import React from 'react';
import { act, render, renderHook, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { NotificationCenter } from '@/components/NotificationCenter';
import { useXdrTransaction } from '@/hooks/useXdrTransaction';
import { useNotificationsStore } from '@/src/store/notificationsStore';
import { signTransaction } from '@/lib/stellar';

jest.mock('@/lib/stellar', () => ({
  signTransaction: jest.fn(),
}));

const mockSignTransaction = signTransaction as jest.Mock;

async function submitSuccessfulTransaction() {
  mockSignTransaction.mockResolvedValue('signed-xdr');
  const { result } = renderHook(() => useXdrTransaction());

  let hash = '';
  await act(async () => {
    hash = await result.current.submit(
      jest.fn().mockResolvedValue('unsigned-xdr'),
      jest.fn().mockResolvedValue({ hash: 'tx-hash-123' })
    );
  });

  expect(result.current.status).toBe('success');
  return hash;
}

describe('NotificationCenter + useXdrTransaction integration', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    window.localStorage.clear();
    useNotificationsStore.setState({ notifications: [] });
  });

  // Gap: useXdrTransaction has no notification wiring, so a successful
  // transaction never reaches NotificationCenter unless the caller adds it.
  it('does not add a notification on its own when status reaches success', async () => {
    const addNotification = jest.spyOn(
      useNotificationsStore.getState(),
      'addNotification'
    );

    await submitSuccessfulTransaction();
    render(<NotificationCenter />);
    await userEvent.click(
      screen.getByRole('button', { name: 'Notifications' })
    );

    expect(addNotification).not.toHaveBeenCalled();
    expect(useNotificationsStore.getState().notifications).toHaveLength(0);
    expect(screen.getByText('No notifications yet')).toBeInTheDocument();
  });

  it('surfaces a success notification added by the caller in NotificationCenter', async () => {
    render(<NotificationCenter />);

    const hash = await submitSuccessfulTransaction();
    act(() => {
      useNotificationsStore.getState().addNotification({
        type: 'system',
        title: 'Transaction confirmed',
        message: `Transaction ${hash} was submitted successfully.`,
      });
    });

    const trigger = screen.getByRole('button', { name: 'Notifications' });
    expect(trigger).toHaveTextContent('1');

    await userEvent.click(trigger);
    expect(screen.getByText('Transaction confirmed')).toBeInTheDocument();
    expect(
      screen.getByText('Transaction tx-hash-123 was submitted successfully.')
    ).toBeInTheDocument();
  });
});
