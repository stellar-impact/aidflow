#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "🔨 Building AidFlow contracts with stellar-cli 27.x..."
echo ""

# Build config contract
echo "Building aidflow-config..."
stellar contract build --package aidflow-config

# Build voucher registry
echo "Building aidflow-voucher-registry..."
stellar contract build --package aidflow-voucher-registry

# Build merchant registry
echo "Building aidflow-merchant-registry..."
stellar contract build --package aidflow-merchant-registry

# Build escrow contract
echo "Building aidflow-escrow..."
stellar contract build --package aidflow-escrow

echo ""
echo "✅ Contracts built successfully!"
echo ""
echo "WASM outputs:"
ls -lh target/wasm32v1-none/release/*.wasm 2>/dev/null || echo "  (No WASM files yet)"
