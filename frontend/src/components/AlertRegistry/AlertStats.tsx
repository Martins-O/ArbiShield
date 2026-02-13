import { Activity, ShieldAlert, ShieldCheck } from 'lucide-react';
import { formatNumber } from '../../utils/helpers';
import { motion } from 'framer-motion';

interface AlertStatsProps {
  total?: bigint;
  open?: bigint;
  resolved?: bigint;
}

export default function AlertStats({ total, open, resolved }: AlertStatsProps) {
  const stats = [
    {
      label: 'Security Events',
      value: total ? formatNumber(total) : '0',
      icon: Activity,
      color: 'cyan',
    },
    {
      label: 'Active Alerts',
      value: open ? formatNumber(open) : '0',
      icon: ShieldAlert,
      color: 'red',
    },
    {
      label: 'Neutralized',
      value: resolved ? formatNumber(resolved) : '0',
      icon: ShieldCheck,
      color: 'emerald',
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
      {stats.map((stat, i) => {
        const Icon = stat.icon;
        return (
          <motion.div
            key={stat.label}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.1 }}
            className="glass-card flex items-center justify-between"
          >
            <div>
              <p className="text-[10px] font-black text-slate-500 uppercase tracking-widest mb-1">{stat.label}</p>
              <p className={`text-3xl font-black text-white mt-1`}>{stat.value}</p>
            </div>
            <div className={`p-3 bg-${stat.color}-500/10 rounded-xl`}>
              <Icon className={`w-6 h-6 text-${stat.color}-400`} />
            </div>
          </motion.div>
        );
      })}
    </div>
  );
}
