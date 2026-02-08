import { useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { Clock, CheckCircle } from 'lucide-react';
import { Alert } from '../../types/contracts';
import { CONTRACT_ADDRESSES } from '../../types/contracts';
import { AlertRegistryABI } from '../../config/abis';
import {
  formatAddress,
  formatRelativeTime,
  formatTimestamp,
  getPriorityLabel,
  getPriorityBadgeClass,
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

  return (
    <div
      className={`card border-l-4 ${
        alert.priority === 3
          ? 'border-l-red-500'
          : alert.priority === 2
          ? 'border-l-orange-500'
          : alert.priority === 1
          ? 'border-l-yellow-500'
          : 'border-l-blue-500'
      } ${alert.acknowledged ? 'opacity-60' : ''}`}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          {/* Header */}
          <div className="flex items-center space-x-3 mb-2">
            <span className={getPriorityBadgeClass(alert.priority)}>
              {getPriorityLabel(alert.priority)}
            </span>
            <span className="text-slate-500 text-sm">
              #{alert.id.toString()}
            </span>
            {alert.acknowledged && (
              <span className="bg-green-500/20 text-green-400 border border-green-500/30 px-2 py-1 rounded text-xs font-medium flex items-center space-x-1">
                <CheckCircle className="w-3 h-3" />
                <span>Acknowledged</span>
              </span>
            )}
          </div>

          {/* Message */}
          <p className="text-white font-medium mb-3">{alert.message}</p>

          {/* Meta Info */}
          <div className="flex flex-wrap items-center gap-4 text-sm text-slate-400">
            <div className="flex items-center space-x-1">
              <Clock className="w-4 h-4" />
              <span>{formatRelativeTime(alert.timestamp)}</span>
              <span className="text-slate-600">•</span>
              <span className="text-xs">{formatTimestamp(alert.timestamp)}</span>
            </div>
            <div>
              Source: <span className="text-slate-300">{formatAddress(alert.source)}</span>
            </div>
            <div>
              Threat Level: <span className="text-slate-300">{alert.threatLevel.toString()}%</span>
            </div>
          </div>

          {/* Acknowledgment Info */}
          {alert.acknowledged && alert.acknowledger !== '0x0000000000000000000000000000000000000000' && (
            <div className="mt-3 pt-3 border-t border-slate-700 text-sm text-slate-400">
              Acknowledged by {formatAddress(alert.acknowledger)} on {formatTimestamp(alert.ackTimestamp)}
            </div>
          )}
        </div>

        {/* Action Button */}
        {!alert.acknowledged && (
          <button
            onClick={handleAcknowledge}
            disabled={isPending || isConfirming}
            className="btn-primary ml-4"
          >
            {isPending || isConfirming ? 'Acknowledging...' : 'Acknowledge'}
          </button>
        )}
      </div>
    </div>
  );
}
