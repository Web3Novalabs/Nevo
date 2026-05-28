import type { Metadata } from "next";
import "./globals.css";
import { ToastProvider, ToastContainer } from "@/components/ui/toast";

export const metadata: Metadata = {
  title: "Nevo - Secure Donation Pools on Stellar",
  description:
    "Create transparent, secure donation pools on Stellar blockchain with low fees and DeFi yield generation.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body suppressHydrationWarning={true}>
        <ToastProvider>
          <main>{children}</main>
          <ToastContainer />
        </ToastProvider>
      </body>
    </html>
  );
}
