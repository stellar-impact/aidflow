---
name: Drips Wave Contribution
about: Template for Stellar Wave contribution issues
title: '[Complexity] Feature/Task Title'
labels: ['Stellar Wave', 'needs-triage']
assignees: ''
---

## Description

<!-- 2-3 sentences: what needs to be done and why -->

## Acceptance Criteria

- [ ] Concrete, testable outcome 1
- [ ] Concrete, testable outcome 2
- [ ] Concrete, testable outcome 3

## Technical Context

<!-- File paths, APIs, documentation links, dependencies -->

**Files:**
- `path/to/file1.rs`
- `path/to/file2.go`

**APIs:**
- [Soroban SDK Docs](https://docs.rs/soroban-sdk/26.1.0/)
- [Stellar Developers](https://developers.stellar.org)

**Dependencies:**
- Depends on #<!-- issue number -->

## Definition of Done

- [ ] Tests pass in CI
- [ ] Code reviewed and approved
- [ ] Documentation updated

## Verification (REQUIRED before PR — PRs won't merge until CI passes)

Run locally before opening a PR:

**For Contracts:**
```bash
cd contracts
stellar contract build --package <package-name>
cargo test --package <package-name>
cargo clippy --package <package-name> -- -D warnings
```

**For Services:**
```bash
cd services
go build ./cmd/<service-name>
go test ./...
```

**For Clients:**
```bash
cd clients/<client-name>
npm run build
npm test
```

**Boundary Check:**
```bash
./scripts/check-boundaries.sh
```

> **Using an AI agent?** Paste this issue's Technical Context + these commands into it. Use the PINNED versions (§0.5 of `docs/AIDFLOW_MASTER_PROMPT.md`): 
> - `stellar-cli` 27.x (command: `stellar contract build`, NOT `soroban contract build`)
> - `soroban-sdk` 26.1+ for contracts
> - `github.com/stellar/go-stellar-sdk` for Go services (NOT deprecated `github.com/stellar/go`)
> 
> **Do NOT submit code that fails these checks locally.** If your agent generates code that won't compile or uses outdated APIs, ask in the comments before submitting a PR.

---

**Points:** <!-- 100 | 150 | 200 -->  
**Area:** <!-- contracts | services | clients | infra | docs -->  
**Priority:** <!-- P0 | P1 | P2 -->
