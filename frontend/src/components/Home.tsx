import { Link } from 'react-router-dom';
import { motion } from 'framer-motion';
import { Shield, Activity, Zap, AlertCircle, ArrowRight, TrendingDown, Lock, Cpu, Globe } from 'lucide-react';

export default function Home() {
  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.2,
      },
    },
  };

  const itemVariants = {
    hidden: { y: 20, opacity: 0 },
    visible: {
      y: 0,
      opacity: 1,
      transition: {
        duration: 0.6,
        ease: [0.22, 1, 0.36, 1],
      } as const,
    },
  };

  const features = [
    {
      icon: Activity,
      title: 'Detection Engine',
      description: 'Real-time anomaly detection with threshold-based monitoring and adaptive severity scoring.',
      link: '/detection-engine',
      color: 'cyan',
    },
    {
      icon: Zap,
      title: 'Circuit Breaker',
      description: 'Automated emergency pause mechanism to halt operations the moment a threat is validated.',
      link: '/circuit-breaker',
      color: 'purple',
    },
    {
      icon: AlertCircle,
      title: 'Alert Registry',
      description: 'Advanced incident management with priority-based routing and full audit logging.',
      link: '/alerts',
      color: 'blue',
    },
  ];

  const stats = [
    { label: 'Efficiency Boost', value: '12x', sub: 'vs Solidity' },
    { label: 'Security Response', value: '< 1s', sub: 'Real-time' },
    { label: 'Gas Reduction', value: '90%', sub: 'Avg. Saving' },
    { label: 'Architecture', value: 'L2 Native', sub: 'Performance-Optimized' },
  ];

  return (
    <div className="min-h-screen bg-gradient-mesh">
      {/* Hero Section */}
      <section className="relative pt-32 pb-20 overflow-hidden">
        <div className="container mx-auto px-6 relative z-10">
          <motion.div
            initial="hidden"
            animate="visible"
            variants={containerVariants}
            className="max-w-5xl mx-auto text-center"
          >
            <motion.div
              variants={itemVariants}
              className="inline-flex items-center space-x-2 px-3 py-1 rounded-full bg-cyan-500/10 border border-cyan-500/20 mb-8"
            >
              <TrendingDown className="w-4 h-4 text-cyan-400" />
              <span className="text-xs font-bold text-cyan-400 uppercase tracking-widest">Efficiency Redefined</span>
            </motion.div>

            <motion.h1
              variants={itemVariants}
              className="text-6xl md:text-8xl font-extrabold text-white mb-8 tracking-tighter leading-none"
            >
              Secure Your Protocol <br />
              <span className="bg-gradient-to-r from-cyan-400 via-blue-500 to-purple-600 bg-clip-text text-transparent">
                With Intelligence
              </span>
            </motion.h1>

            <motion.p
              variants={itemVariants}
              className="text-xl text-slate-400 mb-12 max-w-2xl mx-auto leading-relaxed"
            >
              ArbiShield delivers enterprise-grade security monitoring for Arbitrum Smart Contracts.
              Real-time detection, instant response, and surgical precision.
            </motion.p>

            <motion.div
              variants={itemVariants}
              className="flex flex-col sm:flex-row items-center justify-center gap-6"
            >
              <Link to="/detection-engine" className="btn-premium group min-w-[200px]">
                <span>Launch App</span>
                <ArrowRight className="w-5 h-5 group-hover:translate-x-1 transition-transform" />
              </Link>
              <a
                href="https://github.com/Martins-O/ArbiShield"
                target="_blank"
                rel="noreferrer"
                className="btn-outline-premium min-w-[200px]"
              >
                Documentation
              </a>
            </motion.div>
          </motion.div>
        </div>

        {/* Abstract Background Elements */}
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-full h-full -z-0 pointer-events-none">
          <div className="absolute top-1/4 left-1/4 w-64 h-64 bg-cyan-500/10 blur-[120px] rounded-full"></div>
          <div className="absolute bottom-1/4 right-1/4 w-96 h-96 bg-purple-500/10 blur-[150px] rounded-full"></div>
        </div>
      </section>

      {/* Stats Section */}
      <section className="py-20 border-y border-white/5 bg-slate-900/20">
        <div className="container mx-auto px-6">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-8">
            {stats.map((stat, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ delay: i * 0.1 }}
                className="text-center"
              >
                <div className="text-4xl font-bold text-white mb-1">{stat.value}</div>
                <div className="text-sm font-semibold text-cyan-400 uppercase tracking-wider">{stat.label}</div>
                <div className="text-xs text-slate-500 mt-1">{stat.sub}</div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Features Grid */}
      <section className="py-32">
        <div className="container mx-auto px-6">
          <div className="text-center mb-20">
            <h2 className="text-4xl font-bold text-white mb-4">Core Security Infrastructure</h2>
            <p className="text-slate-400">Integrated modules designed for high-performance defense</p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {features.map((feature, i) => {
              const Icon = feature.icon;
              return (
                <motion.div
                  key={i}
                  initial={{ opacity: 0, scale: 0.95 }}
                  whileInView={{ opacity: 1, scale: 1 }}
                  viewport={{ once: true }}
                  transition={{ delay: i * 0.2 }}
                  className="glass-card group relative"
                >
                  <div className={`w-14 h-14 rounded-2xl flex items-center justify-center mb-6 bg-slate-800 border border-white/10 group-hover:border-${feature.color}-500/50 transition-colors`}>
                    <Icon className={`w-7 h-7 text-${feature.color}-400`} />
                  </div>
                  <h3 className="text-2xl font-bold text-white mb-4">{feature.title}</h3>
                  <p className="text-slate-400 mb-8 leading-relaxed">{feature.description}</p>
                  <Link
                    to={feature.link}
                    className="flex items-center gap-2 text-cyan-400 font-bold group-hover:gap-4 transition-all"
                  >
                    Explore Module <ArrowRight className="w-4 h-4" />
                  </Link>

                  {/* Decorative background glow on hover */}
                  <div className={`absolute -inset-1 bg-gradient-to-r from-${feature.color}-500 to-blue-500 rounded-2xl opacity-0 group-hover:opacity-5 blur-2xl transition-opacity`}></div>
                </motion.div>
              );
            })}
          </div>
        </div>
      </section>

      {/* Security Architecture */}
      <section className="py-32 relative overflow-hidden">
        <div className="container mx-auto px-6">
          <div className="glass-card flex flex-col md:flex-row items-center gap-16 p-12 overflow-hidden relative">
            <div className="absolute top-0 right-0 w-1/2 h-full bg-gradient-to-l from-cyan-500/5 to-transparent pointer-events-none"></div>

            <div className="w-full md:w-1/2">
              <h2 className="text-4xl font-bold text-white mb-8">Next-Gen Performance <br />on Arbitrum</h2>
              <ul className="space-y-6">
                {[
                  { icon: Cpu, text: 'Native L2 execution for near-native speed', color: 'cyan' },
                  { icon: Lock, text: 'Advanced memory safety and security', color: 'purple' },
                  { icon: Globe, text: 'Massive reduction in compute resources and fees', color: 'blue' },
                ].map((item, i) => (
                  <li key={i} className="flex items-center gap-4">
                    <div className="p-2 rounded-lg bg-slate-800/80 border border-white/10">
                      <item.icon className={`w-5 h-5 text-${item.color}-400`} />
                    </div>
                    <span className="text-lg text-slate-300">{item.text}</span>
                  </li>
                ))}
              </ul>
            </div>

            <div className="w-full md:w-1/2 relative">
              <div className="aspect-square rounded-full border border-white/10 flex items-center justify-center relative p-8">
                <div className="absolute inset-0 bg-cyan-500/5 blur-3xl animate-pulse"></div>
                <div className="w-full h-full rounded-full border border-white/5 flex items-center justify-center p-8">
                  <Shield className="w-1/2 h-1/2 text-cyan-400 drop-shadow-[0_0_15px_rgba(34,211,238,0.4)]" />
                </div>

                {/* Orbiting elements */}
                <motion.div
                  animate={{ rotate: 360 }}
                  transition={{ duration: 20, repeat: Infinity, ease: 'linear' }}
                  className="absolute inset-0"
                >
                  <div className="absolute top-0 left-1/2 -translate-x-1/2 -translate-y-1/2 w-4 h-4 bg-purple-500 rounded-full shadow-[0_0_10px_#a855f7]"></div>
                  <div className="absolute bottom-0 left-1/2 -translate-x-1/2 translate-y-1/2 w-4 h-4 bg-cyan-500 rounded-full shadow-[0_0_10px_#22d3ee]"></div>
                </motion.div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-32">
        <div className="container mx-auto px-6 text-center">
          <motion.div
            initial={{ opacity: 0, y: 30 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            className="max-w-4xl mx-auto"
          >
            <h2 className="text-5xl font-extrabold text-white mb-8">Ready to Bulletproof Your Smart Contracts?</h2>
            <p className="text-xl text-slate-400 mb-12">Join the next generation of secure DeFi protocols on Arbitrum.</p>
            <div className="flex justify-center gap-6">
              <Link to="/detection-engine" className="btn-premium px-12 py-4">
                Deploy Security Node
              </Link>
            </div>
          </motion.div>
        </div>
      </section>
    </div>
  );
}
