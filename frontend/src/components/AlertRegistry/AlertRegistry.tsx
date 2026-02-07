import { useState } from 'react';
import { useAccount, useReadContract } from 'wagmi';
import { AlertCircle, Filter } from 'lucide-react';
import { CONTRACT_ADDRESSES, Priority } from '../../types/contracts';
import { AlertRegistryABI } from '../../config/abis';
import AlertCard from './AlertCard';
import AlertStats from './AlertStats';
import FilterPanel from './FilterPanel';

export default function AlertRegistry() {
  const { isConnected } = useAccount();
  const [selectedPriority, setSelectedPriority] = useState<number | null>(null);
  const [showAcknowledged, setShowAcknowledged] = useState(false);

  // Read alert count
  const { data: alertCount } = useReadContract({
    address: CONTRACT_ADDRESSES.AlertRegistry,
    abi: AlertRegistryABI,
    functionName: 'getAlertCount',
  });

  // Read priority counts
  const { data: lowCount } = useReadContract({
    address: CONTRACT_ADDRESSES.AlertRegistry,
    abi: AlertRegistryABI,
    functionName: 'getPriorityCount',
    args: [Priority.LOW],
  });

  const { data: mediumCount } = useReadContract({
    address: CONTRACT_ADDRESSES.AlertRegistry,
    abi: AlertRegistryABI,
    functionName: 'getPriorityCount',
    args: [Priority.MEDIUM],
  });

  const { data: highCount } = useReadContract({
    address: CONTRACT_ADDRESSES.AlertRegistry,
    abi: AlertRegistryABI,
    functionName: 'getPriorityCount',
    args: [Priority.HIGH],
  });

  const { data: criticalCount } = useReadContract({
    address: CONTRACT_ADDRESSES.AlertRegistry,
    abi: AlertRegistryABI,
    functionName: 'getPriorityCount',
    args: [Priority.CRITICAL],
  });

  // Mock alerts for demo (would come from subgraph/events in production)
  const mockAlerts = [
    {
      id: 1n,
      source: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb' as const,
      timestamp: BigInt(Math.floor(Date.now() / 1000) - 3600),
      message: 'High gas consumption detected in contract execution',
      priority: Priority.HIGH,
      threatLevel: 85n,
      acknowledged: false,
      acknowledger: '0x0000000000000000000000000000000000000000' as const,
      ackTimestamp: 0n,
      expired: false,
    },
    {
      id: 2n,
      source: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb' as const,
      timestamp: BigInt(Math.floor(Date.now() / 1000) - 7200),
      message: 'Unusual transaction pattern identified',
      priority: Priority.MEDIUM,
      threatLevel: 55n,
      acknowledged: true,
      acknowledger: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb' as const,
      ackTimestamp: BigInt(Math.floor(Date.now() / 1000) - 3000),
      expired: false,
    },
    {
      id: 3n,
      source: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb' as const,
      timestamp: BigInt(Math.floor(Date.now() / 1000) - 300),
      message: 'CRITICAL: Potential reentrancy attack detected',
      priority: Priority.CRITICAL,
      threatLevel: 95n,
      acknowledged: false,
      acknowledger: '0x0000000000000000000000000000000000000000' as const,
      ackTimestamp: 0n,
      expired: false,
    },
    {
      id: 4n,
      source: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb' as const,
      timestamp: BigInt(Math.floor(Date.now() / 1000) - 86400),
      message: 'Low priority informational alert',
      priority: Priority.LOW,
      threatLevel: 25n,
      acknowledged: true,
      acknowledger: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb' as const,
      ackTimestamp: BigInt(Math.floor(Date.now() / 1000) - 80000),
      expired: false,
    },
  ];

  // Filter alerts
  const filteredAlerts = mockAlerts.filter((alert) => {
    if (selectedPriority !== null && alert.priority !== selectedPriority) {
      return false;
    }
    if (!showAcknowledged && alert.acknowledged) {
      return false;
    }
    return true;
  });

  // Sort by priority (critical first) and timestamp (recent first)
  const sortedAlerts = [...filteredAlerts].sort((a, b) => {
    if (a.priority !== b.priority) {
      return b.priority - a.priority;
    }
    return Number(b.timestamp - a.timestamp);
  });

  if (!isConnected) {
    return (
      <div className="flex flex-col items-center justify-center h-96">
        <AlertCircle className="w-16 h-16 text-slate-600 mb-4" />
        <h2 className="text-2xl font-bold text-white mb-2">Alert Registry</h2>
        <p className="text-slate-400">Connect your wallet to view system alerts</p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-white mb-2">Alert Registry</h1>
        <p className="text-slate-400">Monitor and manage security alerts</p>
      </div>

      {/* Stats */}
      <AlertStats
        total={alertCount}
        low={lowCount}
        medium={mediumCount}
        high={highCount}
        critical={criticalCount}
      />

      {/* Filters */}
      <div className="card">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white flex items-center space-x-2">
            <Filter className="w-5 h-5" />
            <span>Filters</span>
          </h3>
          <button
            onClick={() => {
              setSelectedPriority(null);
              setShowAcknowledged(false);
            }}
            className="text-sm text-primary-400 hover:text-primary-300"
          >
            Clear All
          </button>
        </div>

        <FilterPanel
          selectedPriority={selectedPriority}
          setSelectedPriority={setSelectedPriority}
          showAcknowledged={showAcknowledged}
          setShowAcknowledged={setShowAcknowledged}
        />
      </div>

      {/* Alerts List */}
      <div>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-xl font-bold text-white">
            Alerts ({sortedAlerts.length})
          </h3>
        </div>

        {sortedAlerts.length === 0 ? (
          <div className="card text-center py-12">
            <AlertCircle className="w-16 h-16 text-slate-600 mx-auto mb-4" />
            <p className="text-slate-400">No alerts match your filters</p>
          </div>
        ) : (
          <div className="space-y-4">
            {sortedAlerts.map((alert) => (
              <AlertCard key={alert.id.toString()} alert={alert} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
