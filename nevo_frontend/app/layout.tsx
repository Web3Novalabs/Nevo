import type { Metadata, Viewport } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";
import Navbar from "@/components/Navbar";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: {
    default: "Nevo",
    template: "%s | Nevo",
  },
  description:
    "Nevo is an open-source donation platform built on Stellar. Create transparent, secure, and efficient fundraising pools on-chain.",
  openGraph: {
    title: "Nevo",
    description:
      "Transparent, secure, and efficient fundraising pools on Stellar.",
    url: "https://nevo.app",
    siteName: "Nevo",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Nevo",
    description:
      "Transparent, secure, and efficient fundraising pools on Stellar.",
  },
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col">
        <Navbar />
        {children}
      </body>
    </html>
  );
}
