# AidFlow — Master Scaffolding Prompt (Spec-Driven Agent)

> **Status:** Stack LOCKED. This is the canonical prompt to hand to a spec-driven
> agent (or to execute directly) to scaffold AidFlow and generate contribution issues.

---

## 0. Locked Technology Stack (DO NOT DEVIATE)

| Layer | Choice | Rationale |
|-------|--------|-----------|
| **Smart contracts** | Rust + `soroban-sdk` **26.x** (build via `stellar-cli` 27.x) | Native Soroban — see §0.5 for pinned versions & build target |
| **Backend services** | **Go** — SDK `github.com/stellar/go-stellar-sdk` (NOT the deprecated `stellar/go`) | SDF reference SDK, goroutine concurrency, single static binary, best for indexer/relayer |
| **Donor dashboard** | **Vite + React + TypeScript** + TanStack Query | SPA behind auth, no SSR need, fast builds |
| **Beneficiary app** | **Vite + React + TypeScript** + Workbox (PWA) | Offline-first, tiny bundle, passkey |
| **Merchant app** | **Expo (React Native)** | Native camera/QR, offline sync |
| **Shared contract SDK** | TypeScript, published as `@aidflow/contract-sdk` | Consumed by all clients |
| **DB / queue / storage** | Postgres · Redis · S3/IPFS | Standard |
| **Infra** | Docker Compose (dev) · Kubernetes + Kustomize (prod) | — |
| **Observability** | Prometheus + Grafana | — |

**Language coherence:** Rust (contracts) + Go (services) + TypeScript (clients). Three languages, each playing to strengths.

### 0.5. Verified Versions & Anti-Drift Guards (Updated 2026-08-05)

**CRITICAL:** The Stellar toolchain had breaking changes in late 2025 / early 2026. An agent using stale knowledge will generate broken code. Pin these exact versions.

