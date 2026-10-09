#!/bin/bash
# Deploy Config + Escrow + VoucherRegistry + MerchantRegistry to Stellar testnet from a local machine.
# Usage: ./scripts/deploy-testnet.sh
# Requires stellar-cli 27.x. Creates/funds testnet-only identities if missing.
set -euo pipefail

cd "$(dirname "$0")/.."

NETWORK=testnet
DEPLOYER=aidflow-deployer
ORACLE=aidflow-oracle

./scripts/build.sh >/dev/null

for id in "$DEPLOYER" "$ORACLE"; do
  if ! stellar keys address "$id" >/dev/null 2>&1; then
    stellar keys generate "$id" --network "$NETWORK" --fund
  fi
done

ADMIN_ADDR=$(stellar keys address "$DEPLOYER")
ORACLE_ADDR=$(stellar keys address "$ORACLE")

# Native XLM asset contract stands in for USDC on testnet.
TOKEN_ID=$(stellar contract id asset --asset native --network "$NETWORK")

CONFIG_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/aidflow_config.wasm \
  --source "$DEPLOYER" --network "$NETWORK")

stellar contract invoke --id "$CONFIG_ID" --source "$DEPLOYER" --network "$NETWORK" \
  -- init --admin "$ADMIN_ADDR" --oracle "$ORACLE_ADDR"

ESCROW_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/aidflow_escrow.wasm \
  --source "$DEPLOYER" --network "$NETWORK")

REGISTRY_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/aidflow_voucher_registry.wasm \
  --source "$DEPLOYER" --network "$NETWORK")

MERCHANT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/aidflow_merchant_registry.wasm \
  --source "$DEPLOYER" --network "$NETWORK")

# Escrow and registry reference each other, so both are deployed before either
# is initialized.
stellar contract invoke --id "$ESCROW_ID" --source "$DEPLOYER" --network "$NETWORK" \
  -- init --config_contract "$CONFIG_ID" --token "$TOKEN_ID" --voucher_registry "$REGISTRY_ID"

stellar contract invoke --id "$REGISTRY_ID" --source "$DEPLOYER" --network "$NETWORK" \
  -- init --config_contract "$CONFIG_ID" --escrow_contract "$ESCROW_ID" --token "$TOKEN_ID"

stellar contract invoke --id "$MERCHANT_ID" --source "$DEPLOYER" --network "$NETWORK" \
  -- init --config_contract "$CONFIG_ID" --token "$TOKEN_ID"

echo "Config:  $CONFIG_ID"
echo "Escrow:  $ESCROW_ID"
echo "Registry: $REGISTRY_ID"
echo "Merchant: $MERCHANT_ID"
echo "Token:   $TOKEN_ID"
echo "Admin:   $ADMIN_ADDR"
echo "Oracle:  $ORACLE_ADDR"
echo "Explorer: https://stellar.expert/explorer/testnet/contract/$ESCROW_ID"
