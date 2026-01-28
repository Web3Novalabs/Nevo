export const CTASection = () => {
  return (
    <section className="py-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-4xl mx-auto text-center bg-gradient-to-br from-blue-500 to-cyan-500 rounded-3xl p-12 sm:p-16 text-white">
        <h2 className="text-4xl sm:text-5xl font-bold mb-6">
          Ready to Make Impact?
        </h2>
        <p className="text-lg mb-8 opacity-95">
          Start creating transparent donation pools today. Join the future of
          collective giving.
        </p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <button className="bg-white text-blue-600 hover:bg-slate-50 px-8 py-3 rounded-lg font-semibold transition transform hover:scale-105">
            Launch Application
          </button>
          <button className="border-2 border-white hover:bg-white/10 text-white px-8 py-3 rounded-lg font-semibold transition">
            View Documentation
          </button>
        </div>
      </div>
    </section>
  );
};
