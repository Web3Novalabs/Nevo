import { Lock, TrendingUp, Zap, Eye, CheckCircle, Shield } from "lucide-react";

interface FeatureCardProps {
  icon: React.ReactNode;
  iconBgColor: string;
  title: string;
  description: string;
}

const FeatureCard: React.FC<FeatureCardProps> = ({
  icon,
  iconBgColor,
  title,
  description,
}) => (
  <div className="bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-800 dark:to-slate-900 p-8 rounded-2xl border border-slate-200 dark:border-slate-700 hover:shadow-xl transition-all duration-300 hover:-translate-y-1.5">
    <div
      className={`${iconBgColor} w-12 h-12 rounded-lg flex items-center justify-center mb-4`}
    >
      {icon}
    </div>
    <h3 className="text-xl font-bold text-slate-900 dark:text-white mb-2">
      {title}
    </h3>
    <p className="text-slate-600 dark:text-slate-400">{description}</p>
  </div>
);

export const FeaturesSection = () => {
  const features = [
    {
      icon: <Lock className="text-blue-600 dark:text-blue-300" size={24} />,
      iconBgColor: "bg-blue-100 dark:bg-blue-900",
      title: "Secure Donation Pools",
      description:
        "Smart contracts ensure funds are secure and can only be used as intended. Immutable audit trails on blockchain.",
    },
    {
      icon: (
        <TrendingUp className="text-cyan-600 dark:text-cyan-300" size={24} />
      ),
      iconBgColor: "bg-cyan-100 dark:bg-cyan-900",
      title: "Multi-Asset Support",
      description:
        "Accept XLM, USDC, and custom assets. Flexible donation options for your contributors.",
    },
    {
      icon: <Zap className="text-green-600 dark:text-green-300" size={24} />,
      iconBgColor: "bg-green-100 dark:bg-green-900",
      title: "DeFi Yield Generation",
      description:
        "Generate passive yield on pooled donations. Increase impact without additional fundraising.",
    },
    {
      icon: <Eye className="text-purple-600 dark:text-purple-300" size={24} />,
      iconBgColor: "bg-purple-100 dark:bg-purple-900",
      title: "Transparent Tracking",
      description:
        "Real-time dashboard showing pool balance, contributions, and fund utilization. Full blockchain transparency.",
    },
    {
      icon: (
        <CheckCircle className="text-amber-600 dark:text-amber-300" size={24} />
      ),
      iconBgColor: "bg-amber-100 dark:bg-amber-900",
      title: "Ultra-Low Costs",
      description:
        "Stellar&apos;s efficient network means near-zero fees. More donations reach your cause.",
    },
    {
      icon: <Shield className="text-pink-600 dark:text-pink-300" size={24} />,
      iconBgColor: "bg-pink-100 dark:bg-pink-900",
      title: "Community Trust",
      description:
        "Verified contributors and transparent governance. Build confidence in your cause.",
    },
  ];

  return (
    <section
      id="features"
      className="py-20 px-4 sm:px-6 lg:px-8 bg-white dark:bg-slate-900/50"
    >
      <div className="max-w-7xl mx-auto">
        <div className="text-center mb-16">
          <h2 className="text-4xl sm:text-5xl font-bold text-slate-900 dark:text-white mb-4">
            Powerful Features
          </h2>
          <p className="text-xl text-slate-600 dark:text-slate-400">
            Everything you need for trustworthy collective giving
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
          {features.map((feature, index) => (
            <FeatureCard key={index} {...feature} />
          ))}
        </div>
      </div>
    </section>
  );
};
