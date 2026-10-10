# MerchantRegistry Contract

**Stability:** `@stable`

## Purpose

The MerchantRegistry contract maintains an admin-managed allowlist of merchants and provides the redemption path through which beneficiaries spend settlement tokens. The registry never custodies funds — redemptions transfer directly from beneficiary to merchant's payout address.

## Public Functions

### Initialization

- `init(config_contract: Address, token: Address)` — Initialize with Config and settlement token addresses. Requires Config admin auth.

### Merchant Management

- `register(merchant: Address, payout: Address, category: u32)` — Register or update a merchant. Upsert: re-registering updates payout/category and reactivates. Category must be 1..=5. Requires admin auth.
- `deactivate(merchant: Address)` — Mark merchant inactive (prevents redemptions). Requires admin auth. **Allowed while paused** (risk reduction only).
- `is_active(merchant: Address) -> bool` — Check if merchant is registered and active.
- `get_merchant(merchant: Address) -> Merchant` — Query merchant details.

### Redemption

- `redeem(from: Address, merchant: Address, amount: i128)` — Transfer tokens from beneficiary to merchant's payout address. Requires `from` auth. Rejects inactive/unregistered merchants.

## Storage Keys and TTL

### Instance Storage (30-day TTL)

- `Config` — Config contract address
- `Token` — Settlement token address

### Persistent Storage (30/37-day TTL)

- `Merchant(Address)` — Merchant records by address

## Events

- `merchant_registered(merchant, category)` — Merchant added or updated
- `merchant_deactivated(merchant)` — Merchant marked inactive
- `voucher_redeemed(from, merchant, amount)` — Beneficiary paid merchant

## Merchant Categories

Defined in `aidflow-contract-types`:

1. **Food** — `CATEGORY_FOOD`
2. **Health** — `CATEGORY_HEALTH`
3. **Education** — `CATEGORY_EDUCATION`
4. **Agriculture** — `CATEGORY_AGRICULTURE`
5. **Other** — `CATEGORY_OTHER`

## Cross-Contract Calls

**Calls FROM this contract:**

- Reads `get_admin()`, `is_paused()` from **Config**
- Transfers tokens from beneficiary to merchant payout (direct, no custody)

## Design Decisions

1. **No Custody:** The registry never holds tokens. `redeem()` transfers directly from beneficiary to `merchant.payout`.
2. **Upsert Registration:** Re-registering a merchant updates its payout/category and reactivates it (there is no separate `reactivate` function).
3. **Deactivate Allowed While Paused:** Removing a merchant reduces risk, so `deactivate()` works even when the system is paused.

## Testnet Deployment

Contract ID: [`CCQPQFR762BJ5GLMCR7P4BEM4PEHPD5UVRSGSJBWQ5PM6GFXK33L5OPP`](https://stellar.expert/explorer/testnet/contract/CCQPQFR762BJ5GLMCR7P4BEM4PEHPD5UVRSGSJBWQ5PM6GFXK33L5OPP)

## References

- [CONTRACT_INTERFACES.md](../../docs/CONTRACT_INTERFACES.md#merchantregistry-contract) — Full API specification
- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) — System design
