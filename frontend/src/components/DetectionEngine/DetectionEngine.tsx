import { useState } from 'react';
import { useAccount, useReadContract, useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { Activity, Plus, TrendingUp, AlertTriangle } from 'lucide-react';
import { CONTRACT_ADDRESSES } from '../../types/contracts';
import { DetectionEngineABI } from '../../config/abis';
import { formatNumber, calculatePercentage } from '../../utils/helpers';
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

  // Report metric mutation
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
        functionName: 'reportMetric',
        args: [BigInt(metricId), BigInt(value)],
      });
      setMetricId('');
      setValue('');
    } catch (error) {
      console.error('Error reporting metric:', error);
    }
  };

  // Mock metrics for demo (would come from subgraph/events in production)
  const mockMetrics = [
    { id: 1n, threshold: 1000n, currentValue: 850n },
    { id: 2n, threshold: 500n, currentValue: 620n },
    { id: 3n, threshold: 2000n, currentValue: 1200n },
  ];

  if (!isConnected) {
    return (
      <div className="flex flex-col items-center justify-center h-96">
        <Activity className="w-16 h-16 text-slate-600 mb-4" />
        <h2 className="text-2xl font-bold text-white mb-2">Detection Engine</h2>
        <p className="text-slate-400">Connect your wallet to interact with the Detection Engine</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-white mb-2">Detection Engine</h1>
          <p className="text-slate-400">Monitor metrics and detect anomalies in real-time</p>
        </div>
        {isOwner && (
          <button
            onClick={() => setShowRegisterForm(true)}
            className="btn-primary flex items-center space-x-2"
          >
            <Plus className="w-5 h-5" />
            <span>Register Metric</span>
          </button>
        )}
      </div>

      {/* Stats Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Total Metrics</p>
              <p className="text-2xl font-bold text-white mt-1">{mockMetrics.length}</p>
            </div>
            <Activity className="w-10 h-10 text-primary-500" />
          </div>
        </div>

        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Anomalies Detected</p>
              <p className="text-2xl font-bold text-orange-500 mt-1">1</p>
            </div>
            <AlertTriangle className="w-10 h-10 text-orange-500" />
          </div>
        </div>

        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm">Normal Status</p>
              <p className="text-2xl font-bold text-green-500 mt-1">2</p>
            </div>
            <TrendingUp className="w-10 h-10 text-green-500" />
          </div>
        </div>
      </div>

      {/* Report Metric Form */}
      <div className="card">
        <h3 className="text-xl font-bold text-white mb-4">Report Metric</h3>
        <form onSubmit={handleReportMetric} className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-slate-400 mb-2">
                Metric ID
              </label>
              <input
                type="number"
                value={metricId}
                onChange={(e) => setMetricId(e.target.value)}
                className="input w-full"
                placeholder="e.g., 1"
                min="0"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-400 mb-2">
                Value
              </label>
              <input
                type="number"
                value={value}
                onChange={(e) => setValue(e.target.value)}
                className="input w-full"
                placeholder="e.g., 850"
                min="0"
              />
            </div>
          </div>
          <button
            type="submit"
            disabled={isPending || isConfirming || !metricId || !value}
            className="btn-primary w-full"
          >
            {isPending || isConfirming ? 'Reporting...' : 'Report Metric'}
          </button>
        </form>
      </div>

      {/* Metrics List */}
      <div>
        <h3 className="text-xl font-bold text-white mb-4">Registered Metrics</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {mockMetrics.map((metric) => (
            <MetricCard key={metric.id.toString()} metric={metric} />
          ))}
        </div>
      </div>

      {/* Register Metric Modal */}
      {showRegisterForm && (
        <RegisterMetricForm onClose={() => setShowRegisterForm(false)} />
      )}
    </div>
  );
}
