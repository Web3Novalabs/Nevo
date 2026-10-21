import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import Navbar from '@/components/Navbar';

jest.mock('@/components/ConnectWallet', () => ({
  __esModule: true,
  default: () => <div>Connect Wallet</div>,
}));

jest.mock('@/components/NotificationCenter', () => ({
  NotificationCenter: () => <div>Notifications</div>,
}));

jest.mock('@/components/ThemeToggle', () => ({
  __esModule: true,
  default: () => <div>Theme Toggle</div>,
}));

jest.mock('@/components/MobileMenu', () => ({
  NAV_LINKS: [
    { label: 'Home', href: '/' },
    { label: 'Pools', href: '/pools' },
    { label: 'Donations', href: '/donations' },
    { label: 'Transactions', href: '/transactions' },
    { label: 'Stories', href: '/stories' },
    { label: 'Dashboard', href: '/dashboard' },
    { label: 'Profile', href: '/profile' },
    { label: 'Create Pool', href: '/pools/new' },
  ],
  MobileMenu: ({ open }: { open: boolean }) =>
    open ? <div role="dialog">Mobile Menu</div> : null,
  MobileMenuButton: ({
    open,
    onOpen,
  }: {
    open: boolean;
    onOpen: () => void;
  }) => (
    <button
      type="button"
      aria-label={open ? 'Menu open' : 'Open menu'}
      onClick={onOpen}
    >
      Menu
    </button>
  ),
}));

describe('Navbar', () => {
  it('renders every navigation link with the correct href', () => {
    render(<Navbar />);

    const links = [
      ['Home', '/'],
      ['Pools', '/pools'],
      ['Donations', '/donations'],
      ['Transactions', '/transactions'],
      ['Stories', '/stories'],
      ['Dashboard', '/dashboard'],
      ['Profile', '/profile'],
      ['Create Pool', '/pools/new'],
    ];

    links.forEach(([name, href]) => {
      expect(screen.getByRole('link', { name })).toHaveAttribute('href', href);
    });
  });

  it('opens MobileMenu when the mobile menu button is clicked', async () => {
    render(<Navbar />);

    await userEvent.click(screen.getByRole('button', { name: 'Open menu' }));

    expect(screen.getByRole('dialog')).toHaveTextContent('Mobile Menu');
  });
});
