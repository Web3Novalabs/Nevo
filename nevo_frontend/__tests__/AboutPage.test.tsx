import React from 'react';
import { render, screen } from '@testing-library/react';
import AboutPage from '@/app/about/page';
import { AboutPageSkeleton } from '@/app/about/AboutPageSkeleton';

describe('AboutPage', () => {
  it('renders mission statement', () => {
    render(<AboutPage />);
    expect(
      screen.getByRole('heading', { name: 'Our Mission' })
    ).toBeInTheDocument();
    expect(
      screen.getByText(/empower anyone, anywhere to create transparent/i)
    ).toBeInTheDocument();
  });

  it('renders company values section', () => {
    render(<AboutPage />);
    expect(
      screen.getByRole('heading', { name: 'Our Values' })
    ).toBeInTheDocument();
    expect(screen.getByText('Transparency')).toBeInTheDocument();
    expect(screen.getByText('Security')).toBeInTheDocument();
    expect(screen.getByText('Community')).toBeInTheDocument();
    expect(screen.getByText('Accessibility')).toBeInTheDocument();
  });

  it('renders team section with member bios', () => {
    render(<AboutPage />);
    expect(
      screen.getByRole('heading', { name: 'Meet the Team' })
    ).toBeInTheDocument();
    expect(screen.getByText('Alex Morgan')).toBeInTheDocument();
    expect(screen.getByText('Founder & CEO')).toBeInTheDocument();
    expect(screen.getByText(/Blockchain enthusiast/i)).toBeInTheDocument();
  });

  it('renders testimonials', () => {
    render(<AboutPage />);
    expect(
      screen.getByRole('heading', { name: 'What They Say' })
    ).toBeInTheDocument();
    expect(
      screen.getByText(/revolutionized how we collect donations/i)
    ).toBeInTheDocument();
    expect(screen.getByText('Maria Santos')).toBeInTheDocument();
  });

  it('renders contact information', () => {
    render(<AboutPage />);
    expect(
      screen.getByRole('heading', { name: 'Get in Touch' })
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Have questions or want to learn more/i)
    ).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Email Us' })).toHaveAttribute(
      'href',
      'mailto:hello@nevo.app'
    );
    expect(screen.getByRole('link', { name: 'Contact Form' })).toHaveAttribute(
      'href',
      '/contact'
    );
  });

  it('renders social media links for team members', () => {
    render(<AboutPage />);
    expect(screen.getByLabelText(/Alex Morgan's Twitter/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/Alex Morgan's GitHub/i)).toBeInTheDocument();
  });

  it('is mobile responsive with proper text sizes', () => {
    const { container } = render(<AboutPage />);
    const main = container.querySelector('main');
    expect(main).toHaveClass('max-w-5xl');
  });
});

describe('AboutPageSkeleton', () => {
  it('renders loading skeleton with accessibility attributes', () => {
    render(<AboutPageSkeleton />);
    const main = screen.getByRole('main');
    expect(main).toHaveAttribute('aria-busy', 'true');
    expect(main).toHaveAttribute('aria-label', 'Loading about page');
  });

  it('renders skeleton placeholders', () => {
    render(<AboutPageSkeleton />);
    const skeletonElements = document.querySelectorAll('.animate-pulse');
    expect(skeletonElements.length).toBeGreaterThan(0);
  });
});
