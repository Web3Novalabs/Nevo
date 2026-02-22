import Navigation from "@/components/Navigation";
import Footer from "@/components/Footer";
import { PoolGrid } from "@/components/PoolGrid";

export default function ExplorePage() {
  return (
    <div className="bg-[#0F172A] min-h-screen flex flex-col">
      <Navigation />

      <main className="flex-grow pt-24 pb-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          {/* Header Section */}
          <div className="text-center mb-16">
            <h1 className="text-4xl md:text-5xl font-extrabold text-white mb-6 tracking-tight">
              Explore{" "}
              <span className="text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-cyan-300">
                Pools
              </span>
            </h1>
            <p className="text-xl text-slate-400 max-w-2xl mx-auto">
              Discover and contribute to causes that matter. Every pool is
              secured by smart contracts for full transparency and impact.
            </p>
          </div>

          {/* Grid Section */}
          <PoolGrid />
        </div>
      </main>

      <Footer />
    </div>
  );
}
