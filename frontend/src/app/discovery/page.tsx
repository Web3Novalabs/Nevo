import Footer from "@/components/Footer";
import Navigation from "@/components/Navigation";
import { PoolBrowser } from "@/components/pools/PoolBrowser";

export default function DiscoveryPage() {
  return (
    <div className="min-h-screen bg-[#0F172A]">
      <Navigation />
      <main>
        <PoolBrowser />
      </main>
      <Footer />
    </div>
  );
}
