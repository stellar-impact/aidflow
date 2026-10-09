# Contract Types

**Stability:** `@stable`

## Purpose

The `aidflow-contract-types` crate defines shared, stable data types used across all AidFlow Soroban contracts. This keeps type definitions centralized and ensures consistent encoding across contract boundaries.

## Exported Types

### Milestone

Represents a single milestone in an aid program.

```rust
pub struct Milestone {
    pub id: u32,
    pub description: String,
    pub target_amount: i128,
}
```

### Program Status

Status of an aid program.

```rust
pub enum ProgramStatus {
    Active,      // Program is active and accepting funds/releases
    Completed,   // All milestones completed
    Refunded,    // Program cancelled, unspent funds refunded
}
```

### Voucher

A voucher issued to a beneficiary, redeemable until expiry.

```rust
pub struct Voucher {
    pub id: u64,
    pub program_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub status: VoucherStatus,
    pub expiry: u64,
    pub claimed_at: Option<u64>,
}
```

### Voucher Status

```rust
pub enum VoucherStatus {
    Unclaimed,   // Voucher issued but not yet claimed
    Claimed,     // Voucher claimed by beneficiary
    Expired,     // Voucher expired, funds returned to escrow
}
```

### Merchant

A merchant approved to accept voucher redemptions.

```rust
pub struct Merchant {
    pub address: Address,
    pub payout: Address,
    pub category: u32,
    pub active: bool,
}
```

### Merchant Categories (Constants)

```rust
pub const CATEGORY_FOOD: u32 = 1;
pub const CATEGORY_HEALTH: u32 = 2;
pub const CATEGORY_EDUCATION: u32 = 3;
pub const CATEGORY_AGRICULTURE: u32 = 4;
pub const CATEGORY_OTHER: u32 = 5;
```

## Usage

All contracts import types from this crate:

```rust
use aidflow_contract_types::{Milestone, ProgramStatus, Voucher, VoucherStatus, Merchant};
```

## Stability Guarantees

All types are marked `@stable` and will not change their wire format without a major version bump. This ensures that contracts built against different versions of the types crate remain compatible on-chain.

## Publishing

This crate will be published to crates.io as `aidflow-contract-types` after audit, allowing independent versioning from the rest of the workspace.

## References

- [CONTRACT_INTERFACES.md](../../docs/CONTRACT_INTERFACES.md) — API specifications using these types
- [EXTRACTION_PLAN.md](../../docs/EXTRACTION_PLAN.md) — Post-audit extraction strategy
