import React from 'react';
import { render, screen } from '@testing-library/react';
import { PoolCard } from '@/components/PoolCard';
import type { Pool } from '@/src/store/poolsStore';

// next/link renders a plain <a> in tests
jest.mock('next/link', () => {
  const MockLink = ({
    href,
    children,
    className,
  }: {
    href: string;
    children: React.ReactNode;
    className?: string;
  }) => (
    <a href={href} className={className}>
      {children}
    </a>
  );
  MockLink.displayName = 'MockLink';
  return MockLink;
});

const basePool: Pool = {
  id: 'pool-42',
  title: 'Test Pool',
  description: 'A test pool description',
  category: 'Education',
  status: 'Active',
  target: 1000,
  raised: 250,
  imageColor: '#ff0000',
  creator: 'GABCDEF1234567890XYZ',
};

describe('PoolCard', () => {
  describe('progress percentage', () => {
    it('calculates progress correctly', () => {
      render(<PoolCard pool={basePool} />);
      expect(screen.getByText('25%')).toBeInTheDocument();
    });

    it('clamps progress to 100% when raised exceeds target', () => {
      const pool: Pool = { ...basePool, raised: 2000, target: 1000 };
      render(<PoolCard pool={pool} />);
      expect(screen.getByText('100%')).toBeInTheDocument();
    });

    it('shows 0% when nothing has been raised', () => {
      const pool: Pool = { ...basePool, raised: 0, target: 1000 };
      render(<PoolCard pool={pool} />);
      expect(screen.getByText('0%')).toBeInTheDocument();
    });
  });

  describe('status badge', () => {
    it('shows "Open" badge when pool status is Active', () => {
      render(<PoolCard pool={{ ...basePool, status: 'Active' }} />);
      expect(screen.getByText('Open')).toBeInTheDocument();
    });

    it('shows "Closed" badge when pool status is Completed', () => {
      render(<PoolCard pool={{ ...basePool, status: 'Completed' }} />);
      expect(screen.getByText('Closed')).toBeInTheDocument();
    });
  });

  describe('creator address', () => {
    it('truncates a long creator address', () => {
      render(<PoolCard pool={basePool} />);
      // basePool.creator = 'GABCDEF1234567890XYZ'
      // slice(0,6) = 'GABCDE', slice(-4) = '0XYZ'
      expect(screen.getByText('GABCDE...0XYZ')).toBeInTheDocument();
    });

    it('shows "Anonymous" when creator is an empty string', () => {
      render(<PoolCard pool={{ ...basePool, creator: '' }} />);
      expect(screen.getByText('Anonymous')).toBeInTheDocument();
    });

    it('shows "Anonymous" when creator is undefined', () => {
      const pool = { ...basePool };
      delete pool.creator;
      render(<PoolCard pool={pool} />);
      expect(screen.getByText('Anonymous')).toBeInTheDocument();
    });
  });

  describe('link', () => {
    it('links to /pools/[id]', () => {
      render(<PoolCard pool={basePool} />);
      const link = screen.getByRole('link');
      expect(link).toHaveAttribute('href', '/pools/pool-42');
    });
  });
});
