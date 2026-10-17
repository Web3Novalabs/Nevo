import React from 'react';
import { render, screen } from '@testing-library/react';
import AboutPage from '@/app/about/page';
import HelpPage from '@/app/help/page';
import PrivacyPage from '@/app/privacy/page';
import TermsPage from '@/app/terms/page';
import StoriesPage from '@/app/stories/page';

jest.mock('next/link', () => {
    const MockLink = ({
        href,
        children,
        ...props
    }: {
        href: string;
        children: React.ReactNode;
        [key: string]: unknown;
    }) => (
        <a href={href} {...props}>
            {children}
        </a>
    );

    MockLink.displayName = 'MockLink';
    return MockLink;
});

const VALID_APP_ROUTES = [
    '/about',
    '/help',
    '/privacy',
    '/terms',
    '/stories',
    '/pools/new',
    '/pools',
    '/login',
    '/profile',
    '/transactions',
    '/dashboard',
    '/donations',
];

const PAGE_CASES = [
    { name: 'about', Page: AboutPage, heading: 'Our Mission' },
    { name: 'help', Page: HelpPage, heading: 'Help & FAQ' },
    { name: 'privacy', Page: PrivacyPage, heading: 'Privacy Policy' },
    { name: 'terms', Page: TermsPage, heading: 'Terms of Service' },
    { name: 'stories', Page: StoriesPage, heading: 'Success Stories' },
] as const;

describe('static content pages', () => {
    it.each(PAGE_CASES)('$name renders its primary heading without crashing', ({ Page, heading }) => {
        expect(() => render(<Page />)).not.toThrow();
        expect(screen.getByRole('heading', { name: heading })).toBeInTheDocument();
    });

    it.each(PAGE_CASES)('$name uses valid internal routes for internal links', ({ Page }) => {
        const { container } = render(<Page />);
        const internalLinks = Array.from(container.querySelectorAll('a[href]'))
            .map((link) => link.getAttribute('href'))
            .filter(
                (href): href is string =>
                    !!href &&
                    href.startsWith('/') &&
                    !href.startsWith('//') &&
                    !href.startsWith('/_')
            );

        internalLinks.forEach((href) => {
            expect(VALID_APP_ROUTES).toContain(href);
        });
    });
});
