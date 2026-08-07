# Services Extraction

**Status:** Remains in monorepo (Phase 3)

Services are `@internal` and tightly coupled to AidFlow's specific use case. They will not be extracted unless external demand emerges.

## Exception: Indexer

If 3+ external integrators request standalone indexer, may extract in a later phase:
- Target: `stellar-impact/aidflow-indexer`
- Publish: `@aidflow/indexer-client` (npm)
- Status: `@beta` (interface may change)

See `../docs/EXTRACTION_PLAN.md` for details.

## Services

- `cmd/indexer/` — Horizon SSE → Postgres (`@beta`, may extract)
- `cmd/relayer/` — Fee-bump sponsorship (`@internal`)
- `cmd/onboarding/` — CSV import + PII encryption (`@internal`)
- `cmd/oracle/` — Evidence → attestation (`@internal`)
