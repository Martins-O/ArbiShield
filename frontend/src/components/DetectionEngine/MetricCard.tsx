import { TrendingUp, AlertTriangle, CheckCircle } from 'lucide-react';
import { formatNumber, calculatePercentage } from '../../utils/helpers';

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
    <div className="card">
      <div className="flex items-center justify-between mb-4">
        <h4 className="text-lg font-semibold text-white">Metric #{metric.id.toString()}</h4>
        {isAnomaly ? (
          <AlertTriangle className="w-6 h-6 text-orange-500" />
        ) : (
          <CheckCircle className="w-6 h-6 text-green-500" />
        )}
      </div>

      <div className="space-y-3">
        <div>
          <p className="text-sm text-slate-400">Current Value</p>
          <p className="text-xl font-bold text-white">{formatNumber(metric.currentValue)}</p>
        </div>

        <div>
          <p className="text-sm text-slate-400">Threshold</p>
          <p className="text-lg text-slate-300">{formatNumber(metric.threshold)}</p>
        </div>

        {/* Progress Bar */}
        <div>
          <div className="flex items-center justify-between mb-1">
            <p className="text-xs text-slate-400">Status</p>
            <p className="text-xs text-slate-400">{percentage}%</p>
          </div>
          <div className="w-full bg-slate-700 rounded-full h-2">
            <div
              className={`h-2 rounded-full transition-all ${
                isAnomaly ? 'bg-orange-500' : 'bg-green-500'
              }`}
              style={{ width: `${Math.min(percentage, 100)}%` }}
            ></div>
          </div>
        </div>

        {/* Status Badge */}
        <div className="flex items-center justify-between pt-2">
          <span
            className={`px-3 py-1 rounded-full text-xs font-medium ${
              isAnomaly
                ? 'bg-orange-500/20 text-orange-400 border border-orange-500/30'
                : 'bg-green-500/20 text-green-400 border border-green-500/30'
            }`}
          >
            {isAnomaly ? 'Anomaly Detected' : 'Normal'}
          </span>
          {isAnomaly && (
            <span className="text-xs text-slate-400">
              +{formatNumber(metric.currentValue - metric.threshold)} over
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
