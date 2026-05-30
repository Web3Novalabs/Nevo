import React from 'react';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { EmptyState } from '@/components/EmptyState';

describe('EmptyState', () => {
  it('renders title and description', () => {
    render(
      <EmptyState
        title="No pools yet"
        description="Create your first pool to get started."
      />
    );
    expect(screen.getByRole('status')).toBeInTheDocument();
    expect(screen.getByText('No pools yet')).toBeInTheDocument();
    expect(
      screen.getByText('Create your first pool to get started.')
    ).toBeInTheDocument();
  });

  it('renders primary CTA link', () => {
    render(
      <EmptyState
        title="No results"
        action={{ label: 'Create a Pool', href: '/pools/new' }}
      />
    );
    expect(
      screen.getByRole('link', { name: 'Create a Pool' })
    ).toHaveAttribute('href', '/pools/new');
  });

  it('calls onClick for secondary action', async () => {
    const onClear = jest.fn();
    render(
      <EmptyState
        title="No matching transactions"
        action={{ label: 'Clear filters', onClick: onClear, variant: 'secondary' }}
      />
    );
    await userEvent.click(screen.getByRole('button', { name: 'Clear filters' }));
    expect(onClear).toHaveBeenCalledTimes(1);
  });

  it('renders suggested next steps', () => {
    render(
      <EmptyState
        title="No contributions yet"
        steps={[
          { text: 'Connect your Stellar wallet' },
          { text: 'Choose an amount and donate' },
        ]}
      />
    );
    expect(screen.getByText('Connect your Stellar wallet')).toBeInTheDocument();
    expect(screen.getByText('Choose an amount and donate')).toBeInTheDocument();
  });
});
