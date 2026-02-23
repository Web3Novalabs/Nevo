import { CheckCircle, Lock, Shield, Eye, Zap } from "lucide-react";

interface TrustIndicatorProps {
  icon: React.ReactNode;
  bgColor: string;
  value: string;
  label: string;
}

const TrustIndicator: React.FC<TrustIndicatorProps> = ({
  icon,
  bgColor,
  value,
  label,
}) => (
  <div className={`${bgColor} p-8 rounded-2xl border`}>
    {icon}
    <div className="text-3xl font-bold text-slate-900 dark:text-white mb-2">
      {value}
    </div>
    <p className="text-sm text-slate-600 dark:text-slate-400">{label}</p>
  </div>
);

interface SecurityFeatureProps {
  icon: React.ReactNode;
  title: string;
  description: string;
}

const SecurityFeature: React.FC<SecurityFeatureProps> = ({
  icon,
  title,
  description,
}) => (
  <div className="flex gap-4 items-start">
    {icon}
    <div>
      <h4 className="font-semibold text-slate-900 dark:text-white">{title}</h4>
      <p className="text-slate-600 dark:text-slate-400">{description}</p>
    </div>
  </div>
);

export const SecuritySection = () => {
  return (
    <section
      id="security"
      className="py-20 px-4 sm:px-6 lg:px-8 bg-white dark:bg-slate-900/50"
    >
      <div className="max-w-7xl mx-auto">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
          <div>
            <h2 className="text-4xl sm:text-5xl font-bold text-slate-900 dark:text-white mb-6">
              Trust &amp; Security Built In
            </h2>
            <p className="text-lg text-slate-600 dark:text-slate-400 mb-8">
              Your donations are protected by industry-leading security measures
              and blockchain transparency.
            </p>

            <div className="space-y-4">
              <SecurityFeature
                icon={
                  <CheckCircle
                    className="text-green-500 flex-shrink-0 mt-1"
                    size={24}
                  />
                }
                title="Stellar Blockchain"
                description="All transactions recorded immutably on one of the most trusted blockchains"
              />
              <SecurityFeature
                icon={
                  <CheckCircle
                    className="text-green-500 flex-shrink-0 mt-1"
                    size={24}
                  />
                }
                title="Smart Contract Audited"
                description="All contracts are audited and open-source for community verification"
              />
              <SecurityFeature
                icon={
                  <CheckCircle
                    className="text-green-500 flex-shrink-0 mt-1"
                    size={24}
                  />
                }
                title="Multi-Sig Protection"
                description="Critical operations require multiple signatures for enhanced security"
              />
              <SecurityFeature
                icon={
                  <CheckCircle
                    className="text-green-500 flex-shrink-0 mt-1"
                    size={24}
                  />
                }
                title="Real-Time Monitoring"
                description="24/7 monitoring for suspicious activity and anomalies"
              />
            </div>
          </div>

          <div className="grid grid-cols-2 gap-6">
            <TrustIndicator
              icon={
                <Lock
                  className="text-blue-600 dark:text-blue-300 mb-4"
                  size={32}
                />
              }
              bgColor="bg-gradient-to-br from-blue-500/10 to-cyan-500/10 dark:from-blue-500/20 dark:to-cyan-500/20 border-blue-200 dark:border-blue-800"
              value="256-bit"
              label="Encryption Standard"
            />
            <TrustIndicator
              icon={
                <Shield
                  className="text-green-600 dark:text-green-300 mb-4"
                  size={32}
                />
              }
              bgColor="bg-gradient-to-br from-green-500/10 to-emerald-500/10 dark:from-green-500/20 dark:to-emerald-500/20 border-green-200 dark:border-green-800"
              value="0"
              label="Breaches Ever"
            />
            <TrustIndicator
              icon={
                <Eye
                  className="text-purple-600 dark:text-purple-300 mb-4"
                  size={32}
                />
              }
              bgColor="bg-gradient-to-br from-purple-500/10 to-pink-500/10 dark:from-purple-500/20 dark:to-pink-500/20 border-purple-200 dark:border-purple-800"
              value="100%"
              label="Transparent"
            />
            <TrustIndicator
              icon={
                <Zap
                  className="text-amber-600 dark:text-amber-300 mb-4"
                  size={32}
                />
              }
              bgColor="bg-gradient-to-br from-amber-500/10 to-orange-500/10 dark:from-amber-500/20 dark:to-orange-500/20 border-amber-200 dark:border-amber-800"
              value="&lt;3s"
              label="Load Time"
            />
          </div>
        </div>
      </div>
    </section>
  );
};
