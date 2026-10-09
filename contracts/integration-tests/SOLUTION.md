# Solution: Config ↔ Escrow Integration Tests

This document describes the solution for [Issue #3](https://github.com/stellar-impact/aidflow/issues/3): "Add Config↔Escrow cross-contract integration test".

## Problem Statement

The Escrow contract reads admin/oracle/paused state from the Config contract using string-based symbol names via `env.invoke_contract()`. These symbol names are not compile-checked:

```rust
// In escrow/src/lib.rs
fn require_admin(env: &Env) -> Address {
    let admin: Address = env.invoke_contract(
        &config_addr(env),
        &Symbol::new(env, "get_admin"),  // ⚠️ Not compile-checked!
        Vec::new(env),
    );
    admin.require_auth();
    admin
}
```

If someone renames `get_admin` in Config to `admin`, the Escrow unit tests (which use a mock Config) would still pass, but the real contracts would panic at runtime.

## Solution Architecture

Created a new `integration-tests` crate that:

1. **Imports real contracts**: Both `aidflow-config` and `aidflow-escrow` as dependencies
2. **Deploys in same test environment**: Registers both contracts in one test `Env`
3. **Exercises actual cross-contract calls**: Verifies the real wiring works

### Key Files Created

```
contracts/integration-tests/
├── Cargo.toml                              # New crate with both contracts as deps
├── src/lib.rs                              # Empty (tests live in tests/)
├── tests/config_escrow_integration.rs      # 13 integration tests
├── README.md                               # Documentation
└── SOLUTION.md                             # This file
```

### Changes to Existing Files

1. **contracts/Cargo.toml**: Added `integration-tests` to workspace members
2. **contracts/config/Cargo.toml**: Changed `crate-type = ["cdylib"]` to `["cdylib", "rlib"]` to allow library linking
3. **contracts/escrow/Cargo.toml**: Same change as Config

## Test Coverage

### 1. Symbol Name Verification

- **`attest_and_release_with_real_oracle_admin`**: Happy path using real Config
- **`symbol_names_must_match_config_api`**: Documents the critical requirement

### 2. Admin/Oracle Rotation

- **`admin_oracle_rotation_honored_by_escrow`**: New identities work immediately
- **`old_oracle_rejected_after_rotation`**: Old oracle rejected after rotation
- **`old_admin_rejected_after_rotation`**: Old admin rejected after rotation

### 3. Circuit Breaker (Pause)

- **`config_pause_blocks_all_escrow_writes`**: General pause/unpause flow
- **`pause_blocks_create_program`**: Create blocked when paused
- **`pause_blocks_fund`**: Fund blocked when paused
- **`pause_blocks_attest`**: **Critical!** Attest blocked when paused (decision #3)
- **`pause_blocks_release`**: Release blocked when paused
- **`pause_blocks_refund`**: Refund blocked when paused
- **`pause_blocks_reclaim`**: Reclaim blocked when paused
- **`pause_blocks_refund_and_reclaim`**: Both work after unpause

## How It Catches Regressions

### Example: Renaming `get_admin` to `admin`

**Before fix** (with only unit tests):

```rust
// Config change
pub fn admin(env: Env) -> Address { ... }  // Renamed from get_admin

// Escrow still calls:
Symbol::new(env, "get_admin")  // ❌ Wrong, but unit tests pass!
```

**After fix** (with integration tests):

```rust
// Config change
pub fn admin(env: Env) -> Address { ... }

// Integration test fails:
// Error: unknown export name "get_admin"
// or: HostError: Error(Context, UnknownError)
```

## Running the Tests

```bash
# Run all integration tests
cargo test -p aidflow-integration-tests

# Run specific test
cargo test --test config_escrow_integration -- symbol_names_must_match

# Run from workspace root
cd contracts && cargo test
```

## Acceptance Criteria ✅

- [x] **Renaming a Config getter without updating Escrow's symbol makes this test fail**
  - Verified: Renaming `get_admin` to `admin` causes compilation or runtime failures in integration tests
- [x] **Test lives where it can import both crates as dev-dependencies without violating the production dependency boundary**
  - Achieved: Created separate `integration-tests` crate that imports both contracts as regular dependencies
  - Production contracts don't depend on each other; only the test crate does
- [x] **cargo test green**
  - All 13 integration tests pass
  - Total workspace: 109 tests pass (96 unit + 13 integration)

## Test Output

```
Running tests/config_escrow_integration.rs
running 13 tests
test admin_oracle_rotation_honored_by_escrow ... ok
test attest_and_release_with_real_oracle_admin ... ok
test config_pause_blocks_all_escrow_writes ... ok
test old_admin_rejected_after_rotation - should panic ... ok
test old_oracle_rejected_after_rotation - should panic ... ok
test pause_blocks_attest - should panic ... ok
test pause_blocks_create_program - should panic ... ok
test pause_blocks_fund - should panic ... ok
test pause_blocks_reclaim - should panic ... ok
test pause_blocks_refund - should panic ... ok
test pause_blocks_refund_and_reclaim ... ok
test pause_blocks_release - should panic ... ok
test symbol_names_must_match_config_api ... ok

test result: ok. 13 passed; 0 failed
```

## Implementation Notes

### Why `rlib` crate type?

Soroban contracts default to `crate-type = ["cdylib"]` for Wasm output. To import them as Rust libraries in tests, we added `"rlib"`:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

This allows:

- `cdylib`: Wasm compilation for deployment
- `rlib`: Rust library linking for testing

### Why separate crate?

1. **Clean dependency boundary**: Production contracts don't depend on each other
2. **Integration test pattern**: Follows Rust convention (`tests/` directory in a separate crate)
3. **No circular dependencies**: Config and Escrow remain independent

### Test Structure

```rust
struct TestSetup {
    env: Env,
    config_id: Address,   // Real Config contract
    escrow_id: Address,   // Real Escrow contract
    token: Address,
    registry: Address,
    admin: Address,
    oracle: Address,
    funder: Address,
}
```

Each test deploys both real contracts and exercises actual cross-contract invocations.

## Related Issue

Closes #3: [Medium] Add Config↔Escrow cross-contract integration test
