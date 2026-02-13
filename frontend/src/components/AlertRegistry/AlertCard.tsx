import { useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { Clock, ShieldAlert, User, ShieldCheck, ArrowRight, Activity } from 'lucide-react';
import { motion } from 'framer-motion';
import { Alert } from '../../types/contracts';
import { CONTRACT_ADDRESSES } from '../../types/contracts';
import { AlertRegistryABI } from '../../config/abis';
import {
  formatAddress,
  formatRelativeTime,
  formatTimestamp,
  getPriorityLabel,
} from '../../utils/helpers';

interface AlertCardProps {
  alert: Alert;
}

export default function AlertCard({ alert }: AlertCardProps) {
  const { writeContract, data: hash, isPending } = useWriteContract();
  const { isLoading: isConfirming } = useWaitForTransactionReceipt({ hash });

  const handleAcknowledge = () => {
    writeContract({
      address: CONTRACT_ADDRESSES.AlertRegistry,
      abi: AlertRegistryABI,
      functionName: 'acknowledgeAlert',
      args: [alert.id],
    });
  };

  const getPriorityTheme = (p: number) => {
    switch (p) {
      case 3: return { color: 'red', icon: ShieldAlert };
      case 2: return { color: 'orange', icon: ShieldAlert };
      case 1: return { color: 'yellow', icon: ShieldAlert };
      default: return { color: 'cyan', icon: Activity };
    }
  };

  const theme = getPriorityTheme(alert.priority);
  const StatusIcon = theme.icon;

  return (
    <div
      className={`glass-card relative overflow-hidden group transition-all duration-300 ${alert.acknowledged ? 'opacity-50 grayscale hover:grayscale-0 hover:opacity-100' : ''
        }`}
    >
      <div className={`absolute left-0 top-0 bottom-0 w-1 bg-${theme.color}-500 shadow-[2px_0_10px_rgba(239,68,68,0.3)]`}></div>

      <div className="flex flex-col md:flex-row md:items-start justify-between gap-6">
        <div className="flex-1 space-y-4">
          {/* Header */}
          <div className="flex flex-wrap items-center gap-3">
            <div className={`flex items-center gap-1.5 px-2 py-1 rounded-md bg-${theme.color}-500/10 border border-${theme.color}-500/20`}>
              <StatusIcon className={`w-3 h-3 text-${theme.color}-500`} />
              <span className={`text-[10px] font-black text-${theme.color}-500 uppercase tracking-widest`}>
                {getPriorityLabel(alert.priority)}
              </span>
            </div>
            <span className="text-[10px] font-black text-slate-500 uppercase tracking-widest">
              Event Hash: <span className="text-white ml-1">#{alert.id.toString()}</span>
            </span>
            {alert.acknowledged && (
              <div className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-emerald-500/10 border border-emerald-500/20">
                <ShieldCheck className="w-3 h-3 text-emerald-500" />
                <span className="text-[10px] font-black text-emerald-500 uppercase tracking-widest">Neutralized</span>
              </div>
            )}
          </div>

          {/* Message */}
          <h4 className="text-lg font-bold text-white tracking-tight leading-snug group-hover:text-cyan-400 transition-colors">
            {alert.message}
          </h4>

          {/* Meta Info */}
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-6 pt-2">
            <div className="space-y-1">
              <div className="flex items-center gap-2 text-slate-500">
                <Clock className="w-3 h-3" />
                <span className="text-[9px] font-black uppercase tracking-widest">Time Registered</span>
              </div>
              <p className="text-xs font-bold text-white">{formatRelativeTime(alert.timestamp)}</p>
              <p className="text-[9px] text-slate-500 italic mt-0.5">{formatTimestamp(alert.timestamp)}</p>
            </div>

            <div className="space-y-1">
              <div className="flex items-center gap-2 text-slate-500">
                <User className="w-3 h-3" />
                <span className="text-[9px] font-black uppercase tracking-widest">Reporting Source</span>
              </div>
              <p className="text-xs font-bold text-slate-300 font-mono tracking-tight">{formatAddress(alert.source)}</p>
            </div>

            <div className="space-y-1">
              <div className="flex items-center gap-2 text-slate-500">
                <ShieldAlert className="w-3 h-3" />
                <span className="text-[9px] font-black uppercase tracking-widest">Threat Score</span>
              </div>
              <div className="flex items-center gap-3">
                <div className="h-1.5 flex-1 bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className={`h-full bg-${theme.color}-500`}
                    style={{ width: `${alert.threatLevel.toString()}%` }}
                  ></div>
                </div>
                <span className="text-xs font-black text-white">{alert.threatLevel.toString()}%</span>
              </div>
            </div>
          </div>

          {/* Acknowledgment Info */}
          {alert.acknowledged && alert.acknowledger !== '0x0000000000000000000000000000000000000000' && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              className="mt-4 p-3 bg-white/5 rounded-xl border border-white/5 flex items-center gap-3"
            >
              <div className="p-1.5 bg-emerald-500/10 rounded-lg">
                <ShieldCheck className="w-3 h-3 text-emerald-500" />
              </div>
              <p className="text-[10px] text-slate-400 font-medium">
                Incident neutralized by <span className="text-white font-bold">{formatAddress(alert.acknowledger)}</span> at {formatTimestamp(alert.ackTimestamp)}
              </p>
            </motion.div>
          )}
        </div>

        {/* Action Button */}
        {!alert.acknowledged && (
          <div className="shrink-0">
            <button
              onClick={handleAcknowledge}
              disabled={isPending || isConfirming}
              className="btn-premium !rounded-xl text-xs py-3 px-6 h-auto group/btn flex items-center gap-2"
            >
              {isPending || isConfirming ? 'Neutralizing...' : (
                <>
                  Acknowledge Threat
                  <ArrowRight className="w-3 h-3 group-hover/btn:translate-x-1 transition-transform" />
                </>
              )}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
