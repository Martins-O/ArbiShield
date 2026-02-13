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

// Contract addresses (deployed on Arbitrum Sepolia - 2026-02-13)
export const CONTRACT_ADDRESSES = {
  DetectionEngine: '0x833468151FF5b1f31AFa8D2E4876e4E76ADBDD8F' as Address,
  CircuitBreaker: '0x62e8F692094506831790DA100040Bd17FfE158F0' as Address,
  AlertRegistry: '0x454d2F7c4b4bCF5034B0CeE9C50bbF41F2168BB3' as Address,
} as const;

export const ARBITRUM_SEPOLIA_CHAIN_ID = 421614;
