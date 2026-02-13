import { Outlet, NavLink, Link, useLocation } from 'react-router-dom';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { Shield, Activity, AlertCircle, Zap, Github, Twitter, Layers, ExternalLink } from 'lucide-react';
import { useAccount } from 'wagmi';
import { motion, AnimatePresence } from 'framer-motion';

export default function Dashboard() {
  const location = useLocation();
  const { chain } = useAccount();

  const navLinks = [
    { to: '/detection-engine', label: 'Detection', icon: Activity },
    { to: '/circuit-breaker', label: 'Breaker', icon: Zap },
    { to: '/alerts', label: 'Registry', icon: AlertCircle },
  ];

  // const isHomePage = location.pathname === '/';

  return (
    <div className="min-h-screen bg-obsidian flex flex-col selection:bg-cyan-500/30">
      {/* Global Navigation */}
      <header className="fixed top-0 left-0 right-0 z-50 px-6 py-4">
        <div className="max-w-7xl mx-auto">
          <div className="glass px-6 py-3 rounded-2xl flex items-center justify-between shadow-2xl shadow-black/50 border-white/5">
            <Link to="/" className="flex items-center space-x-3 group translate-z-0">
              <div className="relative">
                <Shield className="w-8 h-8 text-cyan-400 group-hover:scale-110 transition-transform" strokeWidth={2.5} />
                <div className="absolute inset-0 bg-cyan-400/20 blur-xl rounded-full opacity-0 group-hover:opacity-100 transition-opacity"></div>
              </div>
              <div className="hidden sm:block">
                <h1 className="text-xl font-extrabold text-white tracking-tighter leading-none">ArbiShield</h1>
                <p className="text-[10px] text-cyan-400/80 font-bold uppercase tracking-widest mt-1">Arbitrum Protocol</p>
              </div>
            </Link>

            {/* Desktop Nav */}
            <nav className="hidden md:flex items-center bg-white/5 rounded-xl p-1 border border-white/5">
              {navLinks.map(({ to, label, icon: Icon }) => (
                <NavLink
                  key={to}
                  to={to}
                  className={({ isActive }) =>
                    `flex items-center space-x-2 px-4 py-2 rounded-lg transition-all duration-300 font-semibold text-sm ${isActive
                      ? 'bg-cyan-500 text-obsidian shadow-lg shadow-cyan-500/40'
                      : 'text-slate-400 hover:text-white hover:bg-white/5'
                    }`
                  }
                >
                  <Icon className="w-4 h-4" />
                  <span>{label}</span>
                </NavLink>
              ))}
            </nav>

            <div className="flex items-center space-x-4">
              {chain && (
                <div className="hidden lg:flex items-center space-x-2 px-3 py-1.5 rounded-lg bg-emerald-500/10 border border-emerald-500/20">
                  <div className="w-2 h-2 bg-emerald-400 rounded-full animate-pulse"></div>
                  <span className="text-[11px] font-bold text-emerald-400 uppercase tracking-wider">{chain.name}</span>
                </div>
              )}
              <ConnectButton accountStatus="avatar" chainStatus="icon" showBalance={false} />
            </div>
          </div>
        </div>
      </header>

      <main className="flex-1 flex flex-col pt-24">
        <AnimatePresence mode="wait">
          <motion.div
            key={location.pathname}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.3 }}
            className="flex-1"
          >
            <Outlet />
          </motion.div>
        </AnimatePresence>
      </main>

      {/* Footer */}
      <footer className="bg-slate-900/40 backdrop-blur-md border-t border-white/5 py-12 mt-20">
        <div className="container mx-auto px-6">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-12 mb-12">
            <div className="col-span-1 md:col-span-2">
              <div className="flex items-center space-x-2 mb-6">
                <Shield className="w-8 h-8 text-cyan-400" />
                <span className="text-2xl font-black text-white tracking-tighter">ArbiShield</span>
              </div>
              <p className="text-slate-400 max-w-sm leading-relaxed mb-6">
                Redefining smart contract security with the raw power of Arbitrum Layer 2.
                Real-time monitoring and emergency response at a fraction of the cost.
              </p>
              <div className="flex space-x-4">
                <a href="#" className="p-2 bg-white/5 hover:bg-white/10 rounded-lg transition-colors border border-white/5">
                  <Twitter className="w-5 h-5 text-slate-300" />
                </a>
                <a href="https://github.com/Martins-O/ArbiShield" target="_blank" rel="noreferrer" className="p-2 bg-white/5 hover:bg-white/10 rounded-lg transition-colors border border-white/5">
                  <Github className="w-5 h-5 text-slate-300" />
                </a>
              </div>
            </div>

            <div>
              <h4 className="text-white font-bold mb-6 flex items-center gap-2">
                <Layers className="w-4 h-4 text-cyan-400" />
                Platform
              </h4>
              <ul className="space-y-4">
                {navLinks.map(link => (
                  <li key={link.to}>
                    <Link to={link.to} className="text-slate-400 hover:text-cyan-400 transition-colors text-sm font-medium">
                      {link.label} Module
                    </Link>
                  </li>
                ))}
              </ul>
            </div>

            <div>
              <h4 className="text-white font-bold mb-6 flex items-center gap-2">
                <ExternalLink className="w-4 h-4 text-purple-400" />
                Resources
              </h4>
              <ul className="space-y-4">
                <li><a href="#" className="text-slate-400 hover:text-cyan-400 transition-colors text-sm font-medium">Documentation</a></li>
                <li><a href="#" className="text-slate-400 hover:text-cyan-400 transition-colors text-sm font-medium">Security Guide</a></li>
                <li><a href="#" className="text-slate-400 hover:text-cyan-400 transition-colors text-sm font-medium">Security Policy</a></li>
              </ul>
            </div>
          </div>

          <div className="pt-8 border-t border-white/5 flex flex-col md:flex-row justify-between items-center gap-4">
            <p className="text-xs text-slate-500 font-medium">
              &copy; {new Date().getFullYear()} ArbiShield Protocol. All rights reserved.
            </p>
            <div className="flex items-center gap-6">
              <div className="flex items-center gap-2">
                <div className="w-1.5 h-1.5 bg-emerald-500 rounded-full"></div>
                <span className="text-[10px] text-slate-400 font-bold uppercase tracking-widest">Testnet Live</span>
              </div>
              <p className="text-[10px] text-cyan-400/60 font-bold uppercase tracking-widest">Powered by Arbitrum L2</p>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}
