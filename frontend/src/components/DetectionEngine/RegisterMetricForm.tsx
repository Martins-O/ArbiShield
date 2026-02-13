import { useState } from 'react';
import { useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { X } from 'lucide-react';
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
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="card w-full max-w-md">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-xl font-bold text-white">Register New Metric</h3>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-white transition-colors"
          >
            <X className="w-6 h-6" />
          </button>
        </div>

        {isSuccess ? (
          <div className="text-center py-8">
            <div className="w-16 h-16 bg-green-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
              <svg className="w-8 h-8 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <p className="text-green-500 font-medium">Metric registered successfully!</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-slate-400 mb-2">
                Threshold
              </label>
              <input
                type="number"
                value={threshold}
                onChange={(e) => setThreshold(e.target.value)}
                className="input w-full"
                placeholder="e.g., 1000"
                min="1"
                required
              />
              <p className="text-xs text-slate-500 mt-1">
                Values above this threshold will be flagged as anomalies
              </p>
            </div>

            <div className="flex space-x-3 pt-4">
              <button
                type="button"
                onClick={onClose}
                className="btn-secondary flex-1"
                disabled={isPending || isConfirming}
              >
                Cancel
              </button>
              <button
                type="submit"
                className="btn-primary flex-1"
                disabled={isPending || isConfirming || !threshold}
              >
                {isPending || isConfirming ? 'Registering...' : 'Register Metric'}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}
