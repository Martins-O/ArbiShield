# ArbiShield Frontend

React + TypeScript frontend for ArbiShield smart contract security monitoring platform.

## Features

- 🔍 **Detection Engine** - Monitor metrics and detect anomalies
- ⚡ **Circuit Breaker** - Emergency system pause mechanism
- 🚨 **Alert Registry** - View and manage security alerts with priority levels
- 🌐 **Web3 Integration** - Connect wallet via RainbowKit
- 📊 **Real-time Updates** - Live contract state monitoring

## Tech Stack

- **React 18** with TypeScript
- **Vite** for fast development
- **Tailwind CSS** for styling
- **Wagmi v2** + **Viem** for Ethereum interactions
- **RainbowKit** for wallet connection
- **React Router** for navigation

## Getting Started

### Prerequisites

- Node.js 18+ or pnpm/yarn

### Installation

```bash
cd frontend
pnpm install
```

### Development

```bash
pnpm dev
```

Frontend will run at `http://localhost:3000`

### Build

```bash
pnpm build
```

## Configuration

Update contract addresses in `src/types/contracts.ts` after deployment:

```typescript
export const CONTRACT_ADDRESSES = {
  DetectionEngine: '0x...',
  CircuitBreaker: '0x...',
  AlertRegistry: '0x...',
};
```

Get a WalletConnect Project ID at https://cloud.walletconnect.com and update `src/config/wagmi.ts`.

## Project Structure

```
src/
├── components/          # React components
│   ├── AlertRegistry/   # Alert management UI
│   ├── CircuitBreaker/  # Emergency controls
│   ├── DetectionEngine/ # Metric monitoring
│   └── Dashboard.tsx    # Main layout
├── config/              # Configuration
│   ├── abis.ts          # Contract ABIs
│   └── wagmi.ts         # Web3 config
├── hooks/               # Custom hooks
├── types/               # TypeScript types
├── utils/               # Helper functions
└── App.tsx              # App entry point
```

## Features

### Detection Engine
- Register new metrics (owner only)
- Report metric values
- View anomaly detection status
- Real-time threshold monitoring

### Circuit Breaker
- View system status (tripped/active)
- Trip circuit (owner only)
- Reset circuit (owner only)
- Track trip history

### Alert Registry
- View all security alerts
- Filter by priority (LOW/MEDIUM/HIGH/CRITICAL)
- Acknowledge alerts
- Track threat levels
- Real-time notifications

## License

MIT OR Apache-2.0
