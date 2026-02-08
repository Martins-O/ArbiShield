import { Outlet, NavLink, Link, useLocation } from 'react-router-dom';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { Shield, Activity, AlertCircle, Zap, Home, Github, Twitter, FileText } from 'lucide-react';
import { useNetwork } from 'wagmi';

export default function Dashboard() {
  const location = useLocation();
  const { chain } = useNetwork();

  const navLinks = [
    { to: '/', label: 'Home', icon: Home },
    { to: '/detection-engine', label: 'Detection Engine', icon: Activity },
    { to: '/circuit-breaker', label: 'Circuit Breaker', icon: Zap },
    { to: '/alerts', label: 'Alert Registry', icon: AlertCircle },
  ];

  const isHomePage = location.pathname === '/';

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-slate-800 to-slate-900 flex flex-col">
      {/* Header */}
      <header className="bg-slate-900/80 backdrop-blur-sm border-b border-slate-700/50 sticky top-0 z-50">
        <div className="container mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <Link to="/" className="flex items-center space-x-3 hover:opacity-80 transition-opacity">
              <div className="relative">
                <Shield className="w-10 h-10 text-cyan-400" strokeWidth={2.5} />
                <div className="absolute inset-0 bg-cyan-400/20 blur-xl rounded-full"></div>
              </div>
              <div>
                <h1 className="text-2xl font-bold text-white tracking-tight">ArbiShield</h1>
                <p className="text-xs text-cyan-400/80">Powered by Stylus</p>
              </div>
            </Link>

            <div className="flex items-center space-x-4">
              {/* Network Indicator */}
              {chain && (
                <div className="hidden md:flex items-center space-x-2 px-3 py-2 rounded-lg bg-slate-800/50 border border-slate-700/50">
                  <div className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></div>
                  <span className="text-sm text-slate-300">{chain.name}</span>
                </div>
              )}
              <ConnectButton />
            </div>
          </div>
        </div>
      </header>

      {!isHomePage && (
        <div className="flex flex-1">
          {/* Sidebar Navigation */}
          <nav className="w-64 bg-slate-900/50 backdrop-blur-sm border-r border-slate-700/50">
            <div className="p-4 space-y-2">
              {navLinks.map(({ to, label, icon: Icon }) => (
                <NavLink
                  key={to}
                  to={to}
                  className={({ isActive }) =>
                    `flex items-center space-x-3 px-4 py-3 rounded-lg transition-all ${
                      isActive
                        ? 'bg-gradient-to-r from-cyan-500 to-blue-500 text-white shadow-lg shadow-cyan-500/20'
                        : 'text-slate-300 hover:bg-slate-800/50 hover:text-white'
                    }`
                  }
                >
                  <Icon className="w-5 h-5" />
                  <span className="font-medium">{label}</span>
                </NavLink>
              ))}
            </div>
          </nav>

          {/* Main Content */}
          <main className="flex-1 p-8 overflow-auto">
            <Outlet />
          </main>
        </div>
      )}

      {isHomePage && (
        <main className="flex-1">
          <Outlet />
        </main>
      )}

      {/* Footer */}
      <footer className="bg-slate-900/80 backdrop-blur-sm border-t border-slate-700/50 mt-auto">
        <div className="container mx-auto px-6 py-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
            {/* Brand */}
            <div>
              <div className="flex items-center space-x-2 mb-3">
                <Shield className="w-6 h-6 text-cyan-400" />
                <span className="text-lg font-bold text-white">ArbiShield</span>
              </div>
              <p className="text-sm text-slate-400 leading-relaxed">
                Advanced smart contract security monitoring platform built on Arbitrum Stylus
              </p>
            </div>

            {/* Links */}
            <div>
              <h4 className="text-sm font-semibold text-white mb-3">Resources</h4>
              <ul className="space-y-2">
                <li>
                  <a href="/docs" className="text-sm text-slate-400 hover:text-cyan-400 transition-colors flex items-center space-x-1">
                    <FileText className="w-4 h-4" />
                    <span>Documentation</span>
                  </a>
                </li>
                <li>
                  <a href="https://github.com/Martins-O/ArbiShield" target="_blank" rel="noopener noreferrer" className="text-sm text-slate-400 hover:text-cyan-400 transition-colors flex items-center space-x-1">
                    <Github className="w-4 h-4" />
                    <span>GitHub</span>
                  </a>
                </li>
                <li>
                  <a href="https://docs.arbitrum.io/stylus" target="_blank" rel="noopener noreferrer" className="text-sm text-slate-400 hover:text-cyan-400 transition-colors">
                    Arbitrum Stylus
                  </a>
                </li>
              </ul>
            </div>

            {/* Social */}
            <div>
              <h4 className="text-sm font-semibold text-white mb-3">Connect</h4>
              <div className="flex space-x-3">
                <a href="https://twitter.com" target="_blank" rel="noopener noreferrer" className="p-2 bg-slate-800/50 hover:bg-slate-700/50 rounded-lg transition-colors">
                  <Twitter className="w-5 h-5 text-slate-400 hover:text-cyan-400 transition-colors" />
                </a>
                <a href="https://github.com/Martins-O/ArbiShield" target="_blank" rel="noopener noreferrer" className="p-2 bg-slate-800/50 hover:bg-slate-700/50 rounded-lg transition-colors">
                  <Github className="w-5 h-5 text-slate-400 hover:text-cyan-400 transition-colors" />
                </a>
              </div>
            </div>
          </div>

          <div className="border-t border-slate-700/50 mt-6 pt-6 flex flex-col md:flex-row justify-between items-center text-sm text-slate-400">
            <p>&copy; 2026 ArbiShield. Built for Arbitrum Stylus Hackathon.</p>
            <p className="mt-2 md:mt-0">10-12x more gas efficient than Solidity</p>
          </div>
        </div>
      </footer>
    </div>
  );
}
