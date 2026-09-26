import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ThemeToggle from '@/components/ThemeToggle';
import { useThemeStore } from '@/src/store/themeStore';

beforeEach(() => {
  localStorage.clear();
  document.documentElement.classList.remove('dark');
  useThemeStore.setState({ theme: 'light' });
});

describe('ThemeToggle', () => {
  it('renders a button', () => {
    render(<ThemeToggle />);
    expect(screen.getByRole('button')).toBeInTheDocument();
  });

  it('has type="button" to prevent form submissions', () => {
    render(<ThemeToggle />);
    expect(screen.getByRole('button')).toHaveAttribute('type', 'button');
  });

  it('has aria-pressed=false in light mode', () => {
    render(<ThemeToggle />);
    expect(screen.getByRole('button')).toHaveAttribute('aria-pressed', 'false');
  });

  it('has aria-pressed=true in dark mode', () => {
    localStorage.setItem('nevo-theme', 'dark');
    render(<ThemeToggle />);
    expect(screen.getByRole('button')).toHaveAttribute('aria-pressed', 'true');
  });

  it('has aria-label to switch to dark mode when in light mode', () => {
    render(<ThemeToggle />);
    expect(screen.getByRole('button')).toHaveAttribute(
      'aria-label',
      'Switch to dark mode'
    );
  });

  it('has aria-label to switch to light mode when in dark mode', () => {
    localStorage.setItem('nevo-theme', 'dark');
    render(<ThemeToggle />);
    expect(screen.getByRole('button')).toHaveAttribute(
      'aria-label',
      'Switch to light mode'
    );
  });

  it('shows MoonIcon in light mode', () => {
    render(<ThemeToggle />);
    const svg = screen.getByRole('button').querySelector('svg');
    expect(svg).toBeInTheDocument();
    expect(svg!.querySelector('path')).toHaveAttribute(
      'd',
      expect.stringContaining('21.752 15.002')
    );
  });

  it('shows SunIcon in dark mode', () => {
    localStorage.setItem('nevo-theme', 'dark');
    render(<ThemeToggle />);
    const svg = screen.getByRole('button').querySelector('svg');
    expect(svg).toBeInTheDocument();
    expect(svg!.querySelector('path')).toHaveAttribute(
      'd',
      expect.stringContaining('M12 3v2.25')
    );
  });

  it('toggles from light to dark on click', async () => {
    const user = userEvent.setup();
    render(<ThemeToggle />);
    await user.click(screen.getByRole('button'));
    expect(useThemeStore.getState().theme).toBe('dark');
  });

  it('toggles from dark to light on click', async () => {
    localStorage.setItem('nevo-theme', 'dark');
    const user = userEvent.setup();
    render(<ThemeToggle />);
    await user.click(screen.getByRole('button'));
    expect(useThemeStore.getState().theme).toBe('light');
  });

  it('updates aria-pressed after toggling', async () => {
    const user = userEvent.setup();
    render(<ThemeToggle />);
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-pressed', 'false');
    await user.click(button);
    expect(button).toHaveAttribute('aria-pressed', 'true');
  });

  it('calls toggleTheme on click', async () => {
    const spy = jest.spyOn(useThemeStore.getState(), 'toggleTheme');
    const user = userEvent.setup();
    render(<ThemeToggle />);
    await user.click(screen.getByRole('button'));
    expect(spy).toHaveBeenCalledTimes(1);
    spy.mockRestore();
  });

  it('syncs with localStorage dark value on mount', () => {
    localStorage.setItem('nevo-theme', 'dark');
    render(<ThemeToggle />);
    expect(useThemeStore.getState().theme).toBe('dark');
  });

  it('syncs with localStorage light value on mount', () => {
    localStorage.setItem('nevo-theme', 'light');
    render(<ThemeToggle />);
    expect(useThemeStore.getState().theme).toBe('light');
  });

  it('falls back to matchMedia when no localStorage value', () => {
    const originalMatchMedia = window.matchMedia;
    window.matchMedia = jest.fn().mockImplementation((query) => ({
      matches: query === '(prefers-color-scheme: dark)',
      media: query,
      onchange: null,
      addListener: jest.fn(),
      removeListener: jest.fn(),
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      dispatchEvent: jest.fn(),
    }));
    render(<ThemeToggle />);
    expect(useThemeStore.getState().theme).toBe('dark');
    window.matchMedia = originalMatchMedia;
  });

  it('falls back to light when no localStorage and matchMedia is not dark', () => {
    const originalMatchMedia = window.matchMedia;
    window.matchMedia = jest.fn().mockImplementation(() => ({
      matches: false,
      media: '',
      onchange: null,
      addListener: jest.fn(),
      removeListener: jest.fn(),
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
      dispatchEvent: jest.fn(),
    }));
    render(<ThemeToggle />);
    expect(useThemeStore.getState().theme).toBe('light');
    window.matchMedia = originalMatchMedia;
  });

  it('handles localStorage.getItem throwing an error', () => {
    const spy = jest
      .spyOn(Storage.prototype, 'getItem')
      .mockImplementation(() => {
        throw new Error('localStorage unavailable');
      });
    expect(() => render(<ThemeToggle />)).not.toThrow();
    expect(screen.getByRole('button')).toBeInTheDocument();
    spy.mockRestore();
  });
});
