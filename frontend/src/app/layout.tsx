import type { Metadata } from "next";
// import { DM_Sans, Geist, Geist_Mono } from "next/font/google";
// import "./globals.css";
// import { Anton } from "next/font/google";
import "./globals.css";
import { Toaster } from "@/components/ui/sonner";


// const geistSans = Geist({
//   variable: "--font-geist-sans",
//   subsets: ["latin"],
// });

// const geistMono = Geist_Mono({
//   variable: "--font-geist-mono",
//   subsets: ["latin"],
// });

// const anton = Anton({
//   weight: ["400"], // Anton only has one weight
//   subsets: ["latin"],
//   display: "swap",
//   variable: "--font-anton",
// });

// const dmSans = DM_Sans({
//   subsets: ["latin"],
//   weight: ["400", "500", "700", "900"], // Choose the weights you need
//   display: "swap",
//   variable: "--font-dmsans",
// });

export const metadata: Metadata = {

  authors: [{ name: "Nevo" }],

  robots: {
    index: true,
    follow: true,
  },
  metadataBase: new URL("https://nevo.app"),
  title: "Nevo",
  description: "Decentralized Donation Pools on Stellar",
  keywords: [
    "decentralized security",
    "nevo",
    "payment",
    "security",
    "automated rewards",
    "trustless",
    "Web3 payment",
    "donation",
    "crowd funding",
    "stellar",
    "pools",
    "crypto payment",
  ],
  openGraph: {
    title: "Nevo - Decentralized Donation Pools on Stellar",
    description:
      "Nevo is a decentralized platform that reimagines charitable giving through blockchain technology. Create transparent donation pools, accept multiple assets, and let idle funds generate yields while maintaining complete control over disbursements",
    url: "https://nevo.app",
    siteName: "nevo",
    images: [
      {
        url: "https://nevo.app/logo.jpeg",
        width: 1200,
        height: 630,
        alt: "nevo - Decentralized Donation Pools on Stellar",
      },
    ],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Nevo - Decentralized Donation Pools on Stellar",
    description:
      "Nevo is a decentralized platform that reimagines charitable giving through blockchain technology. Create transparent donation pools, accept multiple assets, and let idle funds generate yields while maintaining complete control over disbursements",
    images: ["https://nevo.app/logo.jpeg"],
    creator: "@nevoapp",
  },

  icons: {
    icon: [
      { url: "/Group 1.svg" },
      {
        url: "/Group 1.svg",
        sizes: "192x192",
        type: "image/svg+xml",
      },
      {
        url: "/Group 1.svg",
        sizes: "512x512",
        type: "image/svg+xml",
      },
    ],
    apple: [
      {
        url: "/Group 1.svg",
        sizes: "180x180",
        type: "image/svg+xml",
      },
    ],
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`bg-no-repeat bg-fixed bg h-full bg-cover antialiased font-dmsans`}
        suppressHydrationWarning={true}
      >
        <main className="mt-28 ">{children}</main>
        <Toaster />
      </body>
    </html>
  );
}
