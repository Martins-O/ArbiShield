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

// Contract addresses (deployed on Arbitrum Sepolia)
export const CONTRACT_ADDRESSES = {
  DetectionEngine: '0x700f33B8fECc0eC61cFAA44F8e6c9BdF751C4e9E' as Address,
  CircuitBreaker: '0x91b3611115ABc980BD1a4F8E16ad4EBDe61aB07d' as Address,
  AlertRegistry: '0x02c5084D7fC06cE22746716df892Cde3E05Bb8E1' as Address,
} as const;

export const ARBITRUM_SEPOLIA_CHAIN_ID = 421614;
