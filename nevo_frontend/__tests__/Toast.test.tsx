import React from 'react';
import { render, screen, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { toast, ToastContainer } from '@/components/Toast';

describe('Toast', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('renders a toast when toast() is called', () => {
    render(<ToastContainer />);
    act(() => {
      toast('Hello world');
    });
    expect(screen.getByText('Hello world')).toBeInTheDocument();
  });

  it('queues multiple toasts', () => {
    render(<ToastContainer />);
    act(() => {
      toast('First');
      toast('Second');
      toast('Third');
    });
    expect(screen.getByText('First')).toBeInTheDocument();
    expect(screen.getByText('Second')).toBeInTheDocument();
    expect(screen.getByText('Third')).toBeInTheDocument();
  });

  it('auto-dismisses a toast after 3000ms', () => {
    render(<ToastContainer />);
    act(() => {
      toast('Auto dismiss');
    });
    expect(screen.getByText('Auto dismiss')).toBeInTheDocument();

    act(() => {
      jest.advanceTimersByTime(3000);
    });
    expect(screen.queryByText('Auto dismiss')).not.toBeInTheDocument();
  });

  it('dismisses on close button click', async () => {
    const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
    render(<ToastContainer />);
    act(() => {
      toast('Dismiss me');
    });
    await user.click(screen.getByLabelText('Dismiss notification'));
    expect(screen.queryByText('Dismiss me')).not.toBeInTheDocument();
  });

  it('renders success type with green background', () => {
    render(<ToastContainer />);
    act(() => {
      toast('Success', 'success');
    });
    const el = screen.getByRole('status');
    expect(el).toHaveClass('bg-green-600');
  });

  it('renders error type with red background', () => {
    render(<ToastContainer />);
    act(() => {
      toast('Error', 'error');
    });
    const el = screen.getByRole('status');
    expect(el).toHaveClass('bg-red-600');
  });

  it('renders info type with blue background', () => {
    render(<ToastContainer />);
    act(() => {
      toast('Info', 'info');
    });
    const el = screen.getByRole('status');
    expect(el).toHaveClass('bg-blue-600');
  });

  it('defaults to success type when not specified', () => {
    render(<ToastContainer />);
    act(() => {
      toast('Default');
    });
    const el = screen.getByRole('status');
    expect(el).toHaveClass('bg-green-600');
  });

  it('returns null when no toasts are present', () => {
    const { container } = render(<ToastContainer />);
    expect(container.firstChild).toBeNull();
  });

  it('does not throw when toast() called before ToastContainer mounts', () => {
    expect(() => toast('early')).not.toThrow();
  });

  it('cleans up addToastFn on unmount', () => {
    const { unmount } = render(<ToastContainer />);
    unmount();
    expect(() => toast('After unmount')).not.toThrow();
  });

  it('pauses auto-dismiss timer on hover and resumes on mouse leave', async () => {
    const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
    render(<ToastContainer />);
    act(() => {
      toast('Hover test');
    });
    const toastEl = screen.getByRole('status');
    await user.hover(toastEl);

    act(() => {
      jest.advanceTimersByTime(4000);
    });
    expect(screen.getByText('Hover test')).toBeInTheDocument();

    await user.unhover(toastEl);
    act(() => {
      jest.advanceTimersByTime(3000);
    });
    expect(screen.queryByText('Hover test')).not.toBeInTheDocument();
  });

  it('pauses auto-dismiss timer on focus and resumes on blur', async () => {
    const user = userEvent.setup({ advanceTimers: jest.advanceTimersByTime });
    render(<ToastContainer />);
    act(() => {
      toast('Focus test');
    });
    const dismissBtn = screen.getByLabelText('Dismiss notification');
    await user.tab();
    expect(dismissBtn).toHaveFocus();

    act(() => {
      jest.advanceTimersByTime(4000);
    });
    expect(screen.getByText('Focus test')).toBeInTheDocument();

    dismissBtn.blur();
    act(() => {
      jest.advanceTimersByTime(3000);
    });
    expect(screen.queryByText('Focus test')).not.toBeInTheDocument();
  });
});

