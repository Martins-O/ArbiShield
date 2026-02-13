import { AlertTriangle, CheckCircle, Activity } from 'lucide-react';
import { formatNumber, calculatePercentage } from '../../utils/helpers';
import { motion } from 'framer-motion';

interface MetricCardProps {
  metric: {
    id: bigint;
    threshold: bigint;
    currentValue: bigint;
  };
}

export default function MetricCard({ metric }: MetricCardProps) {
  const isAnomaly = metric.currentValue > metric.threshold;
  const percentage = calculatePercentage(metric.currentValue, metric.threshold);

  return (
    <motion.div
      layout
      className="glass-card !p-5 relative"
    >
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-2">
          <div className={`p-1.5 rounded-md bg-slate-800 ${isAnomaly ? 'text-red-400' : 'text-emerald-400'}`}>
            <Activity className="w-3.5 h-3.5" />
          </div>
          <h4 className="text-sm font-black text-white tracking-tight">NODE #{metric.id.toString()}</h4>
        </div>
        {isAnomaly ? (
          <div className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-red-500/10 border border-red-500/20">
            <AlertTriangle className="w-3 h-3 text-red-500" />
            <span className="text-[10px] font-black text-red-500 uppercase tracking-widest">Anomaly</span>
          </div>
        ) : (
          <div className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-emerald-500/10 border border-emerald-500/20">
            <CheckCircle className="w-3 h-3 text-emerald-500" />
            <span className="text-[10px] font-black text-emerald-500 uppercase tracking-widest">Secure</span>
          </div>
        )}
      </div>

      <div className="space-y-6">
        <div className="flex items-end justify-between">
          <div>
            <p className="text-[10px] font-black text-slate-500 uppercase tracking-widest mb-1">Observation</p>
            <p className="text-2xl font-black text-white">{formatNumber(metric.currentValue)}</p>
          </div>
          <div className="text-right">
            <p className="text-[10px] font-black text-slate-500 uppercase tracking-widest mb-1">Limit</p>
            <p className="text-sm font-bold text-slate-400">{formatNumber(metric.threshold)}</p>
          </div>
        </div>

        {/* Progress Bar */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="h-1 flex-1 bg-slate-800 rounded-full overflow-hidden">
              <motion.div
                initial={{ width: 0 }}
                animate={{ width: `${Math.min(percentage, 100)}%` }}
                className={`h-full rounded-full ${isAnomaly ? 'bg-red-500 shadow-[0_0_10px_#ef4444]' : 'bg-cyan-500 shadow-[0_0_10px_#06b6d4]'
                  }`}
              />
            </div>
            <span className={`text-[10px] font-black ml-3 ${isAnomaly ? 'text-red-400' : 'text-cyan-400'}`}>
              {percentage}%
            </span>
          </div>
        </div>

        {isAnomaly && (
          <motion.div
            initial={{ opacity: 0, y: 5 }}
            animate={{ opacity: 1, y: 0 }}
            className="pt-2 border-t border-white/5"
          >
            <p className="text-[10px] text-slate-400 leading-tight">
              Surgical intervention recommended. Value is <span className="text-red-400 font-bold">{formatNumber(metric.currentValue - metric.threshold)}</span> units above specified safety parameters.
            </p>
          </motion.div>
        )}
      </div>
    </motion.div>
  );
}
