import React from 'react';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { NotificationCenter } from '@/components/NotificationCenter';
import {
  AppNotification,
  useNotificationsStore,
} from '@/src/store/notificationsStore';

const notifications: AppNotification[] = [
  {
    id: 'notification-1',
    type: 'system',
    title: 'First notice',
    message: 'First message',
    isRead: false,
    timestamp: '2026-01-01T00:00:00.000Z',
  },
  {
    id: 'notification-2',
    type: 'donation',
    title: 'Second notice',
    message: 'Second message',
    isRead: false,
    timestamp: '2026-01-02T00:00:00.000Z',
  },
];

function seedNotifications(items = notifications) {
  act(() => {
    useNotificationsStore.setState({ notifications: items });
  });
}

async function openDropdown(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Notifications' }));
}

describe('NotificationCenter', () => {
  beforeAll(() => {
    HTMLDialogElement.prototype.showModal = jest.fn(function (
      this: HTMLDialogElement
    ) {
      this.open = true;
    });
    HTMLDialogElement.prototype.close = jest.fn(function (
      this: HTMLDialogElement
    ) {
      this.open = false;
    });
  });

  beforeEach(() => {
    window.localStorage.clear();
    seedNotifications([]);
  });

  it('closes the dropdown when clicking outside', async () => {
    const user = userEvent.setup();
    render(
      <>
        <NotificationCenter />
        <button type="button">Outside</button>
      </>
    );

    await openDropdown(user);
    expect(screen.getByText('No notifications yet')).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Outside' }));

    expect(screen.queryByText('No notifications yet')).not.toBeInTheDocument();
  });

  it('closes on Escape and returns focus to the trigger', async () => {
    const user = userEvent.setup();
    render(<NotificationCenter />);

    const trigger = screen.getByRole('button', { name: 'Notifications' });
    await user.click(trigger);
    fireEvent.keyDown(document, { key: 'Escape' });

    expect(screen.queryByText('No notifications yet')).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  it('calls the correct store actions for reading and deleting notifications', async () => {
    const user = userEvent.setup();
    seedNotifications();
    render(<NotificationCenter />);
    await openDropdown(user);

    await user.click(screen.getByRole('button', { name: 'Mark all read' }));
    expect(
      useNotificationsStore.getState().notifications.every((item) => item.isRead)
    ).toBe(true);

    act(() => {
      useNotificationsStore.setState({ notifications });
    });
    await user.click(screen.getAllByRole('button', { name: 'Mark read' })[0]);
    expect(useNotificationsStore.getState().notifications).toEqual([
      { ...notifications[0], isRead: true },
      notifications[1],
    ]);

    await user.click(
      screen.getByRole('button', { name: 'Delete notification: First notice' })
    );
    expect(useNotificationsStore.getState().notifications).toEqual([
      notifications[1],
    ]);
  });

  it('only clears notifications after confirming Clear all', async () => {
    const user = userEvent.setup();
    seedNotifications();
    render(<NotificationCenter />);
    await openDropdown(user);

    await user.click(screen.getByRole('button', { name: 'Clear all' }));
    const dialog = screen.getByRole('dialog');
    expect(useNotificationsStore.getState().notifications).toHaveLength(2);

    await user.click(within(dialog).getByRole('button', { name: 'Cancel' }));
    expect(useNotificationsStore.getState().notifications).toHaveLength(2);

    await user.click(screen.getByRole('button', { name: 'Clear all' }));
    await user.click(
      within(screen.getByRole('dialog')).getByRole('button', {
        name: 'Clear all',
      })
    );
    expect(useNotificationsStore.getState().notifications).toHaveLength(0);
  });

  it('renders EmptyState when there are no notifications', async () => {
    const user = userEvent.setup();
    render(<NotificationCenter />);
    await openDropdown(user);

    expect(screen.getByRole('status')).toHaveTextContent('No notifications yet');
  });
});
