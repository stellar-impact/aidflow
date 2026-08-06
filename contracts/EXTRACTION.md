# Contracts Extraction

**Target:** `stellar-impact/aidflow-contracts` (Wave 7-8)

This directory will be extracted into a standalone repository after audit.

See `../docs/EXTRACTION_PLAN.md` for full details.

## Stability Markers

All public APIs are marked with stability:
- `@stable` — External-dependable, breaking change requires major version bump
- `@beta` — May change, consumers pin exact versions

## Contracts

- `types/` — Shared types (`@stable` post-audit)
- `config/` — Admin multisig + oracle (`@stable` post-audit)
- `escrow/` — Programs, milestones, funding (`@stable` post-audit)
- `voucher_registry/` — Batch issuance, claims (`@stable` post-audit)
- `merchant_registry/` — Allowlist, redemption (`@stable` post-audit)
- `sdk/` — TypeScript SDK (`@stable` post-audit)
