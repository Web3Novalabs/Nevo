export const Footer = () => {
  return (
    <footer className="bg-white dark:bg-slate-950 border-t border-slate-200 dark:border-slate-800 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto grid grid-cols-1 md:grid-cols-4 gap-8 mb-8">
        <div>
          <div className="flex items-center gap-2 mb-4">
            <div className="w-8 h-8 bg-gradient-to-br from-blue-500 to-cyan-500 rounded-lg"></div>
            <span className="text-lg font-bold text-slate-900 dark:text-white">
              Nevo
            </span>
          </div>
          <p className="text-sm text-slate-600 dark:text-slate-400">
            Secure, transparent donation pools on Stellar
          </p>
        </div>
        <div>
          <h4 className="font-semibold text-slate-900 dark:text-white mb-4">
            Product
          </h4>
          <ul className="space-y-2 text-sm text-slate-600 dark:text-slate-400">
            <li>
              <a
                href="#features"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                Features
              </a>
            </li>
            <li>
              <a
                href="#how-it-works"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                How It Works
              </a>
            </li>
            <li>
              <a
                href="#security"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                Security
              </a>
            </li>
          </ul>
        </div>
        <div>
          <h4 className="font-semibold text-slate-900 dark:text-white mb-4">
            Resources
          </h4>
          <ul className="space-y-2 text-sm text-slate-600 dark:text-slate-400">
            <li>
              <a
                href="#"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                Documentation
              </a>
            </li>
            <li>
              <a
                href="#"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                API Reference
              </a>
            </li>
            <li>
              <a
                href="#"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                GitHub
              </a>
            </li>
          </ul>
        </div>
        <div>
          <h4 className="font-semibold text-slate-900 dark:text-white mb-4">
            Legal
          </h4>
          <ul className="space-y-2 text-sm text-slate-600 dark:text-slate-400">
            <li>
              <a
                href="#"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                Privacy
              </a>
            </li>
            <li>
              <a
                href="#"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                Terms
              </a>
            </li>
            <li>
              <a
                href="#"
                className="hover:text-slate-900 dark:hover:text-white transition"
              >
                Contact
              </a>
            </li>
          </ul>
        </div>
      </div>
      <div className="border-t border-slate-200 dark:border-slate-800 pt-8">
        <p className="text-center text-sm text-slate-600 dark:text-slate-400">
          © 2026 Nevo. All rights reserved. Built on Stellar.
        </p>
      </div>
    </footer>
  );
};
