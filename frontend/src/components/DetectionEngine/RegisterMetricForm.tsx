import { useState } from 'react';
import { useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { X, ShieldPlus, ChevronRight } from 'lucide-react';
import { motion } from 'framer-motion';
import { CONTRACT_ADDRESSES } from '../../types/contracts';
import { DetectionEngineABI } from '../../config/abis';

interface RegisterMetricFormProps {
  onClose: () => void;
}

export default function RegisterMetricForm({ onClose }: RegisterMetricFormProps) {
  const [threshold, setThreshold] = useState('');

  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming, isSuccess } = useWaitForTransactionReceipt({ hash });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!threshold) return;

    try {
      writeContract({
        address: CONTRACT_ADDRESSES.DetectionEngine,
        abi: DetectionEngineABI,
        functionName: 'registerMetric',
        args: [BigInt(threshold)],
      });
    } catch (error) {
      console.error('Error registering metric:', error);
    }
  };

  if (isSuccess) {
    setTimeout(() => {
      onClose();
    }, 2000);
  }

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 bg-obsidian/80 backdrop-blur-sm flex items-center justify-center z-[100] p-6"
    >
      <motion.div
        initial={{ scale: 0.9, opacity: 0, y: 20 }}
        animate={{ scale: 1, opacity: 1, y: 0 }}
        exit={{ scale: 0.9, opacity: 0, y: 20 }}
        className="glass-card w-full max-w-lg relative overflow-hidden"
      >
        <div className="absolute top-0 right-0 w-32 h-32 bg-cyan-500/5 blur-3xl rounded-full translate-x-12 -translate-y-12"></div>

        <div className="flex items-center justify-between mb-8 relative z-10">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-slate-800 rounded-lg">
              <ShieldPlus className="w-5 h-5 text-cyan-400" />
            </div>
            <div>
              <h3 className="text-xl font-black text-white tracking-tight">Deploy Monitor</h3>
              <p className="text-[10px] text-slate-500 font-bold uppercase tracking-widest">Detection configuration</p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-white/5 rounded-full transition-colors group"
          >
            <X className="w-5 h-5 text-slate-500 group-hover:text-white" />
          </button>
        </div>

        {isSuccess ? (
          <motion.div
            initial={{ opacity: 0, scale: 0.9 }}
            animate={{ opacity: 1, scale: 1 }}
            className="text-center py-12"
          >
            <div className="w-20 h-20 bg-emerald-500/20 rounded-full flex items-center justify-center mx-auto mb-6 shadow-[0_0_30px_rgba(16,185,129,0.2)]">
              <svg className="w-10 h-10 text-emerald-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <motion.path
                  initial={{ pathLength: 0 }}
                  animate={{ pathLength: 1 }}
                  transition={{ duration: 0.5, delay: 0.2 }}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={3}
                  d="M5 13l4 4L19 7"
                />
              </svg>
            </div>
            <h4 className="text-xl font-black text-white mb-2">Monitor Deployed</h4>
            <p className="text-emerald-500/80 font-bold text-sm tracking-wide">Metric registration confirmed on-chain.</p>
          </motion.div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-8 relative z-10">
            <div className="space-y-6">
              <div>
                <label className="block text-[10px] font-black text-slate-500 uppercase tracking-widest mb-3">
                  Safety Threshold
                </label>
                <div className="relative">
                  <input
                    type="number"
                    value={threshold}
                    onChange={(e) => setThreshold(e.target.value)}
                    className="input-premium w-full !pl-12"
                    placeholder="Enter limit value"
                    min="1"
                    required
                  />
                  <ShieldPlus className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
                </div>
                <p className="text-[10px] text-slate-500 font-medium mt-3 italic">
                  Systems exceeding this value will trigger an immediate anomaly event in the detection registry.
                </p>
              </div>
            </div>

            <div className="flex gap-4">
              <button
                type="button"
                onClick={onClose}
                className="btn-outline-premium !rounded-xl flex-1 text-xs py-4"
                disabled={isPending || isConfirming}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="btn-premium flex-1 !rounded-xl text-xs py-4 group"
                disabled={isPending || isConfirming || !threshold}
              >
                <span className="flex items-center justify-center gap-2">
                  {isPending || isConfirming ? 'Confirming...' : (
                    <>
                      Initialize Monitor
                      <ChevronRight className="w-4 h-4 group-hover:translate-x-1 transition-transform" />
                    </>
                  )}
                </span>
              </button>
            </div>
          </form>
        )}
      </motion.div>
    </motion.div>
  );
}