| Component | Version | Breaking change context |
|-----------|---------|------------------------|
| **Stellar CLI** | `stellar-cli` **27.0.0** (2026-06-17) | [Renamed](https://github.com/stellar/stellar-cli/releases) from `soroban-cli`; commands are now `stellar contract build`, not `soroban contract` |
| **soroban-sdk** | **26.1.0** (2026-06-08 stable) or **27.0.0-rc.1** (2026-06-17 RC) | [Must build](https://crates.io/crates/soroban-sdk) via `stellar contract build`, never `cargo build`; requires Rust **1.84+** for `wasm32v1-none` target |
| **Go Stellar SDK** | `github.com/stellar/go-stellar-sdk` (published 2026-04-07) | [Repo migrated](https://github.com/stellar/go-stellar-sdk/releases); old `github.com/stellar/go` import path is **deprecated** |
| **Rust toolchain** | **1.84+** | wasm32v1-none target required; wasm32-unknown-unknown fails on 1.82+ |
| **Protocol** | 26 (testnet before 2026-04-16; mainnet before 2026-05-06) | Protocol 27 support in CLI 27.x |

**Build commands (contracts):**
```bash
stellar contract build   # CORRECT (CLI 27.x)
soroban contract build   # WRONG — old CLI name
cargo build --target wasm32-unknown-unknown  # WRONG — unsupported target on Rust 1.82+
```

**Go imports:**
```go
import "github.com/stellar/go-stellar-sdk/clients/horizonclient"  // CORRECT
import "github.com/stellar/go/clients/horizonclient"              // WRONG — deprecated path
```

**Anti-hallucination directive for the agent:**
- If you do not recognize an API (Soroban storage type, auth macro, Go Horizon method), **FLAG IT** rather than guessing. Write `// TODO: verify this API exists in soroban-sdk 26.1.0` or `// AGENT-FLAG: confirm horizonclient method` in the code.
- When a method/macro/function from training knowledge doesn't compile, check if it was renamed or moved — do NOT silently invent a replacement.
- Contracts MUST compile with `stellar contract build` on the pinned SDK. Services MUST compile with `go build` on the new import path.

**Checkpoint enforcement:** scaffold in the module order from §9. After each module, the agent MUST report "Module N builds: ✓ / ✗" before proceeding. A build failure blocks the next module.

---

## 1. Mission

**AidFlow** — transparent, conditional aid disbursement on Stellar/Soroban.
NGOs fund milestone-gated escrows; beneficiaries claim stablecoin vouchers;
merchants/anchors redeem to local fiat; every movement is publicly auditable.

**Design principles**
- On-chain: asset movements + commitments only. **No PII.**
- Off-chain: encrypted beneficiary data (Postgres + KMS envelope encryption). GDPR-friendly.
- Two-key release: oracle **attests**, admin multisig **releases**.
- Gas efficiency: batch issuance, lazy/pull claim, minimal on-chain footprint.
- Upgradable contracts: admin multisig + timelock + versioned storage.

---

## 2. Architecture

### Soroban contracts (Rust)
- **Config / AccessControl** — admin multisig, oracle address, circuit breaker (pause).
- **Escrow** — programs, milestones, funding, attestation, release.
- **VoucherRegistry** — batch issuance, claim, expiry, redemption.
- **MerchantRegistry** — merchant allowlist, redemption.

### Services (Go)
- **onboarding** — beneficiary CSV import, wallet provisioning, encrypted PII store. `@internal`
- **oracle** — field evidence upload (S3) → on-chain hash via `Escrow.attest_milestone`. `@internal`
- **indexer** — Horizon event stream → Postgres → GraphQL (gqlgen). `@beta` (future-extractable)
- **relayer** — Redis queue, fee-bump sponsorship for beneficiaries. `@internal`

### Clients
- **donor-dashboard** — Vite/React/TS, TanStack Query, Recharts, Freighter/Albedo connect.
- **beneficiary-pwa** — Vite/React/TS, Workbox, `@github/webauthn-json` passkey, QR gen.
- **merchant-app** — Expo/React Native, QR scan, offline sync.

### Anchor integration
SEP-10 (auth), SEP-24 (interactive withdraw / fiat cash-out), optional SEP-12 (merchant KYC).

### Infra
Docker Compose (full local stack incl. Stellar quickstart) · K8s base + testnet/mainnet overlays · Prometheus/Grafana.

---

## 3. Modular Monorepo — Split-Ready

Single monorepo now; extract contracts (and later indexer) into standalone repos when interfaces stabilize.

### Dependency direction (STRICT)
```
contracts/  →  depends only on soroban-sdk (root of dep tree)
services/   →  depends on @aidflow/contract-types (published), Horizon
clients/    →  depends on @aidflow/contract-sdk (published), services (HTTP/GraphQL)
```
**Rule:** `contracts/` never imports `services/` or `clients/`. Services/clients consume contracts only via **published, versioned packages** — never relative paths.

### Published packages (from monorepo)
| Package | Source | Consumers | Stability |
|---------|--------|-----------|-----------|
| `aidflow-contract-types` (crate) | `contracts/types/` | services | `@stable` |
| `@aidflow/contract-sdk` (npm) | `contracts/sdk/` | clients | `@stable` |
| `@aidflow/indexer-client` (npm) | `services/indexer/client/` | clients | `@beta` |

### Stability tiers (tag every public interface)
- `@stable` — external-depend-able; breaking change = major bump; audit before change.
- `@beta` — may change; consumers pin exact versions.
- `@internal` — monorepo-only; never published.

### Extraction roadmap (`docs/EXTRACTION_PLAN.md`)
1. **Phase 1 (post-audit):** extract `contracts/` → `aidflow-contracts` via `git filter-repo`; publish v1.0.0 packages; monorepo consumes registry versions.
2. **Phase 2 (optional):** extract `services/indexer/` → `aidflow-indexer` if external demand.
3. **Phase 3:** no further splits — clients + remaining services stay coupled.

### Boundary enforcement
`scripts/check-boundaries.sh` (CI, every PR): fails if `contracts/` imports `services/`, or if `services/`/`clients/` use relative paths into `contracts/` instead of published packages.

---

## 4. Repository Layout

```
aidflow/
├── README.md · LICENSE (MIT or Apache-2.0) · .gitignore · Makefile
├── package.json                     # npm workspaces root
├── .github/
│   ├── workflows/
│   │   ├── boundaries-check.yml      # dependency-direction guard
│   │   ├── contracts-ci.yml          # fmt, clippy, test, soroban build, cargo audit
│   │   ├── services-ci.yml           # go fmt, golangci-lint, go test, docker build
│   │   ├── clients-ci.yml            # lint, test, build; Playwright E2E on main
│   │   └── deploy-testnet.yml        # deploy + init contracts on testnet
│   └── ISSUE_TEMPLATE/ { bug_report, feature_request, contribution }.md
├── contracts/                        # EXTRACTABLE → aidflow-contracts
│   ├── EXTRACTION.md · Cargo.toml
│   ├── types/  (aidflow-contract-types crate, @stable)
│   ├── config/ escrow/ voucher_registry/ merchant_registry/
│   ├── sdk/    (@aidflow/contract-sdk, TypeScript, @stable)
│   └── scripts/ { build, test, deploy-testnet }.sh
├── services/                         # Go
│   ├── EXTRACTION.md · go.mod
│   ├── cmd/ { onboarding, oracle, indexer, relayer }/
│   ├── internal/ { stellar, crypto, db }/
│   ├── pkg/indexer-client/           # @beta published client
│   └── migrations/                   # golang-migrate
├── clients/
│   ├── donor-dashboard/   (Vite+React+TS)
│   ├── beneficiary-pwa/   (Vite+React+TS+Workbox)
│   └── merchant-app/      (Expo/React Native)
├── infra/ { docker-compose.yml, k8s/{base,overlays/{testnet,mainnet}}, helm/ }
├── docs/ { ARCHITECTURE, EXTRACTION_PLAN, CONTRACT_INTERFACES, API, CONTRIBUTING, SECURITY }.md
│        └── adr/ 0001-use-soroban-for-escrow.md · 0002-monorepo-with-extraction-plan.md
└── scripts/ { setup-dev, seed-testnet, check-boundaries }.sh
```

---

## 5. Contract Interfaces (implement from these)

```rust
// Config / AccessControl
fn init(env, admin: Address /*multisig*/, oracle: Address);
fn set_admin(env, new_admin: Address);      // require_auth(admin)
fn set_oracle(env, oracle: Address);        // require_auth(admin)
fn pause(env); fn unpause(env); fn is_paused(env) -> bool;

// Escrow
fn create_program(env, funder, token: Address, milestones: Vec<Milestone>) -> u64;
fn fund(env, program_id: u64, amount: i128);
fn attest_milestone(env, program_id, milestone_id: u32, evidence_hash: BytesN<32>); // require_auth(oracle)
fn release(env, program_id, milestone_id: u32);            // require_auth(admin)
fn refund_unspent(env, program_id: u64);                   // require_auth(admin)
fn get_program(env, program_id) -> ProgramView;

// VoucherRegistry
fn issue_batch(env, program_id, recipients: Vec<(Address,i128)>, expiry: u64) -> Vec<u64>; // admin
fn claim(env, voucher_id: u64);            // require_auth(beneficiary), pull-based
fn expire(env, voucher_id: u64);           // anyone after expiry → returns to escrow
fn get_voucher(env, voucher_id) -> VoucherView;

// MerchantRegistry
fn register(env, merchant, payout: Address, category: u32);  // admin
fn deactivate(env, merchant: Address);                        // admin
fn redeem(env, from /*beneficiary*/, merchant, amount: i128); // require_auth(from)
fn is_active(env, merchant) -> bool;
```

**Events:** program_created · funded · milestone_attested{evidence_hash} · released · voucher_issued · voucher_claimed · voucher_redeemed · voucher_expired · refunded · paused/unpaused.

**Gas & upgrade:** batch issuance (≤40/tx, paginate); lazy claim/redeem; store 32-byte hashes only; explicit TTL/rent per voucher; fee sponsorship via relayer; `update_current_contract_wasm` behind multisig + 48–72h timelock + `upgrade_scheduled` event; `schema_version` field + lazy migration on touch.

---

## 6. Contribution Issue Format

```markdown
labels: ["<Trivial|Medium|High>", "<area>", "<Pn>"]

## [<Complexity>] <Title>
**Complexity:** <Trivial|Medium|High>
### Description
<2–3 sentences: what and why>
### Acceptance Criteria
- [ ] concrete, testable outcome
### Technical Context
<file paths, docs, APIs>
### Definition of Done
- [ ] tests pass in CI  - [ ] reviewed  - [ ] docs updated
### Verification (REQUIRED before PR — PRs won't merge until CI passes)
Run locally before opening a PR:
- [ ] contracts: `stellar contract build` + `cargo test`
- [ ] services: `go build ./... && go test ./...`
- [ ] clients: `npm run build && npm test`
- [ ] imports touched: `./scripts/check-boundaries.sh`
> **Using an AI agent?** Paste this issue's Technical Context + these commands into it. Use the PINNED versions (§0.5): `stellar-cli` 27.x, `soroban-sdk` 26.1, `go-stellar-sdk` (new import path). Do NOT submit code that fails these checks locally.
```

Every generated issue body MUST include this Verification block (tailored to its area).

**Labels:** complexity · area {contracts,services,clients,infra,docs,security,testing,extraction} · priority {P0,P1,P2} · `good first issue`.
**Milestones:** MVP · Beta · Audit Prep.

### Issue backlog (generate full bodies for each)

**Trivial** — architecture diagram; SEP-24 flow doc; Escrow doc comments + `@stable` markers; indexer healthcheck; ESLint/Prettier; voucher-expiry unit test; issue templates; `.env.example`; license headers; CSV import template; write `EXTRACTION_PLAN.md`; `@stable` markers on contract types.

**Medium** — `Escrow.create_program`; onboarding CSV endpoint; indexer Horizon SSE; passkey wallet in PWA; merchant QR scanner; relayer fee-bump; `VoucherRegistry.issue_batch`; donor funding UI; beneficiary Postgres migration; milestone-release integration test; PII envelope encryption; oracle Prometheus metrics; PWA balance display; upgrade timelock; merchant allowlist; publish `aidflow-contract-types`; TS SDK `@aidflow/contract-sdk`; npm workspaces setup; `check-boundaries.sh`.

**High** — full oracle bridge; E2E SEP-24 cash-out; Playwright E2E suite; storage-migration on upgrade; donor audit timeline (GraphQL→React); multisig admin module; batch-issuance gas optimization; security audit checklist + cargo-fuzz; merchant analytics; sealed-secrets K8s; GraphQL subscriptions; voucher batch-expiry cron; SMS notifications (Twilio); relayer chaos tests; privacy-preserving analytics; contracts extraction dry-run (`git filter-repo`); extraction migration guide.

---

## 7. Sample Files to Produce
`contracts/escrow/src/lib.rs` (trait+stub+2 tests) · `services/cmd/onboarding/main.go` (HTTP skeleton) · `clients/donor-dashboard/src/App.tsx` · `infra/docker-compose.yml` · `.github/workflows/contracts-ci.yml` · `.github/workflows/boundaries-check.yml` · root `README.md` · `CONTRIBUTING.md` · `scripts/check-boundaries.sh`.

### 7.1. Contributor-Agent Guardrails (generate these in Module 1)

Many contributors will use AI agents to resolve issues. Generate ALL of the following to keep stale-knowledge PRs out of the review queue:

**`.github/PULL_REQUEST_TEMPLATE.md`:**
```markdown
## Checklist
- [ ] I ran the issue's Verification commands locally and they pass
- [ ] Tests pass: `make test` (or the issue-specific command)
- [ ] I read the issue's Technical Context and used the specified APIs/files
- [ ] (If using an AI agent) I confirmed it used the pinned versions (§0.5 of docs/AIDFLOW_MASTER_PROMPT.md)

Closes #<issue-number>
```

**`CONTRIBUTING.md` — add a "Using AI Agents to Complete Issues" section:**
- Always verify locally first; PRs will not be merged until CI passes.
- Pin versions exactly (`soroban-sdk` 26.1, `stellar-cli` 27.x, `go-stellar-sdk` new path).
- Read the issue's Technical Context; don't let the agent invent APIs.
- If the agent's code won't compile, ask in comments — don't submit broken PRs.
- **Common agent mistakes to watch for:** `soroban contract build` → should be `stellar contract build`; `github.com/stellar/go` → should be `github.com/stellar/go-stellar-sdk`; `cargo build` for contracts → should be `stellar contract build` (wasm32v1-none target).

**`.github/workflows/pr-ci-guard.yml` — auto-comment on failed CI:**
```yaml
name: PR CI Guard
on:
  workflow_run:
    workflows: ["contracts-ci", "services-ci", "clients-ci"]
    types: [completed]
jobs:
  comment-on-failure:
    if: ${{ github.event.workflow_run.conclusion == 'failure' }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/github-script@v7
        with:
          script: |
            // Post a comment on the associated PR reminding the contributor to
            // run verification locally and check pinned versions (stellar-cli 27,
            // soroban-sdk 26.1, go-stellar-sdk new import path). Idempotent: update
            // an existing bot comment rather than spamming new ones.
```

**`.github/ISSUE_TEMPLATE/contribution.md`** must embed the same Verification block + pinned-version reminder from §6.


## 8. Success Criteria
`docker-compose up` runs full stack · `make test` green across modules · boundary check passes · issues clear enough to pick up cold · all issues tagged + point-valued · production-oriented (CI/CD, observability, upgrade paths).

---

## 9. Build Order & Checkpoint Protocol (Agent MUST follow in sequence)

Build ONE module at a time, in this order. After each, verify it builds/tests and report `Module N: ✓/✗` before starting the next. **Do not generate later modules until the current one compiles.** This prevents a single Soroban/Go API error from cascading into hundreds of lines of broken code.

| # | Module | Verify with | Gate |
|---|--------|-------------|------|
| 1 | Repo foundation (tree, Apache-2.0, workspaces, Makefile, CI skeletons, `check-boundaries.sh`) | `make lint` · `./scripts/check-boundaries.sh` | scaffolds cleanly |
| 2 | Contracts: `types` + Config/AccessControl | `stellar contract build` | wasm builds |
| 3 | Contracts: Escrow (+ 2 unit tests) | `cargo test` + `stellar contract build` | tests pass |
| 4 | Contracts: VoucherRegistry + MerchantRegistry | `cargo test` + `stellar contract build` | tests pass |
| 5 | Contract SDK (`@aidflow/contract-sdk`, TS) | `npm run build` in `contracts/sdk` | type-checks |
| 6 | Services: indexer + relayer (Go, new SDK path) | `go build ./...` · `go test ./...` | compiles |
| 7 | Services: onboarding + oracle | `go build ./...` · `go test ./...` | compiles |
| 8 | Clients (donor dashboard, PWA, merchant) — use `npm create vite@latest` / `create-expo-app`, don't hardcode versions | `npm run build` each | builds |
| 9 | Issue backlog (~48 full bodies) + `docker-compose up` smoke | manual review · compose up | stack runs |

**Rules for the agent:**
1. Report build status after EVERY module. Stop and flag on failure — do not "work around" a compile error by inventing APIs.
2. For JS/TS clients, prefer official scaffolders (`npm create vite@latest`, `create-expo-app@latest`) over hand-writing `package.json` version pins — this avoids stale-version drift.
3. For Go modules, run `go mod tidy` and let it resolve the latest compatible `go-stellar-sdk`; if unsure of the exact tag, pin to the latest published and note it.
4. For Rust, set `soroban-sdk = "26"` in `Cargo.toml` (allows 26.x); if 27.x is stable at build time, note it as an upgrade candidate but do not use an RC in the committed scaffold.
5. Any uncertainty about a Stellar/Soroban API → emit an `AGENT-FLAG` comment for human (Claude) review rather than guessing.

> **Human review handoff:** after all modules, the coordinating human (Claude) reviews contracts (§2–5) and the Go Stellar SDK usage first — these are the highest-drift-risk surfaces. Clients/docs/CI are lower risk.
