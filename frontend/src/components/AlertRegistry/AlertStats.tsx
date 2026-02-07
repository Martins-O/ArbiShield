import { AlertCircle, Info, AlertTriangle, XCircle } from 'lucide-react';
import { formatNumber } from '../../utils/helpers';

interface AlertStatsProps {
  total?: bigint;
  low?: bigint;
  medium?: bigint;
  high?: bigint;
  critical?: bigint;
}

export default function AlertStats({ total, low, medium, high, critical }: AlertStatsProps) {
  const stats = [
    {
      label: 'Total Alerts',
      value: total ? formatNumber(total) : '0',
      icon: AlertCircle,
      color: 'text-slate-400',
      bgColor: 'bg-slate-500/20',
    },
    {
      label: 'Low Priority',
      value: low ? formatNumber(low) : '0',
      icon: Info,
      color: 'text-blue-400',
      bgColor: 'bg-blue-500/20',
    },
    {
      label: 'Medium Priority',
      value: medium ? formatNumber(medium) : '0',
      icon: AlertTriangle,
      color: 'text-yellow-400',
      bgColor: 'bg-yellow-500/20',
    },
    {
      label: 'High Priority',
      value: high ? formatNumber(high) : '0',
      icon: AlertTriangle,
      color: 'text-orange-400',
      bgColor: 'bg-orange-500/20',
    },
    {
      label: 'Critical',
      value: critical ? formatNumber(critical) : '0',
      icon: XCircle,
      color: 'text-red-400',
      bgColor: 'bg-red-500/20',
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
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
