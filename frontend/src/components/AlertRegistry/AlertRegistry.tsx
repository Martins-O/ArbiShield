import { useState } from 'react';
import { useAccount, useReadContract } from 'wagmi';
import { AlertCircle, ShieldAlert, ListFilter, Activity, RefreshCw } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { CONTRACT_ADDRESSES, Alert } from '../../types/contracts';
import { AlertRegistryABI } from '../../config/abis';
import AlertCard from './AlertCard';
import AlertStats from './AlertStats';
import FilterPanel from './FilterPanel';

export default function AlertRegistry() {
  const { isConnected } = useAccount();
  const [selectedPriority, setSelectedPriority] = useState<number | null>(null);
  const [showAcknowledged, setShowAcknowledged] = useState(false);

  // Read system stats
  const { data: stats } = useReadContract({
    address: CONTRACT_ADDRESSES.AlertRegistry,
    abi: AlertRegistryABI,
    functionName: 'getSystemStats',
  });

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const statsArr = stats as any;
  const totalAlerts = statsArr ? BigInt(statsArr[0] || 0) : 0n;
  const openAlerts = statsArr ? BigInt(statsArr[1] || 0) : 0n;
  const resolvedAlerts = statsArr ? BigInt(statsArr[2] || 0) : 0n;

  const alerts: Alert[] = [];

  // Filter alerts
  const filteredAlerts = alerts.filter((alert) => {
    if (selectedPriority !== null && alert.priority !== selectedPriority) {
      return false;
    }
    if (!showAcknowledged && alert.acknowledged) {
      return false;
    }
    return true;
  });

  // Sort by priority (critical first) and timestamp (recent first)
  const sortedAlerts = [...filteredAlerts].sort((a, b) => {
    if (a.priority !== b.priority) {
      return b.priority - a.priority;
    }
    return Number(b.timestamp - a.timestamp);
  });

  if (!isConnected) {
    return (
      <div className="container mx-auto px-6 py-20">
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          className="flex flex-col items-center justify-center glass-card border-dashed py-20 max-w-2xl mx-auto"
        >
          <div className="w-20 h-20 bg-slate-800/50 rounded-full flex items-center justify-center mb-6">
            <ShieldAlert className="w-10 h-10 text-slate-600" />
          </div>
          <h2 className="text-3xl font-black text-white mb-2 tracking-tight">Security Log Offline</h2>
          <p className="text-slate-400 text-center px-12">Connect your secure wallet to view real-time protocol alerts and the global security audit log.</p>
        </motion.div>
      </div>
    );
  }

  return (
    <div className="container mx-auto px-6 py-12 space-y-12">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-6">
        <div>
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            className="flex items-center gap-3 mb-2"
          >
            <div className="p-2 bg-red-500/10 rounded-lg">
              <AlertCircle className="w-6 h-6 text-red-400" />
            </div>
            <span className="text-xs font-black text-red-400 uppercase tracking-widest">Security Audit Log</span>
          </motion.div>
          <motion.h1
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.1 }}
            className="text-4xl md:text-5xl font-black text-white tracking-tighter"
          >
            Alert Registry
          </motion.h1>
        </div>
        <div className="flex items-center gap-4">
          <button className="p-3 bg-slate-900/50 rounded-xl border border-white/5 hover:bg-slate-800 transition-colors">
            <RefreshCw className="w-5 h-5 text-slate-400" />
          </button>
        </div>
      </div>

      {/* Stats */}
      <AlertStats
        total={totalAlerts}
        open={openAlerts}
        resolved={resolvedAlerts}
      />

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-12">
        {/* Filters Panel */}
        <motion.div
          initial={{ opacity: 0, x: -20 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ delay: 0.2 }}
          className="lg:col-span-4 space-y-6"
        >
          <div className="glass-card">
            <div className="flex items-center justify-between mb-8">
              <h3 className="text-lg font-black text-white flex items-center gap-2">
                <ListFilter className="w-5 h-5 text-cyan-400" />
                Filter Engine
              </h3>
              <button
                onClick={() => {
                  setSelectedPriority(null);
                  setShowAcknowledged(false);
                }}
                className="text-[10px] font-black text-cyan-400 uppercase tracking-widest hover:text-cyan-300 transition-colors"
              >
                Reset All
              </button>
            </div>

            <FilterPanel
              selectedPriority={selectedPriority}
              setSelectedPriority={setSelectedPriority}
              showAcknowledged={showAcknowledged}
              setShowAcknowledged={setShowAcknowledged}
            />
          </div>

          <div className="p-4 bg-purple-500/5 rounded-2xl border border-purple-500/10">
            <div className="flex items-center gap-2 mb-2">
              <Activity className="w-4 h-4 text-purple-400" />
              <span className="text-[10px] font-black text-purple-400 uppercase tracking-widest">Live Syncing</span>
            </div>
            <p className="text-[10px] text-slate-500 font-medium">Monitoring Arbitrum RPC for new incident events. Latency: <span className="text-white">~400ms</span></p>
          </div>
        </motion.div>

        {/* Alerts List */}
        <div className="lg:col-span-8 space-y-6">
          <div className="flex items-center justify-between">
            <h3 className="text-xl font-black text-white">
              Detected Events <span className="text-slate-500 ml-2 font-normal">({sortedAlerts.length})</span>
            </h3>
          </div>

          <AnimatePresence mode="popLayout">
            {sortedAlerts.length === 0 ? (
              <motion.div
                key="empty"
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, scale: 0.95 }}
                className="glass-card border-dashed text-center py-24"
              >
                <div className="w-20 h-20 bg-slate-900/50 rounded-full flex items-center justify-center mx-auto mb-6">
                  <ShieldAlert className="w-10 h-10 text-slate-700" />
                </div>
                <h4 className="text-xl font-black text-white mb-2">No active threats detected</h4>
                <p className="text-slate-500 text-sm max-w-sm mx-auto leading-relaxed">
                  {alerts.length === 0
                    ? "Protocol is operating within safety parameters. Alerts will be recorded here if anomalies are detected by the Stylus backend."
                    : "The refined filter query returned zero results. Adjust criteria to view more incidents."}
                </p>
              </motion.div>
            ) : (
              <div className="space-y-4">
                {sortedAlerts.map((alert, i) => (
                  <motion.div
                    key={alert.id.toString()}
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: i * 0.05 }}
                  >
                    <AlertCard alert={alert} />
                  </motion.div>
                ))}
              </div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
