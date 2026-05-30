import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { MobileMenu } from '@/components/MobileMenu';

beforeAll(() => {
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: jest.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
    })),
  });
});

describe('MobileMenu', () => {
  it('renders navigation links when open', () => {
    render(<MobileMenu open onClose={jest.fn()} />);
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Pools' })).toHaveAttribute(
      'href',
      '/pools'
    );
    expect(screen.getByRole('link', { name: 'Dashboard' })).toHaveAttribute(
      'href',
      '/dashboard'
    );
  });

  it('calls onClose when Escape is pressed', () => {
    const onClose = jest.fn();
    render(<MobileMenu open onClose={onClose} />);
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when backdrop is clicked', () => {
    const onClose = jest.fn();
    render(<MobileMenu open onClose={onClose} />);
    fireEvent.click(screen.getByRole('button', { name: 'Close menu overlay' }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when a nav link is clicked', () => {
    const onClose = jest.fn();
    render(<MobileMenu open onClose={onClose} />);
    fireEvent.click(screen.getByRole('link', { name: 'Home' }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
