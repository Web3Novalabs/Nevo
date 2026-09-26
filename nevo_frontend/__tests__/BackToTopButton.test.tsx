import React from 'react';
import { render, screen, fireEvent, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { BackToTopButton } from '@/components/BackToTopButton';

describe('BackToTopButton', () => {
  let originalScrollTo: typeof window.scrollTo;

  beforeEach(() => {
    originalScrollTo = window.scrollTo;
    window.scrollTo = jest.fn();
    jest.spyOn(window, 'requestAnimationFrame').mockImplementation((cb) => {
      cb(0);
      return 0;
    });
  });

  afterEach(() => {
    window.scrollTo = originalScrollTo;
    jest.restoreAllMocks();
  });

  it('is hidden (aria-hidden="true") below the scroll threshold', () => {
    Object.defineProperty(window, 'scrollY', { value: 100, configurable: true });
    render(<BackToTopButton threshold={400} />);
    const button = screen.getByRole('button', { hidden: true });
    expect(button).toHaveAttribute('aria-hidden', 'true');
    expect(button).toHaveAttribute('tabIndex', '-1');
  });

  it('becomes visible (aria-hidden="false") above the scroll threshold', () => {
    Object.defineProperty(window, 'scrollY', { value: 100, configurable: true });
    render(<BackToTopButton threshold={400} />);
    const button = screen.getByRole('button', { hidden: true });
    expect(button).toHaveAttribute('aria-hidden', 'true');

    act(() => {
      Object.defineProperty(window, 'scrollY', { value: 500, configurable: true });
      fireEvent.scroll(window);
    });

    expect(button).toHaveAttribute('aria-hidden', 'false');
    expect(button).toHaveAttribute('tabIndex', '0');
  });

  it('mocks window.scrollTo and verifies it is called on click', async () => {
    Object.defineProperty(window, 'scrollY', { value: 500, configurable: true });
    render(<BackToTopButton threshold={400} />);
    const button = screen.getByRole('button');

    await userEvent.click(button);
    expect(window.scrollTo).toHaveBeenCalledWith({
      top: 0,
      behavior: 'smooth',
    });
  });
});
