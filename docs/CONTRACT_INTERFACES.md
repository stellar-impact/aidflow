# AidFlow Contract Interfaces

**Status:** Living specification  
**Stability:** See individual function markers

---

## Overview

This document specifies the public interfaces for AidFlow Soroban smart contracts. All interfaces are designed for gas efficiency, two-key security (oracle + admin), and privacy (no PII on-chain).

## Design Principles

1. **Batch Operations:** Voucher issuance ≤100 per transaction, paginate larger batches
2. **Lazy Pull-Based Claims:** Beneficiaries pull vouchers (no push overhead)
3. **Storage Efficiency:** Store 32-byte hashes only, evidence off-chain
4. **Explicit TTL/Rent:** Per-voucher TTL management
5. **Fee Sponsorship:** Relayer service sponsors beneficiary transactions
6. **Upgradability:** `update_current_contract_wasm` behind multisig + 48-72h timelock
7. **Schema Versioning:** `schema_version` field + lazy migration on touch

---

## Config / AccessControl Contract

**Purpose:** Centralized admin multisig, oracle address, circuit breaker

**Stability:** `@stable`

### Functions

```rust
/// Initialize the contract with admin and oracle addresses
/// @stable
fn init(env: Env, admin: Address, oracle: Address);

/// Update admin address (requires current admin auth)
/// @stable
fn set_admin(env: Env, new_admin: Address);

/// Update oracle address (requires admin auth)
/// @stable
fn set_oracle(env: Env, new_oracle: Address);

/// Pause all contract operations (requires admin auth)
/// @stable
fn pause(env: Env);

/// Resume contract operations (requires admin auth)
/// @stable
fn unpause(env: Env);

/// Check if contract is paused
/// @stable
fn is_paused(env: Env) -> bool;

/// Get current admin address
/// @stable
fn get_admin(env: Env) -> Address;

/// Get current oracle address
/// @stable
fn get_oracle(env: Env) -> Address;
```

### Storage

- `ADMIN`: Address (instance storage, contract lifetime)
- `ORACLE`: Address (instance storage, contract lifetime)
- `PAUSED`: bool (persistent storage with TTL)

### Events

- `admin_updated(old_admin: Address, new_admin: Address)`
- `oracle_updated(old_oracle: Address, new_oracle: Address)`
- `paused()`
- `unpaused()`

---

## Escrow Contract

**Purpose:** Milestone-gated program funding, attestation, and release

**Stability:** `@stable`

### Functions

```rust
/// Initialize with config contract address
/// @stable
fn init(env: Env, config_contract: Address);

/// Create a new aid program with milestones
/// @stable
/// Returns program_id
fn create_program(
    env: Env,
    funder: Address,
    token: Address,
    milestones: Vec<Milestone>
) -> u64;

/// Fund a program (transfers tokens to contract)
/// @stable
fn fund(env: Env, program_id: u64, amount: i128);

/// Attest a milestone with evidence hash (requires oracle auth)
/// @stable
fn attest_milestone(
    env: Env,
    program_id: u64,
    milestone_id: u32,
    evidence_hash: BytesN<32>
);

/// Release milestone funds to voucher registry (requires admin auth)
/// @stable
fn release(env: Env, program_id: u64, milestone_id: u32);

/// Refund unspent funds to funder (requires admin auth)
/// @stable
fn refund_unspent(env: Env, program_id: u64);

/// Get program details
/// @stable
fn get_program(env: Env, program_id: u64) -> Program;
```

### Types

```rust
/// @stable
pub struct Milestone {
    pub id: u32,
    pub description: String,
    pub target_amount: i128,
}

/// @stable
pub struct Program {
    pub id: u64,
    pub funder: Address,
    pub token: Address,
    pub milestones: Vec<Milestone>,
    pub status: ProgramStatus,
    pub funded_amount: i128,
    pub released_amount: i128,
}

/// @stable
pub enum ProgramStatus {
    Active,
    Completed,
    Refunded,
}
```

### Storage

- `PROGRAM_COUNTER`: u64 (global counter)
- `PROGRAMS`: Map<u64, Program> (persistent storage with TTL)
- `ATTESTATIONS`: Map<(u64, u32), AttestationRecord> (persistent storage)

### Events

- `program_created(program_id: u64, funder: Address)`
- `funded(program_id: u64, amount: i128)`
- `milestone_attested(program_id: u64, milestone_id: u32, evidence_hash: BytesN<32>)`
- `released(program_id: u64, milestone_id: u32, amount: i128)`
- `refunded(program_id: u64, amount: i128)`

---

## VoucherRegistry Contract

**Purpose:** Batch voucher issuance, pull-based claims, expiry handling

**Stability:** `@stable`

### Functions

