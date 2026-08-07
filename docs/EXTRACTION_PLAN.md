# AidFlow Extraction Plan

**Status:** Roadmap  
**Target:** Post-Audit

---

## Overview

AidFlow starts as a monorepo for rapid development but is designed for contract extraction into a standalone repository once interfaces stabilize. This document outlines the extraction strategy.

## Rationale

### Why Monorepo Now?

- Fast iteration during MVP
- Easy cross-module refactoring
- Single CI/CD pipeline
- Simplified local development setup

### Why Extract Later?

- Contracts are the **public API** of the platform
- External integrators need standalone contract repository
- Separate audit scope for contracts vs services
- Independent versioning and release cycles
- Reduced coupling for long-term maintenance

## Extraction Phases

### Phase 1: Contracts Extraction (Post-Audit)

**Target Repository:** `stellar-impact/aidflow-contracts`

**Scope:**
```
contracts/
├── types/           → aidflow-contract-types (Rust crate)
├── config/          → aidflow-config (contract)
├── escrow/          → aidflow-escrow (contract)
├── voucher_registry/ → aidflow-voucher-registry (contract)
├── merchant_registry/ → aidflow-merchant-registry (contract)
└── sdk/             → @aidflow/contract-sdk (npm package)
```

**Process:**

1. **Pre-Extraction Checklist:**
   - [ ] All contracts audited by reputable firm
   - [ ] Interfaces marked `@stable` and frozen
   - [ ] Contract SDK test coverage ≥80%
   - [ ] Migration guide written
   - [ ] Versioning strategy documented

2. **Extract with git-filter-repo:**
   ```bash
   git clone https://github.com/stellar-impact/aidflow.git aidflow-contracts
   cd aidflow-contracts
   git filter-repo --path contracts/ --path-rename contracts/:
   ```

3. **Publish Packages:**
   - Rust crates to crates.io: `aidflow-contract-types` v1.0.0
   - npm package to registry: `@aidflow/contract-sdk` v1.0.0
   - Tag release: `v1.0.0`

4. **Update Monorepo:**
   - Remove `contracts/` directory (except build scripts)
   - Update `services/` to depend on `aidflow-contract-types` from crates.io
   - Update `clients/` to depend on `@aidflow/contract-sdk` from npm
   - Update CI workflows to use published packages
   - Document migration in `docs/MIGRATION.md`

5. **Verify Boundary Enforcement:**
   - `./scripts/check-boundaries.sh` must still pass
   - No relative imports into removed `contracts/` directory
   - All references via published packages only

**Timeline:** 2-3 weeks post-audit

---

### Phase 2: Indexer Extraction (Optional)

**Target Repository:** `stellar-impact/aidflow-indexer`

**Trigger:** External demand for standalone indexer (3+ external integrators)

**Scope:**
```
services/
├── cmd/indexer/         → main package
├── internal/indexer/    → internal logic
└── pkg/indexer-client/  → @aidflow/indexer-client (npm)
```

**Rationale:**
- Indexer is `@beta` — may change frequently
- Only extract if external integrators need it
- Otherwise keep in monorepo for flexibility

**Process:** Similar to Phase 1, using `git filter-repo --path services/cmd/indexer/ --path services/internal/indexer/`

---

### Phase 3: No Further Splits

**Remaining in Monorepo:**
- `services/cmd/{oracle,onboarding,relayer}` — `@internal` (never published)
- `clients/` — coupled to services, no external demand

These components are AidFlow-specific and benefit from monorepo co-location.

---

## Dependency Direction (STRICT)

Post-extraction, dependency direction remains enforced:

```
@aidflow/contract-sdk (standalone repo, published)
    ↑
services/ (monorepo, internal)
    ↑
clients/ (monorepo, internal)
```

**Rules:**
- Services never import from clients
- Clients never use relative paths into services (HTTP/GraphQL only)
- No circular dependencies between layers

Enforced by `scripts/check-boundaries.sh` in CI.

---

## Versioning Strategy

### Contracts (Post-Extraction)

**Semantic Versioning:**
- **Major (X.0.0):** Breaking contract interface changes (requires data migration)
- **Minor (1.X.0):** New functions, backward-compatible
- **Patch (1.0.X):** Bug fixes, no interface changes

**Stability Tags:**
- `@stable` — External-dependable, breaking change = major bump
- `@beta` — May change, consumers pin exact versions

### SDK

Follows contract versioning (1:1 mapping).

---

## Communication Plan

1. **Announcement:** GitHub discussion + blog post
2. **Migration Guide:** Step-by-step instructions for integrators
3. **Deprecation Period:** 6 months support for old monorepo references
4. **Office Hours:** Weekly Q&A for integrators during migration

---

## Success Criteria

- [ ] Extracted contracts repo has CI green
- [ ] Published packages installable from registries
- [ ] Monorepo updated to consume published packages
- [ ] Boundary checks pass in both repos
- [ ] Documentation complete and public
- [ ] Zero external integrator blockers

---

## Related Documents

- [Architecture](ARCHITECTURE.md)
- [Contract Interfaces](CONTRACT_INTERFACES.md)
- [ADR 0002: Monorepo with Extraction Plan](adr/0002-monorepo-with-extraction-plan.md)
