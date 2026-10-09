# Escrow Contract

**Stability:** `@stable`

## Purpose

The Escrow contract implements milestone-gated program funding with a two-key trust model. Funds for each milestone are released only after independent field verification (oracle attestation) **and** admin authorization, preventing any single key from unilaterally moving funds.

## Public Functions

### Initialization

- `init(config_contract: Address, token: Address, voucher_registry: Address)` — Initialize with Config, settlement token (USDC), and VoucherRegistry addresses. Requires Config admin auth.

### Program Lifecycle

- `create_program(funder: Address, token: Address, milestones: Vec<Milestone>) -> u64` — Create a new aid program. Returns the program ID. Requires funder auth.
- `fund(program_id: u64, amount: i128)` — Add funds to a program. Requires funder auth.
- `get_program(program_id: u64) -> Program` — Query program details.

### Two-Key Release Flow

- `attest_milestone(program_id: u64, milestone_id: u32, evidence_hash: BytesN<32>)` — Oracle attests that a milestone's evidence is valid. Requires oracle auth.
- `release(program_id: u64, milestone_id: u32)` — Release milestone funds to VoucherRegistry. Requires admin auth **and** prior attestation.

### Refund & Reclaim

- `refund_unspent(program_id: u64)` — Return unspent funds (funded - released) to funder. Admin-only, callable only when program status is `Completed`.
- `reclaim_expired(program_id: u64, amount: i128)` — Accept funds returned by VoucherRegistry when vouchers expire. Requires VoucherRegistry auth. Re-credits the program or forwards to funder if already refunded.
- `get_attestation(program_id: u64, milestone_id: u32) -> Option<AttestationRecord>` — Query attestation details.

## Storage Keys and TTL

### Instance Storage (30-day TTL)

- `Config` — Config contract address
- `Token` — Settlement token address
- `VoucherRegistry` — VoucherRegistry address
- `ProgramCounter` — Global program counter

### Persistent Storage (30/37-day TTL)

- `Program(u64)` — Program records by ID
- `Attestation(u64, u32)` — Attestation records by (program_id, milestone_id)
- `Released(u64, u32)` — Per-milestone release flags

## Events

- `program_created(program_id, funder)` — New program created
- `funded(program_id, amount)` — Program received funds
- `milestone_attested(program_id, milestone_id, evidence_hash)` — Oracle attested milestone
- `released(program_id, milestone_id, amount)` — Funds released to VoucherRegistry
- `refunded(program_id, amount)` — Unspent funds returned to funder
- `reclaimed(program_id, amount)` — Expired voucher funds reclaimed

## Cross-Contract Calls

**Calls TO this contract:**

- VoucherRegistry calls `reclaim_expired()` when a voucher expires

**Calls FROM this contract:**

- Reads `get_admin()`, `get_oracle()`, `is_paused()` from **Config**
- Transfers tokens to **VoucherRegistry** on `release()`

## Invariants

1. Each milestone releases **at most once** (per-milestone `Released` flag)
2. Release requires **prior attestation** (two-key model)
3. **Checks-Effects-Interactions** ordering around all token transfers
4. **Checked arithmetic** on all balance operations
5. Pause blocks **all** state-changing operations (including `attest_milestone`)

## Testnet Deployment

Contract ID: [`CBF5SA4EAHZOYSMARBFBFOH7KJKH237LKHPHIOSVDHQ7HYL5KG3EWLVP`](https://stellar.expert/explorer/testnet/contract/CBF5SA4EAHZOYSMARBFBFOH7KJKH237LKHPHIOSVDHQ7HYL5KG3EWLVP)

## References

- [CONTRACT_INTERFACES.md](../../docs/CONTRACT_INTERFACES.md#escrow-contract) — Full API specification
- [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) — System design
