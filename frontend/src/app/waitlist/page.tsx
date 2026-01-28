import { WaitlistSignupForm } from "@/components/waitlist";

export default function WaitlistPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-slate-50 dark:from-slate-950 dark:via-blue-950 dark:to-slate-950 pt-20 pb-12">
      <div className="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8">
        {/* Header */}
        <div className="text-center mb-12">
          <h1 className="text-4xl sm:text-5xl font-bold text-slate-900 dark:text-white mb-4">
            Join Our Waitlist
          </h1>
          <p className="text-lg text-slate-600 dark:text-slate-300 max-w-lg mx-auto">
            Be the first to know when Nevo launches. Get early access to create
            and manage transparent donation pools on the Stellar blockchain.
          </p>
        </div>

        {/* Form Section */}
        <div className="bg-white dark:bg-slate-900 rounded-2xl shadow-xl p-8 sm:p-12 border border-slate-200 dark:border-slate-800">
          <WaitlistSignupForm />
        </div>

        {/* Features */}
        <div className="mt-12 grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="text-center">
            <div className="w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center mx-auto mb-4">
              <span className="text-2xl">🔒</span>
            </div>
            <h3 className="font-semibold text-slate-900 dark:text-white mb-2">
              Secure
            </h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Your data is encrypted and protected
            </p>
          </div>

          <div className="text-center">
            <div className="w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center mx-auto mb-4">
              <span className="text-2xl">⚡</span>
            </div>
            <h3 className="font-semibold text-slate-900 dark:text-white mb-2">
              Fast
            </h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Get instant access to early features
            </p>
          </div>

          <div className="text-center">
            <div className="w-12 h-12 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center mx-auto mb-4">
              <span className="text-2xl">🎁</span>
            </div>
            <h3 className="font-semibold text-slate-900 dark:text-white mb-2">
              Rewards
            </h3>
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Early adopters get exclusive benefits
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
