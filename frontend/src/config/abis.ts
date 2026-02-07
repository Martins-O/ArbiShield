// ArbiShield Contract ABIs
// These will be generated from the actual deployed contracts

export const DetectionEngineABI = [
  {
    type: 'function',
    name: 'registerMetric',
    inputs: [
      { name: 'metricId', type: 'uint256' },
      { name: 'threshold', type: 'uint256' },
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'reportMetric',
    inputs: [
      { name: 'metricId', type: 'uint256' },
      { name: 'value', type: 'uint256' },
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'checkAnomaly',
    inputs: [{ name: 'metricId', type: 'uint256' }],
    outputs: [
      { name: 'isAnomaly', type: 'bool' },
      { name: 'severity', type: 'uint256' },
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'owner',
    inputs: [],
    outputs: [{ name: '', type: 'address' }],
    stateMutability: 'view',
  },
  {
    type: 'event',
    name: 'MetricRegistered',
    inputs: [
      { name: 'metricId', type: 'uint256', indexed: true },
      { name: 'threshold', type: 'uint256', indexed: false },
    ],
  },
  {
    type: 'event',
    name: 'MetricReported',
    inputs: [
      { name: 'metricId', type: 'uint256', indexed: true },
      { name: 'value', type: 'uint256', indexed: false },
      { name: 'anomaly', type: 'bool', indexed: false },
      { name: 'severity', type: 'uint256', indexed: false },
    ],
  },
] as const;

export const CircuitBreakerABI = [
  {
    type: 'function',
    name: 'trip',
    inputs: [],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'reset',
    inputs: [],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'isTripped',
    inputs: [],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'getTripCount',
    inputs: [],
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'getLastTripTime',
    inputs: [],
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'owner',
    inputs: [],
    outputs: [{ name: '', type: 'address' }],
    stateMutability: 'view',
  },
  {
    type: 'event',
    name: 'CircuitTripped',
    inputs: [
      { name: 'tripCount', type: 'uint256', indexed: false },
      { name: 'timestamp', type: 'uint256', indexed: false },
    ],
  },
  {
    type: 'event',
    name: 'CircuitReset',
    inputs: [{ name: 'timestamp', type: 'uint256', indexed: false }],
  },
] as const;

export const AlertRegistryABI = [
  {
    type: 'function',
    name: 'registerAlert',
    inputs: [
      { name: 'source', type: 'address' },
      { name: 'timestamp', type: 'uint256' },
      { name: 'message', type: 'string' },
      { name: 'threatLevel', type: 'uint256' },
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'acknowledgeAlert',
    inputs: [{ name: 'alertId', type: 'uint256' }],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'getAlert',
    inputs: [{ name: 'alertId', type: 'uint256' }],
    outputs: [
      {
        name: '',
        type: 'tuple',
        components: [
          { name: 'id', type: 'uint256' },
          { name: 'source', type: 'address' },
          { name: 'timestamp', type: 'uint256' },
          { name: 'message', type: 'string' },
          { name: 'priority', type: 'uint256' },
          { name: 'threatLevel', type: 'uint256' },
          { name: 'acknowledged', type: 'bool' },
          { name: 'acknowledger', type: 'address' },
          { name: 'ackTimestamp', type: 'uint256' },
          { name: 'expired', type: 'bool' },
        ],
      },
    ],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'getAlertCount',
    inputs: [],
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'getPriorityCount',
    inputs: [{ name: 'priority', type: 'uint256' }],
    outputs: [{ name: '', type: 'uint256' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'subscribe',
    inputs: [],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'unsubscribe',
    inputs: [],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'grantRole',
    inputs: [
      { name: 'account', type: 'address' },
      { name: 'role', type: 'uint256' },
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'revokeRole',
    inputs: [
      { name: 'account', type: 'address' },
      { name: 'role', type: 'uint256' },
    ],
    outputs: [],
    stateMutability: 'nonpayable',
  },
  {
    type: 'function',
    name: 'hasRole',
    inputs: [
      { name: 'account', type: 'address' },
      { name: 'role', type: 'uint256' },
    ],
    outputs: [{ name: '', type: 'bool' }],
    stateMutability: 'view',
  },
  {
    type: 'function',
    name: 'owner',
    inputs: [],
    outputs: [{ name: '', type: 'address' }],
    stateMutability: 'view',
  },
  {
    type: 'event',
    name: 'AlertRegistered',
    inputs: [
      { name: 'id', type: 'uint256', indexed: true },
      { name: 'source', type: 'address', indexed: true },
      { name: 'threatLevel', type: 'uint256', indexed: false },
      { name: 'priority', type: 'uint256', indexed: false },
    ],
  },
  {
    type: 'event',
    name: 'AlertAcknowledged',
    inputs: [
      { name: 'id', type: 'uint256', indexed: true },
      { name: 'acknowledger', type: 'address', indexed: true },
      { name: 'timestamp', type: 'uint256', indexed: false },
    ],
  },
  {
    type: 'event',
    name: 'RoleGranted',
    inputs: [
      { name: 'account', type: 'address', indexed: true },
      { name: 'role', type: 'uint256', indexed: false },
      { name: 'granter', type: 'address', indexed: true },
    ],
  },
  {
    type: 'event',
    name: 'Subscribed',
    inputs: [{ name: 'subscriber', type: 'address', indexed: true }],
  },
] as const;
