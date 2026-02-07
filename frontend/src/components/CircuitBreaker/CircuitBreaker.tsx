import { useAccount, useReadContract, useWriteContract, useWaitForTransactionReceipt } from 'wagmi';
import { Zap, Power, RotateCcw, Clock, Hash } from 'lucide-react';
import { CONTRACT_ADDRESSES } from '../../types/contracts';
import { CircuitBreakerABI } from '../../config/abis';
import { formatTimestamp, formatNumber } from '../../utils/helpers';

export default function CircuitBreaker() {
  const { address, isConnected } = useAccount();

  // Read contract state
  const { data: isTripped, isLoading: isTrippedLoading } = useReadContract({
    address: CONTRACT_ADDRESSES.CircuitBreaker,
    abi: CircuitBreakerABI,
    functionName: 'isTripped',
  });

  const { data: tripCount } = useReadContract({
    address: CONTRACT_ADDRESSES.CircuitBreaker,
    abi: CircuitBreakerABI,
    functionName: 'getTripCount',
  });

  const { data: lastTripTime } = useReadContract({
    address: CONTRACT_ADDRESSES.CircuitBreaker,
    abi: CircuitBreakerABI,
    functionName: 'getLastTripTime',
  });

  const { data: owner } = useReadContract({
    address: CONTRACT_ADDRESSES.CircuitBreaker,
    abi: CircuitBreakerABI,
    functionName: 'owner',
  });

  // Write functions
  const {
    writeContract: trip,
    data: tripHash,
    isPending: isTripPending
  } = useWriteContract();

  const {
    writeContract: reset,
    data: resetHash,
    isPending: isResetPending
  } = useWriteContract();

  const { isLoading: isTripConfirming } = useWaitForTransactionReceipt({ hash: tripHash });
  const { isLoading: isResetConfirming } = useWaitForTransactionReceipt({ hash: resetHash });

  const isOwner = address === owner;

  const handleTrip = () => {
    trip({
      address: CONTRACT_ADDRESSES.CircuitBreaker,
      abi: CircuitBreakerABI,
      functionName: 'trip',
    });
  };

  const handleReset = () => {
    reset({
      address: CONTRACT_ADDRESSES.CircuitBreaker,
      abi: CircuitBreakerABI,
      functionName: 'reset',
    });
  };

  if (!isConnected) {
    return (
      <div className="flex flex-col items-center justify-center h-96">
        <Zap className="w-16 h-16 text-slate-600 mb-4" />
        <h2 className="text-2xl font-bold text-white mb-2">Circuit Breaker</h2>
        <p className="text-slate-400">Connect your wallet to view circuit breaker status</p>
      </div>
    );
  }

  const tripped = isTripped ?? false;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-white mb-2">Circuit Breaker</h1>
        <p className="text-slate-400">Emergency system pause mechanism</p>
      </div>

      {/* Status Card */}
      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-xl font-bold text-white">System Status</h3>
          <div className={`flex items-center space-x-2 px-4 py-2 rounded-lg ${
            tripped
              ? 'bg-red-500/20 text-red-400 border border-red-500/30'
              : 'bg-green-500/20 text-green-400 border border-green-500/30'
          }`}>
            <div className={`w-3 h-3 rounded-full ${tripped ? 'bg-red-500 animate-pulse' : 'bg-green-500'}`}></div>
            <span className="font-medium">{tripped ? 'TRIPPED' : 'ACTIVE'}</span>
          </div>
        </div>

        {/* Large Status Indicator */}
        <div className="flex flex-col items-center justify-center py-12">
          <div className={`w-32 h-32 rounded-full flex items-center justify-center mb-6 ${
            tripped
              ? 'bg-red-500/10 border-4 border-red-500'
              : 'bg-green-500/10 border-4 border-green-500'
          }`}>
            {tripped ? (
              <Power className="w-16 h-16 text-red-500" />
            ) : (
              <Zap className="w-16 h-16 text-green-500" />
            )}
          </div>
          <h2 className="text-3xl font-bold text-white mb-2">
            {tripped ? 'System Paused' : 'System Active'}
          </h2>
          <p className="text-slate-400 text-center max-w-md">
            {tripped
              ? 'The circuit breaker has been triggered. All contract operations are paused.'
              : 'System is operating normally. Circuit breaker is ready for emergency activation.'}
          </p>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-slate-400 text-sm flex items-center space-x-2">
                <Hash className="w-4 h-4" />
                <span>Total Trips</span>
              </p>
              <p className="text-3xl font-bold text-white mt-2">
                {tripCount ? formatNumber(tripCount) : '0'}
              </p>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="flex items-center justify-between">
            <div className="w-full">
              <p className="text-slate-400 text-sm flex items-center space-x-2 mb-2">
                <Clock className="w-4 h-4" />
                <span>Last Trip Time</span>
              </p>
              <p className="text-lg text-white">
                {lastTripTime && lastTripTime > 0n
                  ? formatTimestamp(lastTripTime)
                  : 'Never'}
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Control Panel (Owner Only) */}
      {isOwner && (
        <div className="card">
          <h3 className="text-xl font-bold text-white mb-4">Control Panel</h3>
          <p className="text-slate-400 mb-6">
            Emergency controls for the circuit breaker. Use with caution.
          </p>

          <div className="grid grid-cols-2 gap-4">
            <button
              onClick={handleTrip}
              disabled={tripped || isTripPending || isTripConfirming}
              className="btn-danger flex items-center justify-center space-x-2 py-4"
            >
              <Power className="w-5 h-5" />
              <span>{isTripPending || isTripConfirming ? 'Tripping...' : 'Trip Circuit'}</span>
            </button>

            <button
              onClick={handleReset}
              disabled={!tripped || isResetPending || isResetConfirming}
              className="btn-primary flex items-center justify-center space-x-2 py-4"
            >
              <RotateCcw className="w-5 h-5" />
              <span>{isResetPending || isResetConfirming ? 'Resetting...' : 'Reset Circuit'}</span>
            </button>
          </div>
        </div>
      )}

      {/* Info Box */}
      <div className="card bg-blue-500/10 border-blue-500/30">
        <h4 className="text-lg font-semibold text-blue-400 mb-2">ℹ️ How it Works</h4>
        <p className="text-slate-300">
          The circuit breaker is an emergency pause mechanism that can be triggered by the contract owner
          to halt all critical operations when a security threat is detected. Once tripped, the system
          remains paused until manually reset by the owner.
        </p>
      </div>
    </div>
  );
}
