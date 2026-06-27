'use client';

import React, { useEffect, useState } from 'react';
import { useWalletStore } from '@/src/store/walletStore';
import { Avatar } from '@/components/Avatar';
import { Button } from '@/components/Button';
import { WalletAddress } from '@/components/WalletAddress';
import {
  fetchMyProfile,
  updateProfile,
  type ApiProfile,
} from '@/lib/api-client';
import { toast } from '@/components/Toast';

interface UserPreferences {
  email: string;
  displayName: string;
  notifications: {
    donations: boolean;
    withdrawals: boolean;
    poolUpdates: boolean;
  };
  avatarSrc?: string;
}

const DEFAULT_PREFERENCES: UserPreferences = {
  email: '',
  displayName: '',
  notifications: {
    donations: true,
    withdrawals: true,
    poolUpdates: false,
  },
};

export default function ProfilePage() {
  const { publicKey } = useWalletStore();
  const [preferences, setPreferences] =
    useState<UserPreferences>(DEFAULT_PREFERENCES);
  const [isEditingProfile, setIsEditingProfile] = useState(false);
  const [profile, setProfile] = useState<ApiProfile | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    let active = true;
    fetchMyProfile()
      .then((data) => {
        if (!active) return;
        setProfile(data);
        setPreferences((p) => ({
          ...p,
          displayName:
            data.displayName ??
            (data.publicKey
              ? `${data.publicKey.slice(0, 6)}…${data.publicKey.slice(-4)}`
              : ''),
        }));
      })
      .catch((err) => {
        console.error('Failed to load profile:', err);
      })
      .finally(() => {
        if (active) {
          setIsLoading(false);
        }
      });
    return () => {
      active = false;
    };
  }, []);

  // Handle avatar upload
  const handleAvatarChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        setPreferences({
          ...preferences,
          avatarSrc: e.target?.result as string,
        });
      };
      reader.readAsDataURL(file);
    }
  };

  const handleSaveProfile = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    try {
      const updated = await updateProfile(preferences.displayName);
      setProfile(updated);
      toast('Profile updated successfully');
      setIsEditingProfile(false);
    } catch (err) {
      const msg =
        err instanceof Error ? err.message : 'Failed to update profile';
      toast(msg, 'error');
    }
  };

  const toggleNotification = (key: keyof UserPreferences['notifications']) => {
    setPreferences({
      ...preferences,
      notifications: {
        ...preferences.notifications,
        [key]: !preferences.notifications[key],
      },
    });
  };

  return (
    <main className="mx-auto max-w-4xl px-6 py-10">
      {/* Header */}
      <div className="mb-8">
        <h1 className="text-2xl font-bold tracking-tight">
          Profile & Settings
        </h1>
        <p className="mt-1 text-sm text-[var(--color-text-muted)]">
          Manage your account information and preferences
        </p>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        {/* Profile Card */}
        <div className="lg:col-span-1">
          <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
            <div className="flex flex-col items-center text-center">
              {/* Avatar */}
              <div className="relative mb-4">
                {isLoading ? (
                  <div className="h-24 w-24 rounded-full bg-gray-200 dark:bg-gray-700 animate-pulse" />
                ) : (
                  <>
                    <Avatar
                      name={preferences.displayName}
                      src={preferences.avatarSrc}
                      size="lg"
                      className="h-24 w-24 text-2xl"
                    />
                    <label className="absolute bottom-0 right-0 flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-brand-600 text-white hover:bg-brand-700 transition-colors">
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        strokeWidth={2}
                        stroke="currentColor"
                        className="h-4 w-4"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L6.832 19.82a4.5 4.5 0 0 1-1.897 1.13l-2.685.8.8-2.685a4.5 4.5 0 0 1 1.13-1.897L16.863 4.487Z"
                        />
                      </svg>
                      <input
                        type="file"
                        accept="image/*"
                        hidden
                        onChange={handleAvatarChange}
                        aria-label="Upload profile picture"
                      />
                    </label>
                  </>
                )}
              </div>

              {/* Name */}
              <h2 className="text-lg font-semibold">
                {profile?.displayName ??
                  (publicKey
                    ? `${publicKey.slice(0, 6)}…${publicKey.slice(-4)}`
                    : '—')}
              </h2>
              <div className="mt-2 w-full">
                {isLoading ? (
                  <div className="h-5 w-40 bg-gray-200 dark:bg-gray-700 rounded animate-pulse mx-auto" />
                ) : (
                  <WalletAddress address={publicKey || ''} />
                )}
              </div>
              {profile?.createdAt && (
                <p className="mt-1 text-xs text-[var(--color-text-muted)]">
                  Member since{' '}
                  {new Date(profile.createdAt).toLocaleDateString('en-US', {
                    year: 'numeric',
                    month: 'short',
                  })}
                </p>
              )}

              <div className="mt-6 w-full">
                {isLoading ? (
                  <div className="h-10 w-full bg-gray-200 dark:bg-gray-700 rounded-xl animate-pulse" />
                ) : isEditingProfile ? (
                  <form onSubmit={handleSaveProfile} className="space-y-4">
                    <div>
                      <label
                        htmlFor="displayName"
                        className="block text-sm font-medium mb-1"
                      >
                        Display Name
                      </label>
                      <input
                        id="displayName"
                        type="text"
                        value={preferences.displayName}
                        onChange={(e) =>
                          setPreferences({
                            ...preferences,
                            displayName: e.target.value,
                          })
                        }
                        className="w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm"
                      />
                    </div>
                    <div>
                      <label
                        htmlFor="email"
                        className="block text-sm font-medium mb-1"
                      >
                        Email
                      </label>
                      <input
                        id="email"
                        type="email"
                        value={preferences.email}
                        onChange={(e) =>
                          setPreferences({
                            ...preferences,
                            email: e.target.value,
                          })
                        }
                        className="w-full rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] px-3 py-2 text-sm"
                      />
                    </div>
                    <div className="flex gap-2">
                      <Button type="submit" size="small">
                        Save
                      </Button>
                      <Button
                        type="button"
                        variant="secondary"
                        size="small"
                        onClick={() => setIsEditingProfile(false)}
                      >
                        Cancel
                      </Button>
                    </div>
                  </form>
                ) : (
                  <Button
                    type="button"
                    variant="outlined"
                    className="w-full"
                    onClick={() => setIsEditingProfile(true)}
                  >
                    Edit Profile
                  </Button>
                )}
              </div>
            </div>
          </div>
        </div>

        {/* Settings & Activity */}
        <div className="lg:col-span-2 space-y-6">
          {/* Notifications */}
          <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
            <h3 className="text-lg font-semibold mb-4">
              Notification Preferences
            </h3>
            <div className="space-y-3">
              {[
                {
                  key: 'donations',
                  label: 'Donation confirmations',
                  desc: 'Get notified when your donation is confirmed',
                },
                {
                  key: 'withdrawals',
                  label: 'Withdrawal alerts',
                  desc: 'Get notified when a withdrawal is processed',
                },
                {
                  key: 'poolUpdates',
                  label: 'Pool updates',
                  desc: 'Get notified about updates to pools you follow',
                },
              ].map((item) => (
                <label
                  key={item.key}
                  className="flex items-start gap-3 cursor-pointer"
                >
                  <div className="mt-1">
                    <input
                      type="checkbox"
                      checked={
                        preferences.notifications[
                          item.key as keyof typeof preferences.notifications
                        ]
                      }
                      onChange={() =>
                        toggleNotification(
                          item.key as keyof typeof preferences.notifications
                        )
                      }
                      className="h-4 w-4 text-brand-600 rounded border-[var(--color-border)] focus:ring-brand-500"
                    />
                  </div>
                  <div>
                    <span className="font-medium text-sm">{item.label}</span>
                    <p className="text-xs text-[var(--color-text-muted)]">
                      {item.desc}
                    </p>
                  </div>
                </label>
              ))}
            </div>
          </div>

          {/* Account Settings */}
          <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
            <h3 className="text-lg font-semibold mb-4">Account Settings</h3>
            <div className="space-y-3">
              <button className="w-full text-left px-4 py-3 rounded-xl hover:bg-[var(--color-surface-raised)] transition-colors flex items-center justify-between">
                <span>Change Password</span>
                <span className="text-[var(--color-text-muted)]">→</span>
              </button>
              <button className="w-full text-left px-4 py-3 rounded-xl hover:bg-[var(--color-surface-raised)] transition-colors flex items-center justify-between">
                <span>Connected Wallets</span>
                <span className="text-[var(--color-text-muted)]">→</span>
              </button>
              <button className="w-full text-left px-4 py-3 rounded-xl hover:bg-[var(--color-surface-raised)] transition-colors flex items-center justify-between text-red-500">
                <span>Delete Account</span>
                <span className="text-[var(--color-text-muted)]">→</span>
              </button>
            </div>
          </div>

          {/* Activity History */}
          <div className="rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] p-6">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-semibold">Recent Activity</h3>
              <a
                href="/transactions"
                className="text-sm text-brand-600 hover:text-brand-700 transition-colors"
              >
                View all →
              </a>
            </div>
            <div className="space-y-3">
              {(
                [] as Array<{
                  id: string;
                  type: string;
                  amount: string;
                  asset: string;
                  recipient: string;
                  date: string;
                }>
              ).length === 0 ? (
                <p className="text-sm text-[var(--color-text-muted)]">
                  No recent activity yet.
                </p>
              ) : (
                (
                  [] as Array<{
                    id: string;
                    type: string;
                    amount: string;
                    asset: string;
                    recipient: string;
                    date: string;
                  }>
                ).map((tx) => (
                  <div
                    key={tx.id}
                    className="flex items-center gap-4 rounded-xl p-3 transition-colors hover:bg-[var(--color-surface-raised)]"
                  >
                    <div className="flex size-9 items-center justify-center rounded-full bg-brand-100 text-brand-600">
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        strokeWidth={1.5}
                        stroke="currentColor"
                        className="size-4"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12Z"
                        />
                      </svg>
                    </div>
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">Donation</span>
                        <span className="text-sm font-semibold tabular-nums">
                          {tx.amount} {tx.asset}
                        </span>
                      </div>
                      <p className="truncate text-xs text-[var(--color-text-muted)]">
                        {tx.recipient}
                      </p>
                      <time
                        className="text-xs text-[var(--color-text-muted)]"
                        dateTime={tx.date}
                      >
                        {new Date(tx.date).toLocaleDateString('en-US', {
                          month: 'short',
                          day: 'numeric',
                        })}
                      </time>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>
    </main>
  );
}
