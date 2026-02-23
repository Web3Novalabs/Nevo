import { ArrowRight } from "lucide-react";

interface StepProps {
  number: number;
  title: string;
  description: string;
}

const Step: React.FC<StepProps> = ({ number, title, description }) => (
  <div className="relative">
    <div className="bg-white dark:bg-slate-800 p-8 rounded-2xl border border-slate-200 dark:border-slate-700 h-full">
      <div className="absolute -top-5 -left-5 w-12 h-12 bg-[#50C878] rounded-full flex items-center justify-center text-white font-bold text-lg">
        {number}
      </div>
      <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-3 mt-2">
        {title}
      </h3>
      <p className="text-slate-600 dark:text-slate-400">{description}</p>
    </div>
    {number < 4 && (
      <div className="hidden md:block absolute top-1/2 -right-3 w-6 h-6">
        <ArrowRight className="text-blue-500" size={24} />
      </div>
    )}
  </div>
);

export const HowItWorksSection = () => {
  const steps = [
    {
      number: 1,
      title: "Create Pool",
      description:
        "Set up your donation pool with pool name, description, and fund allocation strategy.",
    },
    {
      number: 2,
      title: "Share & Invite",
      description:
        "Invite contributors to your pool. Share via link or QR code for easy onboarding.",
    },
    {
      number: 3,
      title: "Collect Funds",
      description:
        "Contributors donate XLM, USDC, or custom assets. Funds pool securely on Stellar.",
    },
    {
      number: 4,
      title: "Earn & Impact",
      description:
        "Generate yield on pooled funds while maintaining full transparency and control.",
    },
  ];

  return (
    <section id="how-it-works" className="py-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-4xl sm:text-5xl font-bold text-slate-900 dark:text-white mb-4">
            How It Works
          </h2>
          <p className="text-xl text-slate-600 dark:text-slate-400">
            Simple steps to create and manage your donation pool
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-4 gap-6 md:gap-0">
          {steps.map((step) => (
            <Step key={step.number} {...step} />
          ))}
        </div>
      </div>
    </section>
  );
};
