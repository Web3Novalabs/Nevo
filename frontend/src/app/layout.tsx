import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Nevo - Secure Donation Pools on Stellar | Transparent Fundraising",
  description:
    "Create transparent, secure donation pools on Stellar blockchain. Earn DeFi yield, minimize costs, and track every donation in real-time. Start fundraising today.",
  keywords: [
    "donation pools",
    "blockchain",
    "Stellar",
    "XLM",
    "USDC",
    "DeFi",
    "fundraising",
    "transparent",
    "secure",
  ],
  authors: [{ name: "Nevo" }],
  openGraph: {
    type: "website",
    locale: "en_US",
    url: "https://nevo.app",
    siteName: "Nevo",
    title: "Nevo - Secure Donation Pools on Stellar",
    description:
      "Create transparent, secure donation pools on Stellar blockchain with low fees and DeFi yield generation.",
    images: [
      {
        url: "https://nevo.app/og-image.png",
        width: 1200,
        height: 630,
        alt: "Nevo - Secure Donation Pools on Stellar",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Nevo - Secure Donation Pools on Stellar",
    description:
      "Create transparent, secure donation pools on Stellar blockchain with low fees and DeFi yield generation.",
    images: ["https://nevo.app/og-image.png"],
  },
  robots: {
    index: true,
    follow: true,
  },
  metadataBase: new URL("https://nevo.app"),
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        {children}
      </body>
    </html>
  );
}
