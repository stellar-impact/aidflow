# Config Contract

**Stability:** `@stable`

## Purpose

The Config contract serves as the centralized access control and circuit breaker for the entire AidFlow system. It maintains the admin multisig address, oracle address, and a global pause flag that all other contracts query at call time.

## Public Functions

### Initialization

- `init(admin: Address, oracle: Address)` — Initialize the contract with admin and oracle addresses. Can only be called once. Requires admin authorization.

### Access Control

- `set_admin(new_admin: Address)` — Update the admin address. Requires current admin auth.
- `set_oracle(new_oracle: Address)` — Update the oracle address. Requires admin auth.
- `get_admin() -> Address` — Query the current admin address.
- `get_oracle() -> Address` — Query the current oracle address.

### Circuit Breaker

- `pause()` — Pause all contract operations system-wide. Requires admin auth.
- `unpause()` — Resume contract operations. Requires admin auth.
- `is_paused() -> bool` — Check if the system is paused.

## Storage Keys and TTL

All data is stored in **instance storage** with a 30-day TTL:

- `Admin` — The admin multisig address
- `Oracle` — The oracle address
- `Paused` — Boolean pause flag

TTL is bumped on every state change (`init`, `set_admin`, `set_oracle`, `pause`, `unpause`); the read-only getters do not bump it (threshold: ~23 days, extend to: ~30 days).

## Events

- `admin_updated(old_admin, new_admin)` — Emitted when admin changes
- `oracle_updated(old_oracle, new_oracle)` — Emitted when oracle changes
- `paused()` — Emitted when system is paused
- `unpaused()` — Emitted when system is unpaused

## Cross-Contract Dependencies

The Config contract is called **by** all other contracts:

- **Escrow** calls `get_admin()`, `get_oracle()`, `is_paused()`
- **VoucherRegistry** calls `get_admin()`, `is_paused()`
- **MerchantRegistry** calls `get_admin()`, `is_paused()`

This creates a single source of truth for authorization and emergency shutdown.

## Testnet Deployment

See the [Testnet Deployment](../../README.md#testnet-deployment) table in the root README for the current contract ID.

## References

- [CONTRACT_INTERFACES.md](../../docs/CONTRACT_INTERFACES.md#config--accesscontrol-contract) — Full API specification
- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) — System design
