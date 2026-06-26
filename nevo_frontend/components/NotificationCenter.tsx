'use client';

import React, { useState, useEffect, useRef } from 'react';
import Link from 'next/link';
import { useNotificationsStore } from '@/src/store/notificationsStore';

export function NotificationCenter() {
  const {
    notifications,
    markAsRead,
    markAllAsRead,
    deleteNotification,
    clearAll,
  } = useNotificationsStore();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const unreadCount = notifications.filter((n) => !n.isRead).length;

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        type="button"
        className="relative flex items-center justify-center size-10 rounded-full hover:bg-[var(--color-surface-raised)] transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600"
        onClick={() => setIsOpen(!isOpen)}
        aria-label="Notifications"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
          className="size-6"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            d="M14.857 17.082a23.848 23.848 0 0 0 5.454-1.31A8.967 8.967 0 0 1 18 9.75V9A6 6 0 0 0 6 9v.75a8.967 8.967 0 0 1-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 0 1-5.714 0m5.714 0a3 3 0 1 1-5.714 0"
          />
        </svg>
        {unreadCount > 0 && (
          <span className="absolute top-1 right-1 flex size-4 items-center justify-center rounded-full bg-error text-[10px] font-bold text-white">
            {unreadCount > 9 ? '9+' : unreadCount}
          </span>
        )}
      </button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-80 sm:w-96 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] shadow-lg z-50 overflow-hidden">
          <div className="flex items-center justify-between border-b border-[var(--color-border)] px-4 py-3 bg-[var(--color-surface-raised)]">
            <h3 className="font-semibold text-sm">Notifications</h3>
            <div className="flex items-center gap-3 text-xs">
              <button
                onClick={() => markAllAsRead()}
                className="text-brand-600 hover:text-brand-700 font-medium"
              >
                Mark all read
              </button>
              <button
                onClick={() => clearAll()}
                className="text-error hover:text-error-dark font-medium"
              >
                Clear all
              </button>
            </div>
          </div>

          <div className="max-h-80 overflow-y-auto">
            {notifications.length === 0 ? (
              <div className="px-4 py-8 text-center text-sm text-[var(--color-text-muted)]">
                You have no notifications.
              </div>
            ) : (
              <ul className="divide-y divide-[var(--color-border)]">
                {notifications.map((n) => (
                  <li
                    key={n.id}
                    className={`flex items-start gap-3 p-4 ${n.isRead ? 'opacity-70' : 'bg-[var(--color-surface-raised)]'}`}
                  >
                    <div className="flex-1 min-w-0">
                      <div className="flex items-start justify-between gap-2">
                        <p
                          className={`text-sm font-semibold truncate ${n.isRead ? 'text-[var(--color-text)]' : 'text-brand-600'}`}
                        >
                          {n.title}
                        </p>
                        <button
                          onClick={() => deleteNotification(n.id)}
                          className="text-[var(--color-text-muted)] hover:text-error transition-colors"
                          aria-label="Delete notification"
                        >
                          <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            strokeWidth={2}
                            stroke="currentColor"
                            className="size-4"
                          >
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              d="M6 18 18 6M6 6l12 12"
                            />
                          </svg>
                        </button>
                      </div>
                      <p className="mt-1 text-xs text-[var(--color-text-muted)] line-clamp-2">
                        {n.message}
                      </p>
                      <div className="mt-2 flex items-center justify-between">
                        <span className="text-[10px] text-[var(--color-text-muted)]">
                          {new Date(n.timestamp).toLocaleString()}
                        </span>
                        <div className="flex items-center gap-2">
                          {n.link && (
                            <Link
                              href={n.link}
                              className="text-xs font-medium text-brand-600 hover:underline"
                              onClick={() => setIsOpen(false)}
                            >
                              View
                            </Link>
                          )}
                          {!n.isRead && (
                            <button
                              onClick={() => markAsRead(n.id)}
                              className="text-xs font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                            >
                              Mark read
                            </button>
                          )}
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
