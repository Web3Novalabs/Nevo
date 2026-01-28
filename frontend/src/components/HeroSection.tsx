import { ArrowRight, Shield } from "lucide-react";

export const HeroSection = () => {
  return (
    <section className="pt-32 pb-20 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
        <div>
          <h1 className="text-5xl sm:text-6xl font-bold text-slate-900 dark:text-white mb-6 leading-tight">
            Secure, Transparent Donation Pools on{" "}
            <span className="bg-gradient-to-r from-blue-600 to-cyan-500 bg-clip-text text-transparent">
              Stellar
            </span>
          </h1>
          <p className="text-xl text-slate-600 dark:text-slate-300 mb-8 leading-relaxed">
            Empower collective giving with blockchain transparency. Create
            donation pools that generate yield, minimize costs, and ensure every
            dollar counts.
          </p>
          <div className="flex flex-col sm:flex-row gap-4">
            <button className="bg-blue-600 hover:bg-blue-700 text-white px-8 py-3 rounded-lg font-semibold flex items-center justify-center gap-2 transition transform hover:scale-105">
              Start Creating Pools <ArrowRight size={20} />
            </button>
            <button className="border-2 border-blue-600 text-blue-600 dark:text-blue-400 dark:border-blue-400 hover:bg-blue-50 dark:hover:bg-blue-950 px-8 py-3 rounded-lg font-semibold transition">
              Learn More
            </button>
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
          <div className="relative bg-gradient-to-br from-blue-500 to-cyan-500 rounded-3xl h-full flex items-center justify-center">
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
