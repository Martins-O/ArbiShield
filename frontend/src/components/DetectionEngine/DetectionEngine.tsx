import { useState } from 'react';
import { useAccount, useReadContract, useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { Activity, Plus, TrendingUp, AlertTriangle, ShieldCheck, Zap } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { CONTRACT_ADDRESSES } from '../../types/contracts';
import { DetectionEngineABI } from '../../config/abis';
import MetricCard from './MetricCard';
import RegisterMetricForm from './RegisterMetricForm';

export default function DetectionEngine() {
  const { address, isConnected } = useAccount();
  const [showRegisterForm, setShowRegisterForm] = useState(false);
  const [metricId, setMetricId] = useState('');
  const [value, setValue] = useState('');

  // Read owner
  const { data: owner } = useReadContract({
    address: CONTRACT_ADDRESSES.DetectionEngine,
    abi: DetectionEngineABI,
    functionName: 'owner',
  });

  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming } = useWaitForTransactionReceipt({ hash });

  const isOwner = address === owner;

  const handleReportMetric = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!metricId || !value) return;

    try {
      writeContract({
        address: CONTRACT_ADDRESSES.DetectionEngine,
        abi: DetectionEngineABI,
        functionName: 'updateMetric',
        args: [BigInt(metricId), BigInt(value)],
      });
      setMetricId('');
      setValue('');
    } catch (error) {
      console.error('Error reporting metric:', error);
    }
  };

  const metrics: Array<{ id: bigint; threshold: bigint; currentValue: bigint }> = [];

  if (!isConnected) {
    return (
      <div className="container mx-auto px-6 py-20">
        <motion.div
          initial={{ opacity: 0, scale: 0.9 }}
          animate={{ opacity: 1, scale: 1 }}
          className="flex flex-col items-center justify-center glass-card border-dashed py-20 max-w-2xl mx-auto"
        >
          <div className="w-20 h-20 bg-slate-800/50 rounded-full flex items-center justify-center mb-6">
            <Zap className="w-10 h-10 text-slate-600" />
          </div>
          <h2 className="text-3xl font-black text-white mb-2 tracking-tight">Intelligence Offline</h2>
          <p className="text-slate-400 text-center px-12">Connect your secure wallet to access the ArbiShield Detection Engine and monitor protocol metrics in real-time.</p>
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
            <div className="p-2 bg-cyan-500/10 rounded-lg">
              <Activity className="w-6 h-6 text-cyan-400" />
            </div>
            <span className="text-xs font-black text-cyan-400 uppercase tracking-widest">Analytics Dashboard</span>
          </motion.div>
          <motion.h1
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.1 }}
            className="text-4xl md:text-5xl font-black text-white tracking-tighter"
          >
            Detection Engine
          </motion.h1>
        </div>
        {isOwner && (
          <motion.button
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            whileHover={{ scale: 1.05 }}
            whileTap={{ scale: 0.95 }}
            onClick={() => setShowRegisterForm(true)}
            className="btn-premium flex items-center space-x-2 px-8 py-4"
          >
            <Plus className="w-5 h-5" />
            <span>Register New Metric</span>
          </motion.button>
        )}
      </div>

      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {[
          { label: 'Network Metrics', val: metrics.length, icon: Activity, color: 'cyan' },
          { label: 'Live Anomalies', val: metrics.filter(m => m.currentValue > m.threshold).length, icon: AlertTriangle, color: 'red' },
          { label: 'System Health', val: metrics.length > 0 ? '99.9%' : 'N/A', icon: ShieldCheck, color: 'emerald' },
        ].map((stat, i) => (
          <motion.div
            key={stat.label}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.1 }}
            className="glass-card relative overflow-hidden group"
          >
            <div className="flex items-center justify-between relative z-10">
              <div>
                <p className="text-slate-500 text-[10px] font-black uppercase tracking-widest mb-1">{stat.label}</p>
                <p className={`text-3xl font-black text-white`}>{stat.val}</p>
              </div>
              <div className={`p-3 bg-${stat.color}-500/10 rounded-xl`}>
                <stat.icon className={`w-6 h-6 text-${stat.color}-400`} />
              </div>
            </div>
            <div className={`absolute -bottom-2 -right-2 w-16 h-16 bg-${stat.color}-500/5 blur-2xl rounded-full group-hover:scale-150 transition-transform`}></div>
          </motion.div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-12">
        {/* Report Metric Form */}
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
          className="lg:col-span-5"
        >
          <div className="glass-card h-full">
            <h3 className="text-xl font-black text-white mb-6 flex items-center gap-2">
              <TrendingUp className="w-5 h-5 text-cyan-400" />
              Ingest Data
            </h3>
            <form onSubmit={handleReportMetric} className="space-y-6">
              <div className="space-y-4">
                <div>
                  <label className="block text-[10px] font-black text-slate-500 uppercase tracking-widest mb-2">
                    Metric Identifier
                  </label>
                  <input
                    type="number"
                    value={metricId}
                    onChange={(e) => setMetricId(e.target.value)}
                    className="input-premium w-full"
                    placeholder="ID (e.g., 1)"
                    min="0"
                  />
                </div>
                <div>
                  <label className="block text-[10px] font-black text-slate-500 uppercase tracking-widest mb-2">
                    Observation Value
                  </label>
                  <input
                    type="number"
                    value={value}
                    onChange={(e) => setValue(e.target.value)}
                    className="input-premium w-full"
                    placeholder="Raw Value"
                    min="0"
                  />
                </div>
              </div>
              <button
                type="submit"
                disabled={isPending || isConfirming || !metricId || !value}
                className="btn-premium w-full !rounded-xl"
              >
                {isPending || isConfirming ? 'Processing Transaction...' : 'Submit Observation'}
              </button>
            </form>
          </div>
        </motion.div>

        {/* Metrics List */}
        <div className="lg:col-span-7">
          <h3 className="text-xl font-black text-white mb-6 flex items-center gap-2">
            <Activity className="w-5 h-5 text-purple-400" />
            Metric Registry
          </h3>
          <div className="space-y-6">
            {metrics.length > 0 ? (
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                {metrics.map((metric) => (
                  <MetricCard key={metric.id.toString()} metric={metric} />
                ))}
              </div>
            ) : (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="glass-card border-dashed text-center py-20"
              >
                <div className="w-16 h-16 bg-slate-800/30 rounded-full flex items-center justify-center mx-auto mb-4">
                  <Activity className="w-8 h-8 text-slate-600" />
                </div>
                <h4 className="text-lg font-bold text-white mb-2">No active monitors</h4>
                <p className="text-slate-500 text-sm max-w-xs mx-auto mb-8">
                  There are currently no smart metrics registered for monitoring.
                </p>
                {isOwner && (
                  <button
                    onClick={() => setShowRegisterForm(true)}
                    className="btn-outline-premium text-xs py-2 px-6"
                  >
                    Register Initial Metric
                  </button>
                )}
              </motion.div>
            )}
          </div>
        </div>
      </div>

      {/* Register Metric Modal */}
      <AnimatePresence>
        {showRegisterForm && (
          <RegisterMetricForm onClose={() => setShowRegisterForm(false)} />
        )}
      </AnimatePresence>
    </div>
  );
}
