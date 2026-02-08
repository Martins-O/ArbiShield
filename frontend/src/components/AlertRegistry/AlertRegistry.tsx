import { useState } from 'react';
import { useAccount, useReadContract } from 'wagmi';
import { AlertCircle, Filter } from 'lucide-react';
import { CONTRACT_ADDRESSES, Priority, Alert } from '../../types/contracts';
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

  // TODO: Fetch alerts from contract events or subgraph
  // For now, empty array - will be populated once contracts are deployed
  const alerts: Alert[] = [];

  // Filter alerts
  const filteredAlerts = alerts.filter((alert) => {
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
            <h4 className="text-lg font-semibold text-white mb-2">No Alerts Yet</h4>
            <p className="text-slate-400">
              {alerts.length === 0
                ? "Alerts will appear here once contracts are deployed and monitoring begins"
                : "No alerts match your current filters"}
            </p>
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
