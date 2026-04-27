import Link from "next/link";

export const CTASection = () => {
  return (
    <section aria-label="Call to action" className="py-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-4xl mx-auto text-center bg-[linear-gradient(135deg,#50C878,#14B8A6)] rounded-3xl p-12 sm:p-16 text-white">
        <h2 className="text-4xl sm:text-5xl font-bold mb-6">
          Ready to Make Impact?
        </h2>
        <p className="text-lg mb-8 opacity-95">
          Start creating transparent donation pools today. Join the future of
          collective giving.
        </p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link
            href="/dashboard"
            className="bg-white text-black hover:bg-slate-50 px-8 py-3 rounded-lg font-semibold transition-all duration-300 hover:-translate-y-1 active:scale-95 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
            aria-label="Launch the Nevo application"
          >
            Launch Application
          </Link>
          <Link
            href="/explore"
            className="border-2 border-white hover:bg-white/10 text-white px-8 py-3 rounded-lg font-semibold transition focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
            aria-label="Browse donation pools"
          >
            Browse Pools
          </Link>
        </div>
      </div>
    </section>
  );
};
