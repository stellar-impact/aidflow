# VoucherRegistry Contract

**Stability:** `@stable`

## Purpose

The VoucherRegistry contract custodies released milestone funds and converts them into per-beneficiary vouchers through admin-gated batch issuance, pull-based claims, and permissionless expiry that returns unclaimed funds to Escrow.

## Public Functions

### Initialization

- `init(config_contract: Address, escrow_contract: Address, token: Address)` — Initialize with Config, Escrow, and settlement token addresses. Requires Config admin auth.

### Voucher Lifecycle

- `issue_batch(program_id: u64, recipients: Vec<(Address, i128)>, expiry: u64) -> Vec<u64>` — Issue up to 40 vouchers at once. Returns voucher IDs. Requires admin auth. Enforces solvency: `outstanding + batch_total <= registry_balance`.
- `claim(voucher_id: u64)` — Claim a voucher (pays recipient). Requires recipient auth. Must be called before expiry.
- `expire(voucher_id: u64)` — Expire an unclaimed voucher after its deadline and return funds to Escrow. Anyone may call.
- `get_voucher(voucher_id: u64) -> Voucher` — Query voucher details.
- `get_outstanding() -> i128` — Sum of all Unclaimed voucher amounts.

## Storage Keys and TTL

### Instance Storage (30-day TTL)

- `Config` — Config contract address
- `Escrow` — Escrow contract address
- `Token` — Settlement token address
- `VoucherCounter` — Global voucher counter
- `Outstanding` — Sum of Unclaimed voucher amounts

### Persistent Storage (30/37-day TTL)

- `Voucher(u64)` — Voucher records by ID

## Events

- `voucher_issued(voucher_id, program_id, recipient, amount)` — Voucher created
- `voucher_claimed(voucher_id, recipient, amount)` — Voucher claimed by beneficiary
- `voucher_expired(voucher_id, amount)` — Voucher expired and funds returned

## Cross-Contract Calls

**Calls TO this contract:**

- Escrow transfers funds here on `release()`
- Admin calls `issue_batch()` to convert released funds into vouchers

**Calls FROM this contract:**

- Reads `get_admin()`, `is_paused()` from **Config**
- Calls `reclaim_expired()` on **Escrow** when a voucher expires

## Solvency Guarantee

The registry tracks `outstanding` (sum of Unclaimed voucher amounts) and enforces:

```
outstanding + new_batch_total <= registry_token_balance
```

This prevents over-issuance: the registry can never promise more tokens than it actually holds.

## Batch Size Limit

Maximum 40 recipients per `issue_batch()` call. This leaves headroom within Soroban's 50-ledger-write-per-transaction limit (each voucher is one persistent write).

## Testnet Deployment

See the [Testnet Deployment](../../README.md#testnet-deployment) table in the root README for the current contract ID.

## References

- [CONTRACT_INTERFACES.md](../../docs/CONTRACT_INTERFACES.md#voucherregistry-contract) — Full API specification
- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) — System design
