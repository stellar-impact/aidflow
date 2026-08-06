#!/bin/bash
set -e

cd "$(dirname "$0")/.."

echo "🧪 Running contract tests..."
echo ""

cargo test --all

echo ""
echo "✅ All tests passed!"
