# Config ↔ Escrow Integration Tests

This crate contains cross-contract integration tests that verify the real Config and Escrow contracts work together correctly.

## Purpose

The Escrow contract calls Config contract methods using string-based symbol names via `env.invoke_contract()`:

- `"get_admin"` - to verify admin authorization
- `"get_oracle"` - to verify oracle authorization
- `"is_paused"` - to check the circuit breaker state

These symbol names are **not** compile-checked. If a Config method is renamed without updating Escrow's symbol strings, unit tests with mocks would still pass but the real contracts would panic at runtime.

## What These Tests Verify

1. **Symbol Name Accuracy**: Tests import both real contracts and exercise actual cross-contract calls, so renaming a Config getter without updating Escrow's symbols causes test failures.

2. **Admin/Oracle Rotation**: Tests verify that when admin or oracle addresses are changed in Config, Escrow immediately honors the new identities.

3. **Circuit Breaker Integration**: Tests verify that pausing Config blocks all Escrow write operations, including `attest_milestone`.

## Test Coverage

- ✅ `attest_and_release_with_real_oracle_admin` - Happy path using real Config identities
- ✅ `admin_oracle_rotation_honored_by_escrow` - New admin/oracle work after rotation
- ✅ `old_oracle_rejected_after_rotation` - Old oracle rejected after rotation
- ✅ `old_admin_rejected_after_rotation` - Old admin rejected after rotation
- ✅ `config_pause_blocks_all_escrow_writes` - Pause/unpause flow works
- ✅ `pause_blocks_create_program` - Create blocked when paused
- ✅ `pause_blocks_fund` - Fund blocked when paused
- ✅ `pause_blocks_attest` - Attest blocked when paused (critical!)
- ✅ `pause_blocks_release` - Release blocked when paused
- ✅ `pause_blocks_refund` - Refund blocked when paused
- ✅ `pause_blocks_reclaim` - Reclaim blocked when paused
- ✅ `pause_blocks_refund_and_reclaim` - Both operations work after unpause

## Running the Tests

```bash
# Run all integration tests
cargo test --package aidflow-integration-tests

# Run specific test
cargo test --test config_escrow_integration -- attest_and_release_with_real_oracle_admin

# Run from workspace root
cargo test -p aidflow-integration-tests
```

## Implementation Notes

- This crate imports real Config and Escrow contracts as dependencies (not dev-dependencies, since integration tests need them in the main dependency tree)
- Both contracts are compiled with `crate-type = ["cdylib", "rlib"]` to support both Wasm output and library linking
- Tests deploy both contracts in the same test environment to verify actual cross-contract invocation
