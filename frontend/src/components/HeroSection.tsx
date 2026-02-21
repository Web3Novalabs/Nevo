import { ArrowRight, Shield, Compass } from "lucide-react";
import Link from "next/link";

export const HeroSection = () => {
  return (
    <section className="pt-32 pb-20 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
        <div>
          <h1 className="text-5xl sm:text-6xl font-bold text-slate-900 dark:text-white mb-6 leading-tight">
            Secure, Transparent Donation Pools on{" "}
            <span className="bg-[#50C878] bg-clip-text text-transparent">
              Stellar
            </span>
          </h1>
          <p className="text-xl text-slate-600 dark:text-slate-300 mb-8 leading-relaxed">
            Empower collective giving with blockchain transparency. Create
            donation pools that generate yield, minimize costs, and ensure every
            dollar counts.
          </p>
          <div className="flex flex-col sm:flex-row gap-4">
            <button className="bg-[#50C878] text-black px-8 py-3 rounded-lg font-semibold flex items-center justify-center gap-2 transition-all duration-300 hover:-translate-y-1 active:scale-95 active:shadow-[0_0_20px_rgba(80,200,120,0.6)] cursor-pointer">
              Start Creating Pools <ArrowRight size={20} />
            </button>
            <Link href="/discovery" className="border-2 border-gray-600 text-gray-100  hover:bg-blue-50 hover:text-[#50C878] px-8 py-3 rounded-lg font-semibold transition cursor-pointer flex items-center justify-center gap-2">
              <Compass size={20} /> Discover Pools
            </Link>
          </div>
          <div className="mt-12 flex gap-8">
            <div>
              <div className="text-3xl font-bold text-slate-900 dark:text-white">
                100%
              </div>
              <p className="text-sm text-slate-600 dark:text-slate-400">
                Transparent
              </p>
            </div>
            <div>
              <div className="text-3xl font-bold text-slate-900 dark:text-white">
                0.1%
              </div>
              <p className="text-sm text-slate-600 dark:text-slate-400">
                Avg Fee
              </p>
            </div>
            <div>
              <div className="text-3xl font-bold text-slate-900 dark:text-white">
                Instant
              </div>
              <p className="text-sm text-slate-600 dark:text-slate-400">
                Settlement
              </p>
            </div>
          </div>
        </div>
        <div className="relative h-96 sm:h-full min-h-96">
          <div className="absolute inset-0 bg-gradient-to-br from-blue-500/20 to-cyan-500/20 rounded-3xl blur-3xl"></div>
          <div
            className="relative bg-gradient-to-br from-[#50C878] to-[#14B8A6]
               rounded-3xl h-full flex items-center justify-center"
          >
            <div className="text-center text-white">
              <Shield size={80} className="mx-auto mb-4 opacity-90" />
              <p className="text-lg font-semibold">Secured on Stellar</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
