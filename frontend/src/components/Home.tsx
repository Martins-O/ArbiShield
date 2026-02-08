import { Link } from 'react-router-dom';
import { Shield, Activity, Zap, AlertCircle, ArrowRight, CheckCircle, TrendingDown } from 'lucide-react';

export default function Home() {
  const features = [
    {
      icon: Activity,
      title: 'Detection Engine',
      description: 'Real-time anomaly detection with threshold-based monitoring and severity scoring',
      link: '/detection-engine',
    },
    {
      icon: Zap,
      title: 'Circuit Breaker',
      description: 'Emergency pause mechanism to halt operations when security threats are detected',
      link: '/circuit-breaker',
    },
    {
      icon: AlertCircle,
      title: 'Alert Registry',
      description: 'Comprehensive alert management with priority levels and RBAC access control',
      link: '/alerts',
    },
  ];

  const benefits = [
    '10-12x more gas efficient than Solidity',
    'Built with Arbitrum Stylus (Rust → WASM)',
    'Real-time threat detection and monitoring',
    'Role-based access control (RBAC)',
    'Emergency circuit breaker protection',
    'Comprehensive event logging',
  ];

  return (
    <div className="min-h-screen">
      {/* Hero Section */}
      <section className="relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-cyan-500/10 via-blue-500/5 to-transparent"></div>
        <div className="container mx-auto px-6 py-24 relative">
          <div className="max-w-4xl mx-auto text-center">
            <div className="inline-flex items-center space-x-2 bg-cyan-500/10 border border-cyan-500/20 rounded-full px-4 py-2 mb-6">
              <TrendingDown className="w-4 h-4 text-cyan-400" />
              <span className="text-sm text-cyan-400 font-medium">Reduce gas costs by 90%</span>
            </div>

            <h1 className="text-5xl md:text-6xl font-bold text-white mb-6 leading-tight">
              Advanced Smart Contract
              <span className="block mt-2 bg-gradient-to-r from-cyan-400 to-blue-500 bg-clip-text text-transparent">
                Security Monitoring
              </span>
            </h1>

            <p className="text-xl text-slate-300 mb-8 leading-relaxed max-w-2xl mx-auto">
              Protect your Arbitrum smart contracts with real-time anomaly detection,
              emergency controls, and comprehensive alert management—all built with Stylus for maximum efficiency.
            </p>

            <div className="flex flex-col sm:flex-row items-center justify-center space-y-4 sm:space-y-0 sm:space-x-4">
              <Link
                to="/detection-engine"
                className="group px-8 py-4 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-600 hover:to-blue-600 text-white font-semibold rounded-lg shadow-lg shadow-cyan-500/20 transition-all flex items-center space-x-2"
              >
                <span>Get Started</span>
                <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </Link>
              <a
                href="https://github.com/Martins-O/ArbiShield"
                target="_blank"
                rel="noopener noreferrer"
                className="px-8 py-4 bg-slate-800/50 hover:bg-slate-700/50 text-white font-semibold rounded-lg border border-slate-700/50 transition-all"
              >
                View on GitHub
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 bg-slate-900/50">
        <div className="container mx-auto px-6">
          <div className="text-center mb-16">
            <h2 className="text-3xl md:text-4xl font-bold text-white mb-4">
              Comprehensive Security Suite
            </h2>
            <p className="text-lg text-slate-400 max-w-2xl mx-auto">
              Three powerful modules working together to protect your smart contracts
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8 max-w-6xl mx-auto">
            {features.map((feature) => {
              const Icon = feature.icon;
              return (
                <Link
                  key={feature.title}
                  to={feature.link}
                  className="group p-8 bg-slate-800/30 hover:bg-slate-800/50 border border-slate-700/50 rounded-xl transition-all hover:shadow-xl hover:shadow-cyan-500/10 hover:-translate-y-1"
                >
                  <div className="w-12 h-12 bg-gradient-to-br from-cyan-500 to-blue-500 rounded-lg flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                    <Icon className="w-6 h-6 text-white" />
                  </div>
                  <h3 className="text-xl font-bold text-white mb-3 group-hover:text-cyan-400 transition-colors">
                    {feature.title}
                  </h3>
                  <p className="text-slate-400 leading-relaxed">
                    {feature.description}
                  </p>
                  <div className="mt-4 flex items-center text-cyan-400 font-medium group-hover:gap-2 transition-all">
                    <span>Learn more</span>
                    <ArrowRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
                  </div>
                </Link>
              );
            })}
          </div>
        </div>
      </section>

      {/* Benefits Section */}
      <section className="py-20">
        <div className="container mx-auto px-6">
          <div className="max-w-4xl mx-auto">
            <div className="text-center mb-12">
              <h2 className="text-3xl md:text-4xl font-bold text-white mb-4">
                Why ArbiShield?
              </h2>
              <p className="text-lg text-slate-400">
                Built on Arbitrum Stylus for unmatched performance and efficiency
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {benefits.map((benefit, index) => (
                <div
                  key={index}
                  className="flex items-start space-x-3 p-4 bg-slate-800/30 rounded-lg border border-slate-700/50"
                >
                  <CheckCircle className="w-5 h-5 text-green-400 flex-shrink-0 mt-0.5" />
                  <span className="text-slate-300">{benefit}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 bg-gradient-to-br from-cyan-500/10 to-blue-500/10">
        <div className="container mx-auto px-6">
          <div className="max-w-4xl mx-auto text-center">
            <Shield className="w-16 h-16 text-cyan-400 mx-auto mb-6" />
            <h2 className="text-3xl md:text-4xl font-bold text-white mb-4">
              Ready to Secure Your Contracts?
            </h2>
            <p className="text-lg text-slate-300 mb-8">
              Connect your wallet and start monitoring your smart contracts today
            </p>
            <Link
              to="/detection-engine"
              className="inline-flex items-center space-x-2 px-8 py-4 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-600 hover:to-blue-600 text-white font-semibold rounded-lg shadow-lg shadow-cyan-500/20 transition-all"
            >
              <span>Launch Dashboard</span>
              <ArrowRight className="w-5 h-5" />
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}
