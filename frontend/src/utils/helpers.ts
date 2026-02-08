import { Priority } from '../types/contracts';
import { clsx, type ClassValue } from 'clsx';
import { twMerge } from 'tailwind-merge';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatAddress(address: string): string {
  if (!address) return '';
  return `${address.slice(0, 6)}...${address.slice(-4)}`;
}

export function formatTimestamp(timestamp: bigint): string {
  const date = new Date(Number(timestamp) * 1000);
  return date.toLocaleString();
}

export function formatRelativeTime(timestamp: bigint): string {
  const now = Date.now();
  const time = Number(timestamp) * 1000;
  const diff = now - time;

  const minutes = Math.floor(diff / 60000);
  const hours = Math.floor(diff / 3600000);
  const days = Math.floor(diff / 86400000);

  if (minutes < 1) return 'just now';
  if (minutes < 60) return `${minutes}m ago`;
  if (hours < 24) return `${hours}h ago`;
  return `${days}d ago`;
}

export function getPriorityLabel(priority: number): string {
  switch (priority) {
    case Priority.LOW:
      return 'Low';
    case Priority.MEDIUM:
      return 'Medium';
    case Priority.HIGH:
      return 'High';
    case Priority.CRITICAL:
      return 'Critical';
    default:
      return 'Unknown';
  }
}

export function getPriorityBadgeClass(priority: number): string {
  switch (priority) {
    case Priority.LOW:
      return 'badge-low';
    case Priority.MEDIUM:
      return 'badge-medium';
    case Priority.HIGH:
      return 'badge-high';
    case Priority.CRITICAL:
      return 'badge-critical';
    default:
      return '';
  }
}

export function formatNumber(value: bigint | number): string {
  return new Intl.NumberFormat('en-US').format(Number(value));
}

export function calculatePercentage(value: bigint, threshold: bigint): number {
  if (threshold === 0n) return 0;
  return Number((value * 100n) / threshold);
}
