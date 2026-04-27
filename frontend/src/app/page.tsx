import type { Metadata } from "next";
import Navigation from "@/components/Navigation";
import { HeroSection } from "@/components/HeroSection";
import { FeaturesSection } from "@/components/FeaturesSection";
import { HowItWorksSection } from "@/components/HowItWorksSection";
import { SecuritySection } from "@/components/SecuritySection";
import { CTASection } from "@/components/CTASection";
import Footer from "@/components/Footer";

export const metadata: Metadata = {
  title: "Nevo – Decentralized Donation Pools on Stellar",
  description:
    "Create transparent, secure donation pools on Stellar blockchain. Accept XLM, USDC, and custom assets with near-zero fees and DeFi yield generation.",
};

export default function Home() {
  return (
    <div className="bg-[#0F172A]">
      <Navigation />
      <main id="main-content">
        <HeroSection />
        <FeaturesSection />
        <HowItWorksSection />
        <SecuritySection />
        <CTASection />
      </main>
      <Footer />
    </div>
  );
}
