import { useAccount, useReadContract, useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { Zap, Power, RotateCcw, Clock, Hash, ShieldAlert, Activity } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { CONTRACT_ADDRESSES } from '../../types/contracts';
import { CircuitBreakerABI } from '../../config/abis';
import { formatTimestamp, formatNumber } from '../../utils/helpers';

export default function CircuitBreaker() {
  const { address, isConnected } = useAccount();

  // Read contract state
  const { data: status } = useReadContract({
    address: CONTRACT_ADDRESSES.CircuitBreaker,
    abi: CircuitBreakerABI,
    functionName: 'getStatus',
  });

  const { data: owner } = useReadContract({
    address: CONTRACT_ADDRESSES.CircuitBreaker,
    abi: CircuitBreakerABI,
    functionName: 'owner',
  });

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const statusArr = status as any;
  const isTripped = statusArr ? Boolean(statusArr[0]) : false;
  const tripCount = statusArr ? BigInt(statusArr[1] || 0) : 0n;
  const lastTripTime = statusArr ? BigInt(statusArr[2] || 0) : 0n;

  // Write functions
  const {
    writeContract: trip,
    data: tripHash,
    isPending: isTripPending
  } = useWriteContract();

  const {
    writeContract: reset,
    data: resetHash,
    isPending: isResetPending
  } = useWriteContract();

  const { isLoading: isTripConfirming } = useWaitForTransactionReceipt({ hash: tripHash });
  const { isLoading: isResetConfirming } = useWaitForTransactionReceipt({ hash: resetHash });

  const isOwner = address === owner;

  const handleTrip = () => {
    trip({
      address: CONTRACT_ADDRESSES.CircuitBreaker,
      abi: CircuitBreakerABI,
      functionName: 'tripManual',
      args: ["Manual trip from UI"],
    });
  };

  const handleReset = () => {
    reset({
      address: CONTRACT_ADDRESSES.CircuitBreaker,
      abi: CircuitBreakerABI,
      functionName: 'reset',
      args: ["Manual reset from UI"],
    });
  };

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
          <h2 className="text-3xl font-black text-white mb-2 tracking-tight">Access Restricted</h2>
          <p className="text-slate-400 text-center px-12">Connect your authorized wallet to monitor and manage the global protocol circuit breaker.</p>
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
            <div className="p-2 bg-purple-500/10 rounded-lg">
              <Zap className="w-6 h-6 text-purple-400" />
            </div>
            <span className="text-xs font-black text-purple-400 uppercase tracking-widest">Emergency System</span>
          </motion.div>
          <motion.h1
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.1 }}
            className="text-4xl md:text-5xl font-black text-white tracking-tighter"
          >
            Circuit Breaker
          </motion.h1>
        </div>
      </div>

      {/* Main Status Display */}
      <motion.div
        layout
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        className={`glass-card relative overflow-hidden p-12 transition-colors duration-700 ${isTripped ? 'bg-red-500/[0.03] border-red-500/30' : 'bg-emerald-500/[0.03] border-emerald-500/30'
          }`}
      >
        <div className="relative z-10 flex flex-col items-center text-center">
          <AnimatePresence mode="wait">
            <motion.div
              key={isTripped ? 'tripped' : 'active'}
              initial={{ scale: 0.8, opacity: 0, rotate: -45 }}
              animate={{ scale: 1, opacity: 1, rotate: 0 }}
              exit={{ scale: 1.2, opacity: 0, rotate: 45 }}
              className={`w-32 h-32 rounded-3xl flex items-center justify-center mb-8 shadow-2xl ${isTripped
                ? 'bg-red-500 text-white shadow-red-500/20'
                : 'bg-emerald-500 text-obsidian shadow-emerald-500/20'
                }`}
            >
              {isTripped ? <Power className="w-16 h-16" /> : <Zap className="w-16 h-16" />}
            </motion.div>
          </AnimatePresence>

          <h2 className="text-4xl font-black text-white mb-4 tracking-tight">
            {isTripped ? 'Protocol Halted' : 'Systems Active'}
          </h2>
          <p className="text-slate-400 max-w-lg mx-auto leading-relaxed mb-12">
            {isTripped
              ? 'The fail-safe mechanism has been engaged. All critical contract functions are currently suspended to maintain asset integrity.'
              : 'The security perimeter is intact. Monitor metrics in the Detection Engine for proactive threat assessment.'}
          </p>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 w-full max-w-xl">
            <div className="p-4 bg-slate-900/50 rounded-2xl border border-white/5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Hash className="w-4 h-4 text-slate-500" />
                <span className="text-[10px] font-black text-slate-500 uppercase tracking-widest">Incident History</span>
              </div>
              <span className="text-xl font-black text-white">{formatNumber(tripCount)}</span>
            </div>
            <div className="p-4 bg-slate-900/50 rounded-2xl border border-white/5 flex items-center justify-between">
              <div className="flex items-center gap-3">
                <Clock className="w-4 h-4 text-slate-500" />
                <span className="text-[10px] font-black text-slate-500 uppercase tracking-widest">Last Trigger</span>
              </div>
              <span className="text-sm font-bold text-white">
                {lastTripTime > 0n ? formatTimestamp(lastTripTime) : 'None'}
              </span>
            </div>
          </div>
        </div>

        {/* Ambient background effect */}
        <div className={`absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full h-full p-20 pointer-events-none opacity-20`}>
          <div className={`w-full h-full rounded-full blur-[120px] ${isTripped ? 'bg-red-500' : 'bg-emerald-500'}`}></div>
        </div>
      </motion.div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-12">
        {/* Controls (Admin Only) */}
        <div className="lg:col-span-12 xl:col-span-7">
          <div className="glass-card">
            <div className="flex items-center gap-2 mb-6">
              <ShieldAlert className="w-5 h-5 text-red-500" />
              <h3 className="text-xl font-black text-white">Security Controls</h3>
            </div>

            <div className="p-6 bg-slate-900/50 rounded-2xl border-2 border-dashed border-white/5 mb-8">
              {!isOwner ? (
                <div className="text-center py-4">
                  <p className="text-sm text-slate-500 font-medium italic">Available only to the Protocol Governance / Owner</p>
                </div>
              ) : (
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
                  <div className="space-y-4">
                    <h4 className="text-xs font-black text-slate-400 uppercase tracking-widest">Immediate Response</h4>
                    <button
                      onClick={handleTrip}
                      disabled={isTripped || isTripPending || isTripConfirming}
                      className="btn-premium !bg-red-600 !shadow-red-900/40 w-full py-4 !rounded-xl"
                    >
                      <Power className="w-5 h-5" />
                      <span>{isTripPending || isTripConfirming ? 'Halting...' : 'Engage Breaker'}</span>
                    </button>
                    <p className="text-[10px] text-slate-500 leading-tight">Engaging the breaker immediately suspends all core assets and functions across the protocol.</p>
                  </div>
                  <div className="space-y-4 pt-8 sm:pt-0 sm:border-l sm:border-white/5 sm:pl-6">
                    <h4 className="text-xs font-black text-slate-400 uppercase tracking-widest">Protocol Recovery</h4>
                    <button
                      onClick={handleReset}
                      disabled={!isTripped || isResetPending || isResetConfirming}
                      className="btn-outline-premium w-full py-4 !rounded-xl !border-emerald-500/30 hover:!bg-emerald-500/5 text-emerald-400"
                    >
                      <RotateCcw className="w-5 h-5" />
                      <span>{isResetPending || isResetConfirming ? 'Recovering...' : 'Reset & Resume'}</span>
                    </button>
                    <p className="text-[10px] text-slate-500 leading-tight">Resetting requires incident post-mortem validation. All halted services will be restored to normal operation.</p>
                  </div>
                </div>
              )}
            </div>

            <div className="flex items-start gap-3 p-4 bg-cyan-500/5 rounded-xl border border-cyan-500/10">
              <Activity className="w-5 h-5 text-cyan-400 shrink-0" />
              <p className="text-xs text-slate-400 leading-relaxed">
                ArbiShield employs a multi-tiered security approach. While the Breaker is a manual "last resort",
                the Detection Engine can be configured to autonomously engage this breaker based on cryptographic proofs
                and threshold violations.
              </p>
            </div>
          </div>
        </div>

        {/* Info & Specs */}
        <div className="lg:col-span-12 xl:col-span-5">
          <div className="space-y-6">
            <div className="glass-card">
              <h4 className="text-white font-black text-sm uppercase tracking-widest mb-6">Security Specifications</h4>
              <ul className="space-y-4">
                {[
                  { label: 'Latency', value: '< 200ms', sub: 'On Arbi-Sepolia' },
                  { label: 'Governance', value: '1 of 1', sub: 'Owner-controlled' },
                  { label: 'WASM Runtime', value: 'Native', sub: 'Arbitrum Stylus' },
                ].map((spec, i) => (
                  <li key={i} className="flex items-center justify-between pb-4 border-b border-white/5 last:border-0 last:pb-0">
                    <div>
                      <p className="text-xs font-bold text-white">{spec.label}</p>
                      <p className="text-[10px] text-slate-500 font-medium uppercase tracking-widest">{spec.sub}</p>
                    </div>
                    <span className="text-slate-300 font-black text-sm">{spec.value}</span>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
