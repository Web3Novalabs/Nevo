import Navigation from "@/components/Navigation";
import Footer from "@/components/Footer";
import { Shield, Zap, Eye, TrendingUp, Lock, Heart } from "lucide-react";

export default function AboutUsPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-50 dark:from-slate-950 dark:via-blue-950 dark:to-slate-950">
      <Navigation />
      
      {/* Hero Section */}
      <section className="pt-32 pb-20 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h1 className="text-5xl sm:text-6xl font-bold text-slate-900 dark:text-white mb-6 leading-tight">
            About <span className="bg-gradient-to-r from-blue-600 to-cyan-500 bg-clip-text text-transparent">Nevo</span>
          </h1>
          <p className="text-xl text-slate-600 dark:text-slate-300 max-w-3xl mx-auto leading-relaxed">
            We&apos;re reimagining charitable giving through blockchain technology, making donations more transparent, efficient, and impactful.
          </p>
        </div>
      </section>

      {/* Mission Section */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-white dark:bg-slate-900/50">
        <div className="max-w-7xl mx-auto">
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
            <div>
              <h2 className="text-4xl sm:text-5xl font-bold text-slate-900 dark:text-white mb-6">
                Our Mission
              </h2>
              <p className="text-lg text-slate-600 dark:text-slate-300 mb-6 leading-relaxed">
                Nevo is a decentralized platform that empowers individuals and organizations to create transparent, secure donation pools on the Stellar blockchain. We believe that charitable giving should be accessible, transparent, and efficient.
              </p>
              <p className="text-lg text-slate-600 dark:text-slate-300 mb-6 leading-relaxed">
                Traditional donation platforms often charge high fees and lack transparency, meaning less money reaches the causes that need it most. Nevo solves this by leveraging blockchain technology to minimize costs, ensure complete transparency, and maximize the impact of every donation.
              </p>
              <p className="text-lg text-slate-600 dark:text-slate-300 leading-relaxed">
                Our platform enables you to create donation pools with unique Stellar addresses, accept multiple assets (XLM, USDC, or custom tokens), and let idle funds generate yields through DeFi while maintaining complete control over disbursements.
              </p>
            </div>
            <div className="relative h-96 sm:h-full min-h-96">
              <div className="absolute inset-0 bg-gradient-to-br from-blue-500/20 to-cyan-500/20 rounded-3xl blur-3xl"></div>
              <div className="relative bg-gradient-to-br from-blue-500 to-cyan-500 rounded-3xl h-full flex items-center justify-center">
                <div className="text-center text-white">
                  <Heart size={80} className="mx-auto mb-4 opacity-90" />
                  <p className="text-lg font-semibold">Empowering Giving</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Why Stellar Section */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-4xl sm:text-5xl font-bold text-slate-900 dark:text-white mb-4">
              Why Stellar & Soroban?
            </h2>
            <p className="text-xl text-slate-600 dark:text-slate-400">
              We built Nevo on Stellar&apos;s smart contract platform for good reasons
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div className="bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 p-8 rounded-2xl border border-slate-200 dark:border-slate-700">
              <div className="bg-blue-100 dark:bg-blue-900 w-12 h-12 rounded-lg flex items-center justify-center mb-4">
                <Zap className="text-blue-600 dark:text-blue-300" size={24} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                Ultra-Low Costs
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                Stellar transactions cost approximately $0.00001, ensuring that more of your donations reach the intended causes instead of being consumed by fees.
              </p>
            </div>

            <div className="bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 p-8 rounded-2xl border border-slate-200 dark:border-slate-700">
              <div className="bg-cyan-100 dark:bg-cyan-900 w-12 h-12 rounded-lg flex items-center justify-center mb-4">
                <TrendingUp className="text-cyan-600 dark:text-cyan-300" size={24} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                Lightning-Fast
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                Transactions finalize in 3-5 seconds, providing near-instant confirmation for donations and disbursements.
              </p>
            </div>

            <div className="bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 p-8 rounded-2xl border border-slate-200 dark:border-slate-700">
              <div className="bg-green-100 dark:bg-green-900 w-12 h-12 rounded-lg flex items-center justify-center mb-4">
                <Shield className="text-green-600 dark:text-green-300" size={24} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                Battle-Tested
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                Built on Stellar&apos;s proven network infrastructure, which has been securing billions in transactions since 2014.
              </p>
            </div>

            <div className="bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 p-8 rounded-2xl border border-slate-200 dark:border-slate-700">
              <div className="bg-purple-100 dark:bg-purple-900 w-12 h-12 rounded-lg flex items-center justify-center mb-4">
                <Lock className="text-purple-600 dark:text-purple-300" size={24} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                Multi-Asset Support
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                Native support for XLM, USDC, and custom Stellar assets, giving donors flexibility in how they contribute.
              </p>
            </div>

            <div className="bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 p-8 rounded-2xl border border-slate-200 dark:border-slate-700">
              <div className="bg-amber-100 dark:bg-amber-900 w-12 h-12 rounded-lg flex items-center justify-center mb-4">
                <Eye className="text-amber-600 dark:text-amber-300" size={24} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                Smart Contract Security
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                Soroban smart contracts written in Rust provide robust security and complex logic capabilities for fund management.
              </p>
            </div>

            <div className="bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 p-8 rounded-2xl border border-slate-200 dark:border-slate-700">
              <div className="bg-pink-100 dark:bg-pink-900 w-12 h-12 rounded-lg flex items-center justify-center mb-4">
                <TrendingUp className="text-pink-600 dark:text-pink-300" size={24} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                DeFi Integration
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                Seamless access to Stellar&apos;s growing DeFi ecosystem, enabling yield generation on pooled donations.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* What We Offer Section */}
      <section className="py-20 px-4 sm:px-6 lg:px-8 bg-white dark:bg-slate-900/50">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-4xl sm:text-5xl font-bold text-slate-900 dark:text-white mb-4">
              What We Offer
            </h2>
            <p className="text-xl text-slate-600 dark:text-slate-400">
              Everything you need for transparent, efficient charitable giving
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="bg-gradient-to-br from-blue-500/10 to-cyan-500/10 dark:from-blue-500/20 dark:to-cyan-500/20 p-8 rounded-2xl border border-blue-200 dark:border-blue-800">
              <h3 className="text-2xl font-bold text-slate-900 dark:text-white mb-4">
                Transparent Donation Pools
              </h3>
              <p className="text-slate-600 dark:text-slate-300 leading-relaxed">
                Create donation pools with unique Stellar addresses. Each pool is fully transparent, with all transactions recorded immutably on the blockchain. Contributors can verify exactly how their donations are being used.
              </p>
            </div>

            <div className="bg-gradient-to-br from-green-500/10 to-emerald-500/10 dark:from-green-500/20 dark:to-emerald-500/20 p-8 rounded-2xl border border-green-200 dark:border-green-800">
              <h3 className="text-2xl font-bold text-slate-900 dark:text-white mb-4">
                Multi-Asset Support
              </h3>
              <p className="text-slate-600 dark:text-slate-300 leading-relaxed">
                Accept donations in XLM, USDC, or custom Stellar assets. This flexibility allows contributors to donate using their preferred asset, making giving more accessible to everyone.
              </p>
            </div>

            <div className="bg-gradient-to-br from-purple-500/10 to-pink-500/10 dark:from-purple-500/20 dark:to-pink-500/20 p-8 rounded-2xl border border-purple-200 dark:border-purple-800">
              <h3 className="text-2xl font-bold text-slate-900 dark:text-white mb-4">
                DeFi Yield Generation
              </h3>
              <p className="text-slate-600 dark:text-slate-300 leading-relaxed">
                Idle funds in donation pools can generate passive yield through Stellar&apos;s DeFi ecosystem. This means your donations can grow over time, increasing their impact without requiring additional fundraising.
              </p>
            </div>

            <div className="bg-gradient-to-br from-amber-500/10 to-orange-500/10 dark:from-amber-500/20 dark:to-orange-500/20 p-8 rounded-2xl border border-amber-200 dark:border-amber-800">
              <h3 className="text-2xl font-bold text-slate-900 dark:text-white mb-4">
                Complete Control
              </h3>
              <p className="text-slate-600 dark:text-slate-300 leading-relaxed">
                Pool creators maintain complete control over fund disbursements. Smart contracts ensure funds can only be used as intended, while providing full transparency to all contributors.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Values Section */}
      <section className="py-20 px-4 sm:px-6 lg:px-8">
        <div className="max-w-7xl mx-auto">
          <div className="text-center mb-16">
            <h2 className="text-4xl sm:text-5xl font-bold text-slate-900 dark:text-white mb-4">
              Our Values
            </h2>
            <p className="text-xl text-slate-600 dark:text-slate-400">
              The principles that guide everything we do
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            <div className="text-center">
              <div className="bg-gradient-to-br from-blue-500 to-cyan-500 w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4">
                <Eye className="text-white" size={32} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                Transparency
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                Every transaction is publicly verifiable on the blockchain. We believe in complete transparency in charitable giving.
              </p>
            </div>

            <div className="text-center">
              <div className="bg-gradient-to-br from-green-500 to-emerald-500 w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4">
                <Zap className="text-white" size={32} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                Efficiency
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                By minimizing fees and maximizing yield generation, we ensure that more money reaches the causes that need it.
              </p>
            </div>

            <div className="text-center">
              <div className="bg-gradient-to-br from-purple-500 to-pink-500 w-16 h-16 rounded-full flex items-center justify-center mx-auto mb-4">
                <Shield className="text-white" size={32} />
              </div>
              <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
                Security
              </h3>
              <p className="text-slate-600 dark:text-slate-400">
                Built on battle-tested blockchain technology with smart contract security, ensuring your donations are always safe.
              </p>
            </div>
          </div>
        </div>
      </section>

      <Footer />
    </div>
  );
}
