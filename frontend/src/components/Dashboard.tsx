import { Outlet, NavLink } from 'react-router-dom';
import { ConnectButton } from '@rainbow-me/rainbowkit';
import { Shield, Activity, AlertCircle, Zap } from 'lucide-react';

export default function Dashboard() {
  const navLinks = [
    { to: '/detection-engine', label: 'Detection Engine', icon: Activity },
    { to: '/circuit-breaker', label: 'Circuit Breaker', icon: Zap },
    { to: '/alerts', label: 'Alert Registry', icon: AlertCircle },
  ];

  return (
    <div className="min-h-screen bg-slate-900">
      {/* Header */}
      <header className="bg-slate-800 border-b border-slate-700">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <Shield className="w-8 h-8 text-primary-500" />
              <div>
                <h1 className="text-2xl font-bold text-white">ArbiShield</h1>
                <p className="text-sm text-slate-400">Smart Contract Security Monitoring</p>
              </div>
            </div>
            <ConnectButton />
          </div>
        </div>
      </header>

      <div className="flex">
        {/* Sidebar Navigation */}
        <nav className="w-64 bg-slate-800 border-r border-slate-700 min-h-[calc(100vh-73px)]">
          <div className="p-4 space-y-2">
            {navLinks.map(({ to, label, icon: Icon }) => (
              <NavLink
                key={to}
                to={to}
                className={({ isActive }) =>
                  `flex items-center space-x-3 px-4 py-3 rounded-lg transition-colors ${
                    isActive
                      ? 'bg-primary-600 text-white'
                      : 'text-slate-300 hover:bg-slate-700 hover:text-white'
                  }`
                }
              >
                <Icon className="w-5 h-5" />
                <span className="font-medium">{label}</span>
              </NavLink>
            ))}
          </div>

          {/* Network Info */}
          <div className="absolute bottom-4 left-4 right-4">
            <div className="card p-3">
              <div className="flex items-center space-x-2">
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                <span className="text-sm text-slate-400">Arbitrum Sepolia</span>
              </div>
            </div>
          </div>
        </nav>

        {/* Main Content */}
        <main className="flex-1 p-8">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