```rust
/// Initialize with admin, escrow contract, and token address
/// @stable
fn init(env: Env, admin: Address, escrow_contract: Address, token: Address);

/// Issue batch of vouchers (requires escrow contract auth)
/// @stable
/// Max 100 recipients per batch
/// Returns Vec<voucher_id>
fn issue_batch(
    env: Env,
    program_id: u64,
    recipients: Vec<(Address, i128)>,
    expiry: u64
) -> Vec<u64>;

/// Claim a voucher (requires recipient auth)
/// @stable
fn claim(env: Env, voucher_id: u64);

/// Expire a voucher and return funds to escrow (anyone can call after expiry)
/// @stable
fn expire(env: Env, voucher_id: u64);

/// Get voucher details
/// @stable
fn get_voucher(env: Env, voucher_id: u64) -> Voucher;
```

### Types

```rust
/// @stable
pub struct Voucher {
    pub id: u64,
    pub program_id: u64,
    pub recipient: Address,
    pub amount: i128,
    pub status: VoucherStatus,
    pub expiry: u64,
    pub claimed_at: Option<u64>,
}

/// @stable
pub enum VoucherStatus {
    Unclaimed,
    Claimed,
    Expired,
}
```

### Storage

- `VOUCHER_COUNTER`: u64 (global counter)
- `VOUCHERS`: Map<u64, Voucher> (persistent storage with TTL)
- `ESCROW_CONTRACT`: Address (instance storage)
- `TOKEN_ADDRESS`: Address (instance storage)

### Events

- `voucher_issued(voucher_id: u64, program_id: u64, recipient: Address, amount: i128)`
- `voucher_claimed(voucher_id: u64, recipient: Address, amount: i128)`
- `voucher_expired(voucher_id: u64, amount: i128)`

### Constraints

- Batch size ≤100 recipients (panics if exceeded)
- Expiry checked via `env.ledger().timestamp()`
- Token transfers via `TokenClient`

---

## MerchantRegistry Contract

**Purpose:** Merchant allowlist and voucher redemption

**Stability:** `@stable`

### Functions

```rust
/// Initialize with admin and token address
/// @stable
fn init(env: Env, admin: Address, token: Address);

/// Register a merchant (requires admin auth)
/// @stable
fn register(env: Env, merchant: Address, payout: Address, category: u32);

/// Deactivate a merchant (requires admin auth)
/// @stable
fn deactivate(env: Env, merchant: Address);

/// Redeem tokens from beneficiary to merchant (requires from auth)
/// @stable
fn redeem(env: Env, from: Address, merchant: Address, amount: i128);

/// Check if merchant is active
/// @stable
fn is_active(env: Env, merchant: Address) -> bool;
```

### Types

```rust
/// @stable
pub struct Merchant {
    pub address: Address,
    pub payout: Address,
    pub category: u32,
    pub active: bool,
}
```

### Storage

- `MERCHANTS`: Map<Address, Merchant> (persistent storage with TTL)
- `ADMIN`: Address (instance storage)
- `TOKEN_ADDRESS`: Address (instance storage)

### Events

- `merchant_registered(merchant: Address, category: u32)`
- `merchant_deactivated(merchant: Address)`
- `voucher_redeemed(from: Address, merchant: Address, amount: i128)`

### Categories

- `1` — Food
- `2` — Health
- `3` — Education
- `4` — Agriculture
- `5` — Other

---

## Upgrade Strategy

### Contract Upgrades

- Use `update_current_contract_wasm` operation
- Requires admin multisig authorization
- Enforce 48-72h timelock before applying
- Emit `upgrade_scheduled(new_wasm_hash, execution_time)` event

### Schema Versioning

- Store `schema_version: u32` in each contract
- On version mismatch, trigger lazy migration on next read/write
- Example: v1 → v2 adds new field with default value

### Migration Testing

- Deploy new version to testnet first
- Run migration scripts
- Verify data integrity
- Monitor for 1 week before mainnet

---

## Gas Optimization Patterns

1. **Batch Operations:** Group multiple vouchers into single transaction
2. **Lazy Loading:** Load only required fields from storage
3. **Event Compression:** Emit minimal data, reference by ID
4. **TTL Management:** Extend TTL only when necessary
5. **Token Client Reuse:** Cache TokenClient instances

---

## Security Considerations

### Access Control

- **Admin functions:** Require admin multisig auth (via Config contract)
- **Oracle functions:** Require oracle keypair auth
- **User functions:** Require caller's auth (`from.require_auth()`)

### Re-entrancy Protection

- Soroban SDK provides re-entrancy protection by default
- No external calls after state mutations

### Integer Overflow

- Soroban SDK uses checked arithmetic (panics on overflow)
- Amounts are `i128` (sufficient for token decimals)

---

## Related Documents

- [Architecture](ARCHITECTURE.md)
- [Extraction Plan](EXTRACTION_PLAN.md)
- [Soroban SDK Docs](https://docs.rs/soroban-sdk/26.1.0/)
