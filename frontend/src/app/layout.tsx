import type { Metadata } from "next";
// import { DM_Sans, Geist, Geist_Mono } from "next/font/google";
// import "./globals.css";
// import { Anton } from "next/font/google";
import "./globals.css";
import { Toaster } from "@/components/ui/sonner";
import PageProgressWrapper from "@/components/PageProgressWrapper";
import ScrollButton from "@/components/ScrollButton";


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
  metadataBase: new URL("https://nevo.app"),
  robots: {
    index: true,
    follow: true,
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
        className={`bg-no-repeat bg-fixed bg h-full bg-cover antialiased font-dmsans relative`}
        suppressHydrationWarning={true}
      >
        <PageProgressWrapper />
        <ScrollButton />
        <main className="">{children}</main>
        <Toaster />
      </body>
    </html>
  );
}
