import React from 'react';
import { render, screen, within } from '@testing-library/react';
import { Footer } from '@/components/Footer';

jest.mock('next/link', () => {
  const MockLink = ({
    href,
    children,
    ...rest
  }: {
    href: string;
    children: React.ReactNode;
    [key: string]: unknown;
  }) => (
    <a href={href} {...rest}>
      {children}
    </a>
  );
  MockLink.displayName = 'MockLink';
  return MockLink;
});

describe('Footer', () => {
  describe('copyright year', () => {
    afterEach(() => {
      jest.useRealTimers();
    });

    it('renders the current year from new Date().getFullYear()', () => {
      render(<Footer />);
      const year = new Date().getFullYear();
      expect(
        screen.getByText(`© ${year} Nevo. All rights reserved.`)
      ).toBeInTheDocument();
    });

    it('derives the year dynamically rather than hardcoding it', () => {
      jest.useFakeTimers();
      jest.setSystemTime(new Date('2031-06-15T12:00:00Z'));

      render(<Footer />);

      expect(
        screen.getByText('© 2031 Nevo. All rights reserved.')
      ).toBeInTheDocument();
    });
  });

  describe('navigation links', () => {
    beforeEach(() => {
      render(<Footer />);
    });

    it('renders the footer navigation landmark', () => {
      expect(
        screen.getByRole('navigation', { name: 'Footer navigation' })
      ).toBeInTheDocument();
    });

    it.each([
      ['About', '/about'],
      ['Terms', '/terms'],
      ['Privacy', '/privacy'],
    ])('renders a %s link pointing to %s', (name, href) => {
      const nav = screen.getByRole('navigation', { name: 'Footer navigation' });
      const link = within(nav).getByRole('link', { name });
      expect(link).toHaveAttribute('href', href);
    });
  });

  describe('social links', () => {
    beforeEach(() => {
      render(<Footer />);
    });

    it.each([
      ['Twitter', 'https://twitter.com/Web3Novalabs'],
      ['GitHub', 'https://github.com/Web3Novalabs/Nevo'],
    ])(
      'renders the %s link opening safely in a new tab with an aria-label',
      (label, href) => {
        const link = screen.getByRole('link', { name: label });
        expect(link).toHaveAttribute('href', href);
        expect(link).toHaveAttribute('target', '_blank');
        expect(link).toHaveAttribute('rel', 'noopener noreferrer');
        expect(link).toHaveAttribute('aria-label', label);
      }
    );

    it('every external link uses target="_blank" with rel="noopener noreferrer"', () => {
      const externalLinks = screen
        .getAllByRole('link')
        .filter((a) => a.getAttribute('href')?.startsWith('http'));

      expect(externalLinks).toHaveLength(2);
      externalLinks.forEach((a) => {
        expect(a).toHaveAttribute('target', '_blank');
        expect(a).toHaveAttribute('rel', 'noopener noreferrer');
        expect(a).toHaveAttribute('aria-label');
      });
    });
  });
});
