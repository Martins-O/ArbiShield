import { AlertCircle, CheckCircle } from 'lucide-react';
import { formatNumber } from '../../utils/helpers';

interface AlertStatsProps {
  total?: bigint;
  open?: bigint;
  resolved?: bigint;
}

export default function AlertStats({ total, open, resolved }: AlertStatsProps) {
  const stats = [
    {
      label: 'Total Alerts',
      value: total ? formatNumber(total) : '0',
      icon: AlertCircle,
      color: 'text-slate-400',
      bgColor: 'bg-slate-500/20',
    },
    {
      label: 'Open',
      value: open ? formatNumber(open) : '0',
      icon: AlertCircle,
      color: 'text-red-400',
      bgColor: 'bg-red-500/20',
    },
    {
      label: 'Resolved',
      value: resolved ? formatNumber(resolved) : '0',
      icon: CheckCircle,
      color: 'text-green-400',
      bgColor: 'bg-green-500/20',
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {stats.map((stat) => {
        const Icon = stat.icon;
        return (
          <div key={stat.label} className="card">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-slate-400 text-sm">{stat.label}</p>
                <p className={`text-2xl font-bold ${stat.color} mt-1`}>{stat.value}</p>
              </div>
              <div className={`${stat.bgColor} p-3 rounded-lg`}>
                <Icon className={`w-6 h-6 ${stat.color}`} />
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
