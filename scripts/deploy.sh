#!/bin/bash
# ArbiShield Deployment Script
#
# This script deploys all three ArbiShield contracts to the specified network.
# Make sure to configure your .env file before running this script.

set -e

echo "🛡️  ArbiShield Deployment Script"
echo "=================================="
echo

# Check if .env file exists
if [ ! -f .env ]; then
    echo "❌ Error: .env file not found!"
    echo "Please copy .env.example to .env and configure it."
    exit 1
fi

# Load environment variables
source .env

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v cargo-stylus &> /dev/null; then
    echo "❌ Error: cargo-stylus not found!"
    echo "Install it with: cargo install --force cargo-stylus"
    exit 1
fi

echo "✅ cargo-stylus found"

# Check if WASM target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo "❌ Error: wasm32-unknown-unknown target not installed!"
    echo "Install it with: rustup target add wasm32-unknown-unknown"
    exit 1
fi

echo "✅ WASM target installed"
echo

# Validate contracts
echo "🔍 Validating contracts..."
cargo stylus check

if [ $? -ne 0 ]; then
    echo "❌ Contract validation failed!"
    exit 1
fi

echo "✅ All contracts validated"
echo

# Deploy contracts
echo "🚀 Deploying contracts..."
echo

# Check if RPC_URL and PRIVATE_KEY are set
if [ -z "$RPC_URL" ]; then
    echo "❌ Error: RPC_URL not set in .env"
    exit 1
fi

if [ -z "$PRIVATE_KEY" ]; then
    echo "❌ Error: PRIVATE_KEY not set in .env"
    exit 1
fi

echo "1️⃣  Deploying DetectionEngine..."
echo "   Estimating gas..."
cargo stylus deploy \
    --endpoint="$RPC_URL" \
    --private-key="$PRIVATE_KEY" \
    --estimate-gas

echo
read -p "   Proceed with DetectionEngine deployment? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cargo stylus deploy \
        --endpoint="$RPC_URL" \
        --private-key="$PRIVATE_KEY"
    echo "✅ DetectionEngine deployed"
else
    echo "⏭️  Skipped DetectionEngine deployment"
fi

echo
echo "2️⃣  Deploying CircuitBreaker..."
echo "   Estimating gas..."
cargo stylus deploy \
    --endpoint="$RPC_URL" \
    --private-key="$PRIVATE_KEY" \
    --estimate-gas

echo
read -p "   Proceed with CircuitBreaker deployment? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cargo stylus deploy \
        --endpoint="$RPC_URL" \
        --private-key="$PRIVATE_KEY"
    echo "✅ CircuitBreaker deployed"
else
    echo "⏭️  Skipped CircuitBreaker deployment"
fi

echo
echo "3️⃣  Deploying AlertRegistry..."
echo "   Estimating gas..."
cargo stylus deploy \
    --endpoint="$RPC_URL" \
    --private-key="$PRIVATE_KEY" \
    --estimate-gas

echo
read -p "   Proceed with AlertRegistry deployment? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cargo stylus deploy \
        --endpoint="$RPC_URL" \
        --private-key="$PRIVATE_KEY"
    echo "✅ AlertRegistry deployed"
else
    echo "⏭️  Skipped AlertRegistry deployment"
fi

echo
echo "🎉 Deployment complete!"
echo
echo "📝 Remember to update your .env file with the deployed contract addresses."
