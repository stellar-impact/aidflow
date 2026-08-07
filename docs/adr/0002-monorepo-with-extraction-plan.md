# ADR 0002: Monorepo with Extraction Plan

**Date:** 2026-08-06  
**Status:** Accepted  
**Deciders:** Architecture team

---

## Context

AidFlow consists of multiple components: Soroban contracts (Rust), backend services (Go), and client applications (TypeScript). We need to decide on repository structure for MVP development and long-term maintenance.

### Requirements

1. **Fast Iteration:** Enable rapid development during MVP
2. **Easy Refactoring:** Allow cross-component changes without PRs across repos
3. **Contract Stability:** Eventually publish contracts as standalone package
4. **Independent Versioning:** Contracts vs services may evolve at different rates
5. **External Integration:** Integrators should depend only on stable contracts
6. **Clear Boundaries:** Enforce dependency direction (contracts → services → clients)

---

## Decision

We will use a **monorepo during MVP development** with a **planned extraction of contracts** into a standalone repository after audit.

---

## Rationale

### Why Monorepo (Now)?

**1. Fast Iteration**
- Single PR can update contract + service + client together
- No cross-repo coordination during early development
- Atomic commits across layers

**2. Simplified Local Development**
- Single `git clone`
- Single `docker-compose up` for full stack
- No version mismatches between local copies

**3. Shared CI/CD**
- Single CI pipeline validates all components
- Boundary checks enforce dependency direction
- One build → one deploy

**4. Easier Refactoring**
- Contract interface changes immediately visible to consumers
- Rename functions across contract + SDK + clients in one PR
- No breaking changes discovered later

### Why Extract Later?

**1. Contract Stability**
- Contracts are the **public API** of AidFlow
- Need stable interfaces before external integrators depend on them
- Audit first, then freeze interfaces

**2. Independent Versioning**
- Contracts may be v1.0.0 (stable)
- Services may be v0.5.0 (still iterating)
- Separate repos enable independent release cycles

**3. Audit Scope**
- Auditors need clear boundary: "audit this repo only"
- Smaller surface area = lower audit cost
- Extracted repo is the audited artifact

**4. External Integration**
- Integrators clone `stellar-impact/aidflow-contracts`
- They don't need services or client code
- Reduces confusion and download size

### Alternatives Considered

**Polyrepo (Separate Repos from Day 1)**
- ✅ Clear boundaries from start
- ✅ Independent versioning
- ❌ Slow iteration (PRs across repos)
- ❌ Complex local dev setup
- ❌ Easy to break interfaces without noticing
- **Rejected:** Too much coordination overhead during MVP

**Monorepo Forever**
- ✅ Simple forever
- ❌ External integrators clone everything (overkill)
- ❌ Audit scope unclear
- ❌ Versioning confusion (what's stable?)
- **Rejected:** Doesn't serve integrators well

**Immediate Extraction**
- ✅ Clean from start
- ❌ Contracts not stable yet (will change during MVP)
- ❌ Frequent breaking changes in separate repo
- **Rejected:** Premature optimization

---

## Decision Details

### Monorepo Structure

```
aidflow/
├── contracts/       # EXTRACTABLE → aidflow-contracts (post-audit)
│   ├── types/      # @stable after audit
│   ├── config/     # @stable after audit
│   ├── escrow/     # @stable after audit
│   └── sdk/        # @stable after audit
├── services/        # REMAINS in monorepo
│   ├── cmd/        # @internal services
│   └── internal/
└── clients/         # REMAINS in monorepo
    ├── donor-dashboard/
    ├── beneficiary-pwa/
    └── merchant-app/
```

### Extraction Timeline

| Phase | Action |
|-------|--------|
| **MVP** | Develop in monorepo |
| **Beta** | Stabilize contract interfaces |
| **Audit** | Extract contracts → `aidflow-contracts` repo |
| **Mainnet** | Monorepo depends on published contracts |

### Boundary Enforcement

**Strict dependency direction:**
```
contracts/  →  depends only on soroban-sdk
services/   →  depends on published @aidflow/contract-types
clients/    →  depends on published @aidflow/contract-sdk
```

**Enforced by:**
- `scripts/check-boundaries.sh` (runs in CI on every PR)
- Fails if services/clients use relative paths into contracts/
- Services/clients MUST depend on published packages

### Extraction Process (Audit phase)

1. **Freeze Contract Interfaces:**
   - Mark all public APIs `@stable`
   - Complete audit
   - No breaking changes allowed

2. **Extract with git-filter-repo:**
   ```bash
   git clone aidflow aidflow-contracts
   cd aidflow-contracts
   git filter-repo --path contracts/
   ```

3. **Publish Packages:**
   - `aidflow-contract-types` (Rust crate) v1.0.0
   - `@aidflow/contract-sdk` (npm) v1.0.0

4. **Update Monorepo:**
   - Remove contracts/ directory
   - Update services/clients to depend on published packages
   - CI validates no relative imports

---

## Consequences

### Positive

- **Fast MVP Development:** No cross-repo coordination
- **Simplified Onboarding:** Single clone, single setup
- **Easy Refactoring:** Change interfaces freely during MVP
- **Clear Migration Path:** Extraction plan defined upfront
- **External Integration Ready:** Post-audit, clean contracts repo

### Negative

- **Migration Overhead:** Need to extract later (planned work)
- **Temporary Coupling:** Services/clients coupled to contracts during MVP
- **Version Ambiguity:** "Which version is stable?" unclear until extraction

### Mitigations

- **Stability Markers:** Tag APIs with `@stable` or `@beta` from day 1
- **Boundary Checks:** CI enforces dependency direction immediately
- **Extraction Dry-Run:** Test extraction process early (Beta)
- **Migration Guide:** Write guide before extraction (Audit phase)

---

## Validation

This decision will be validated by:

1. **Developer Velocity:** MVP developed in <8 weeks
2. **Boundary Checks Pass:** CI green on every PR
3. **Successful Extraction:** Dry-run extraction in the Beta phase works
4. **External Integration:** Post-extraction, external team can integrate contracts

If extraction proves too complex, we may keep monorepo but publish contracts from subdirectory (submodule pattern).

---

## Related Documents

- [Extraction Plan](../EXTRACTION_PLAN.md)
- [Architecture](../ARCHITECTURE.md)
- [Contract Interfaces](../CONTRACT_INTERFACES.md)

---

## Future Considerations

**Potential Phase 2 Extraction (later):**
- If indexer sees external demand, extract it too
- Otherwise keep in monorepo (internal-only service)

**Monorepo Tools:**
- Consider Turborepo or Nx if build times become issue
- Currently simple npm workspaces sufficient
