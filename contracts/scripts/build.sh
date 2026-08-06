#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "🔨 Building AidFlow contracts with stellar-cli 27.x..."
echo ""

# Build config contract
echo "Building aidflow-config..."
stellar contract build --package aidflow-config

# Add more packages as they're created
# echo "Building aidflow-escrow..."
# stellar contract build --package aidflow-escrow

echo ""
echo "✅ Contracts built successfully!"
echo ""
echo "WASM outputs:"
ls -lh target/wasm32-unknown-unknown/release/*.wasm 2>/dev/null || echo "  (No WASM files yet)"
