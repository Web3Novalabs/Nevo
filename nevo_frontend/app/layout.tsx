import type { Metadata, Viewport } from 'next';
import './globals.css';
import Navbar from '@/components/Navbar';
import { RateLimitToast } from '@/components/RateLimitToast';

// Using system fonts to avoid network fetch during build (CI friendly)

export const metadata: Metadata = {
  metadataBase: new URL('https://nevo.app'),
  title: {
    default: 'Nevo',
    template: '%s | Nevo',
  },
  description:
    'Nevo is an open-source donation platform built on Stellar. Create transparent, secure, and efficient fundraising pools on-chain.',
  robots: {
    index: true,
    follow: true,
  },
  openGraph: {
    title: 'Nevo',
    description:
      'Transparent, secure, and efficient fundraising pools on Stellar.',
    url: 'https://nevo.app',
    siteName: 'Nevo',
    type: 'website',
    images: [{ url: '/opengraph-image.png', width: 1200, height: 630 }],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Nevo',
    description:
      'Transparent, secure, and efficient fundraising pools on Stellar.',
    images: ['/opengraph-image.png'],
  },
};

export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
};

const organizationJsonLd = {
  '@context': 'https://schema.org',
  '@type': 'Organization',
  name: 'Nevo',
  url: 'https://nevo.app',
  description:
    'Open-source donation platform built on Stellar for transparent, secure fundraising pools.',
  sameAs: ['https://github.com/Dami24-hub/Nevo'],
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="h-full antialiased" suppressHydrationWarning>
      <head>
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{
            __html: JSON.stringify(organizationJsonLd),
          }}
        />
        <script
          dangerouslySetInnerHTML={{
            __html: `(function(){try{var t=localStorage.getItem('nevo-theme');if(t==='dark'){document.documentElement.classList.add('dark')}else if(t==='light'){document.documentElement.classList.remove('dark')}else if(window.matchMedia('(prefers-color-scheme: dark)').matches){document.documentElement.classList.add('dark')}}catch(e){}})()`,
          }}
        />
      </head>
      <body className="min-h-full flex flex-col">
        <Navbar />
        {children}
        <RateLimitToast />
      </body>
    </html>
  );
}
