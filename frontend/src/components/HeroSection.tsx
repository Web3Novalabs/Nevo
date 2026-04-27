import { ArrowRight, Compass } from "lucide-react";
import Link from "next/link";

const stats = [
  { value: "$2.4M+", label: "Total Donated" },
  { value: "1,200+", label: "Active Pools" },
  { value: "~$0.00001", label: "Per Transaction" },
  { value: "3–5s", label: "Finality" },
];

export const HeroSection = () => {
  return (
    <section
      aria-label="Hero"
      className="relative pt-32 pb-20 px-4 sm:px-6 lg:px-8 overflow-hidden"
    >
      {/* Background gradient */}
      <div
        aria-hidden="true"
        className="absolute inset-0 bg-gradient-to-br from-[#0F172A] via-[#0F2A1A] to-[#0F172A]"
      />
      <div
        aria-hidden="true"
        className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[500px] bg-[#50C878]/10 rounded-full blur-3xl pointer-events-none"
      />

      <div className="relative max-w-7xl mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
          <div>
            <h1 className="text-5xl sm:text-6xl font-bold text-white mb-6 leading-tight">
              Secure, Transparent Donation Pools on{" "}
              <span className="text-[#50C878]">Stellar</span>
            </h1>
            <p className="text-xl text-slate-300 mb-8 leading-relaxed">
              Empower collective giving with blockchain transparency. Create
              donation pools that generate yield, minimize costs, and ensure
              every dollar counts.
            </p>

            <div className="flex flex-col sm:flex-row gap-4">
              <Link
                href="/dashboard"
                className="bg-[#50C878] text-black px-8 py-3 rounded-lg font-semibold flex items-center justify-center gap-2 transition-all duration-300 hover:-translate-y-1 active:scale-95 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#50C878]"
                aria-label="Create a donation pool"
              >
                Create Pool <ArrowRight size={20} aria-hidden="true" />
              </Link>
              <Link
                href="/explore"
                className="border-2 border-gray-600 text-gray-100 hover:bg-white/5 hover:text-[#50C878] px-8 py-3 rounded-lg font-semibold transition flex items-center justify-center gap-2 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#50C878]"
                aria-label="Browse donation pools"
              >
                <Compass size={20} aria-hidden="true" /> Browse Pools
              </Link>
            </div>

            {/* Trust stats */}
            <dl className="mt-12 grid grid-cols-2 sm:grid-cols-4 gap-6">
              {stats.map(({ value, label }) => (
                <div key={label}>
                  <dt className="text-sm text-slate-400">{label}</dt>
                  <dd className="text-2xl font-bold text-white mt-1">{value}</dd>
                </div>
              ))}
            </dl>
          </div>

          {/* Visual panel */}
          <div
            aria-hidden="true"
            className="relative h-96 lg:h-full min-h-96"
          >
            <div className="absolute inset-0 bg-gradient-to-br from-blue-500/20 to-cyan-500/20 rounded-3xl blur-3xl" />
            <div className="relative bg-gradient-to-br from-[#50C878]/80 to-[#14B8A6]/80 rounded-3xl h-full flex flex-col items-center justify-center gap-6 p-8">
              <div className="grid grid-cols-2 gap-4 w-full max-w-xs">
                {[
                  { label: "Pool Balance", value: "12,450 XLM" },
                  { label: "Contributors", value: "84" },
                  { label: "Yield Earned", value: "320 USDC" },
                  { label: "Disbursed", value: "8,000 XLM" },
                ].map(({ label, value }) => (
                  <div
                    key={label}
                    className="bg-white/20 backdrop-blur-sm rounded-xl p-4 text-white"
                  >
                    <p className="text-xs opacity-80">{label}</p>
                    <p className="text-lg font-bold mt-1">{value}</p>
                  </div>
                ))}
              </div>
              <p className="text-white/90 text-sm font-medium">
                Secured on Stellar · Powered by Soroban
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
};
