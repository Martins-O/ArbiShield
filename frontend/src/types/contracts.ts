// ArbiShield Contract Types

export type Address = `0x${string}`;

export interface Alert {
  id: bigint;
  source: Address;
  timestamp: bigint;
  message: string;
  priority: number; // 0=LOW, 1=MEDIUM, 2=HIGH, 3=CRITICAL
  threatLevel: bigint;
  acknowledged: boolean;
  acknowledger: Address;
  ackTimestamp: bigint;
  expired: boolean;
}

export enum Priority {
  LOW = 0,
  MEDIUM = 1,
  HIGH = 2,
  CRITICAL = 3,
}

export enum Role {
  ADMIN = 0x01,
  MONITOR = 0x02,
}

export interface Metric {
  id: bigint;
  threshold: bigint;
  currentValue: bigint;
}

export interface CircuitBreakerState {
  isTripped: boolean;
  tripCount: bigint;
  lastTripTime: bigint;
}

// Contract addresses (will be updated after deployment)
export const CONTRACT_ADDRESSES = {
  DetectionEngine: '0x0000000000000000000000000000000000000000' as Address,
  CircuitBreaker: '0x0000000000000000000000000000000000000000' as Address,
  AlertRegistry: '0x0000000000000000000000000000000000000000' as Address,
} as const;

export const ARBITRUM_SEPOLIA_CHAIN_ID = 421614;
