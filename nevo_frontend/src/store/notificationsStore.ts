import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface AppNotification {
  id: string;
  type: 'donation' | 'pool_update' | 'system';
  title: string;
  message: string;
  isRead: boolean;
  timestamp: string;
  link?: string;
}

interface NotificationsState {
  notifications: AppNotification[];
  addNotification: (
    notification: Omit<AppNotification, 'id' | 'isRead' | 'timestamp'>
  ) => void;
  markAsRead: (id: string) => void;
  markAllAsRead: () => void;
  deleteNotification: (id: string) => void;
  clearAll: () => void;
}

const MOCK_NOTIFICATIONS: AppNotification[] = [
  {
    id: 'n1',
    type: 'donation',
    title: 'New Donation Received',
    message: 'Someone donated 500 XLM to your pool "Clean Water Initiative".',
    isRead: false,
    timestamp: new Date().toISOString(),
    link: '/pools/1',
  },
  {
    id: 'n2',
    type: 'pool_update',
    title: 'Goal Reached!',
    message:
      'Congratulations! "Open Source Dev Fund" has reached its funding goal.',
    isRead: false,
    timestamp: new Date(Date.now() - 3600000).toISOString(),
    link: '/pools/2',
  },
];

export const useNotificationsStore = create<NotificationsState>()(
  persist(
    (set) => ({
      notifications: MOCK_NOTIFICATIONS,

      addNotification: (notification) =>
        set((state) => ({
          notifications: [
            {
              ...notification,
              id: `notif-${Date.now()}-${Math.random()}`,
              isRead: false,
              timestamp: new Date().toISOString(),
            },
            ...state.notifications,
          ],
        })),

      markAsRead: (id) =>
        set((state) => ({
          notifications: state.notifications.map((n) =>
            n.id === id ? { ...n, isRead: true } : n
          ),
        })),

      markAllAsRead: () =>
        set((state) => ({
          notifications: state.notifications.map((n) => ({
            ...n,
            isRead: true,
          })),
        })),

      deleteNotification: (id) =>
        set((state) => ({
          notifications: state.notifications.filter((n) => n.id !== id),
        })),

      clearAll: () => set({ notifications: [] }),
    }),
    {
      name: 'nevo-notifications',
    }
  )
);
