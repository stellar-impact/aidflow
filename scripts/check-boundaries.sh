#!/bin/bash
#
# check-boundaries.sh - Enforce dependency direction in AidFlow monorepo
#
# STRICT DEPENDENCY RULES:
# 1. contracts/ must NEVER import from services/ or clients/
# 2. services/ and clients/ must NEVER use relative paths into contracts/
#    (they must use published packages only)
# 3. services/ must NEVER import from clients/
#
# This script runs in CI on every PR to prevent violations.
#

set -e

echo "🔍 Checking dependency boundaries..."
echo ""

ERRORS=0

# Check 1: contracts/ must not import services/ or clients/
echo "Check 1: contracts/ isolation..."
if grep -r "use.*services::" contracts/ 2>/dev/null | grep -v target | grep -v ".git"; then
    echo "❌ ERROR: contracts/ imports from services/"
    echo "   Contracts must depend only on soroban-sdk"
    ERRORS=$((ERRORS + 1))
elif grep -r "use.*clients::" contracts/ 2>/dev/null | grep -v target | grep -v ".git"; then
    echo "❌ ERROR: contracts/ imports from clients/"
    echo "   Contracts must depend only on soroban-sdk"
    ERRORS=$((ERRORS + 1))
elif grep -r "import.*['\"].*services/" contracts/sdk/ 2>/dev/null | grep -v node_modules | grep -v ".git"; then
    echo "❌ ERROR: contracts/sdk/ imports from services/"
    echo "   Contract SDK must be standalone"
    ERRORS=$((ERRORS + 1))
else
    echo "✓ contracts/ is isolated (no imports from services or clients)"
fi
echo ""

# Check 2: services/ and clients/ must not use relative paths into contracts/
echo "Check 2: no relative paths into contracts/..."
if [ -d "services" ]; then
    if grep -r "use.*\.\.\/\.\.\/contracts/" services/ 2>/dev/null | grep -v target | grep -v ".git"; then
        echo "❌ ERROR: services/ uses relative path into contracts/"
        echo "   Use published package 'aidflow-contract-types' instead"
        ERRORS=$((ERRORS + 1))
    else
        echo "✓ services/ uses published packages (no relative paths into contracts/)"
    fi
fi
echo ""

if [ -d "clients" ]; then
    if grep -r "import.*['\"].*\.\.\/\.\.\/contracts/" clients/ 2>/dev/null | grep -v node_modules | grep -v ".git"; then
        echo "❌ ERROR: clients/ uses relative path into contracts/"
        echo "   Use published package '@aidflow/contract-sdk' instead"
        ERRORS=$((ERRORS + 1))
    else
        echo "✓ clients/ uses published packages (no relative paths into contracts/)"
    fi
fi
echo ""

# Check 3: services/ must not import from clients/
echo "Check 3: services/ must not import clients/..."
if [ -d "services" ]; then
    if grep -r "import.*['\"].*clients/" services/ 2>/dev/null | grep -v ".git"; then
        echo "❌ ERROR: services/ imports from clients/"
        echo "   Services must not depend on client code"
        ERRORS=$((ERRORS + 1))
    else
        echo "✓ services/ does not import clients/"
    fi
fi
echo ""

# Check 4: Verify Go SDK uses new import path (not deprecated)
echo "Check 4: Go SDK import path..."
if [ -d "services" ]; then
    if grep -r 'import.*"github.com/stellar/go/' services/ 2>/dev/null | grep -v ".git" | grep -v "go-stellar-sdk"; then
        echo "❌ ERROR: services/ uses DEPRECATED Go SDK import path"
        echo "   Found: github.com/stellar/go/"
        echo "   Use:   github.com/stellar/go-stellar-sdk/"
        ERRORS=$((ERRORS + 1))
    else
        echo "✓ services/ uses correct Go SDK import path (github.com/stellar/go-stellar-sdk)"
    fi
fi
echo ""

# Summary
if [ $ERRORS -eq 0 ]; then
    echo "✅ All boundary checks passed!"
    echo ""
    echo "Dependency direction is correct:"
    echo "  contracts/ (isolated)"
    echo "      ↓"
    echo "  services/ (via published packages)"
    echo "      ↓"
    echo "  clients/ (via published packages + HTTP)"
    exit 0
else
    echo "❌ Boundary check failed with $ERRORS error(s)"
    echo ""
    echo "Dependency violations detected. Please fix before merging."
    echo ""
    echo "Rules:"
    echo "  1. contracts/ must not import services/ or clients/"
    echo "  2. services/clients/ must use published packages, not relative paths"
    echo "  3. services/ must not import clients/"
    echo "  4. Use github.com/stellar/go-stellar-sdk (NOT github.com/stellar/go)"
    echo ""
    echo "See docs/ARCHITECTURE.md for dependency direction rules."
    exit 1
fi
