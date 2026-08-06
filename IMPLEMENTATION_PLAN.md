# AidFlow Implementation Plan

**Project:** AidFlow — Transparent, Conditional Aid Disbursement on Stellar/Soroban  
**GitHub:** `stellar-impact/aidflow`  
**Date:** 2026-08-05  
**Target:** Drips Network — Stellar Wave (MVP, Waves 1-3)

---

## Problem Statement

Scaffold a production-grade open-source platform for transparent, conditional aid disbursement on Stellar/Soroban. The system must support milestone-gated escrows, beneficiary voucher claims, and merchant redemption with full public auditability. 

**Critical requirement:** Avoid Stellar/Soroban API drift by using exact pinned versions (stellar-cli 27.x, soroban-sdk 26.1, new Go SDK import path `github.com/stellar/go-stellar-sdk`) and validating each module builds before proceeding to the next.

**Scope:** Build 9 modules sequentially with verification checkpoints. Generate ~15-20 MVP-focused GitHub issues. Prioritize local dev (docker-compose) and testnet deployment. Defer mainnet until post-audit.

---

## Requirements

### From Specification Analysis

**Sequential Build Protocol (§9):**
- Build modules 1-9 in exact order with verification checkpoints
- Report "Module N: ✓" with summary output after each (key files + command results)
- STOP on build failure (✗) - no workarounds or API invention
- Auto-proceed on success - no manual confirmation needed between modules

**Version Pinning (§0.5 - CRITICAL):**
- `stellar-cli` 27.0.0+ (command: `stellar contract build`, NOT `soroban contract build`)
- `soroban-sdk` 26.1.0 (requires Rust 1.84+, target wasm32v1-none)
- Go SDK: `github.com/stellar/go-stellar-sdk` (NEW path, NOT deprecated `stellar/go`)
- NEVER use: `soroban contract build`, `cargo build --target wasm32-unknown-unknown`, old Go import

**API Safety:**
- Emit `// AGENT-FLAG: verify <API>` comments for uncertain Soroban/Go/Stellar APIs
- Do NOT invent or guess APIs from stale training knowledge
- Stop and flag rather than proceed with uncertain code

**GitHub Integration:**
- Org: `stellar-impact`, Repo: `stellar-impact/aidflow`
- Use in all README URLs, package.json/Cargo.toml metadata, CONTRIBUTING clone commands

**Contributor Guardrails (§7.1 - Module 1):**
- Generate PR template with verification checklist + pinned version confirmation
- CONTRIBUTING.md "Using AI Agents" section with common mistakes warning
- CI auto-comment workflow on build failures (`pr-ci-guard.yml`)
- Issue template with Verification block

**Tooling:**
- Use official scaffolders: `npm create vite@latest`, `create-expo-app@latest`
- Use `go mod tidy` for Go dependency resolution
- Never hardcode JS/TS package versions - let scaffolders handle it

### From User Clarifications

**Verification & Reporting:**
- Option (b): Summary output after each module showing key files created and verification command output
- Auto-proceed to next module on success
- STOP and surface full error on failure (✗)

**Deployment Target:**
- Option (c): Local + testnet focus
- Docker Compose for full local stack (Stellar quickstart, Postgres, Redis, S3, all services)
- Kustomize base + testnet overlay (fleshed out)
- Mainnet overlay: stub directory with README noting "deferred until post-audit"

**Issue Scope:**
- Option (c): MVP issues only (Wave 1-3 milestone)
- Generate ~15-20 full issue bodies per §6 format
- Include Verification block in every issue
- Cover: core contracts, onboarding, claim flow, one SEP-24 cash-out, basic donor dashboard, foundation
- Skip Beta/Audit-Prep tier issues for now

---

## Background

### Technology Stack (LOCKED - §0)

**No deviations allowed from this stack:**

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| **Smart contracts** | Rust + soroban-sdk 26.x via stellar-cli 27.x | Native Soroban, pinned versions for stability |
| **Backend services** | Go + `github.com/stellar/go-stellar-sdk` | SDF reference SDK, goroutines, single binary, best for indexer/relayer |
| **Donor dashboard** | Vite + React + TypeScript + TanStack Query | SPA, no SSR, fast builds, wallet integration |
| **Beneficiary PWA** | Vite + React + TypeScript + Workbox | Offline-first, passkey auth, minimal bundle |
| **Merchant app** | Expo (React Native) | Native camera/QR, offline sync, mobile-first |
| **Contract SDK** | TypeScript as `@aidflow/contract-sdk` | Consumed by all client applications |
| **Infrastructure** | Docker Compose (dev), Kubernetes + Kustomize (prod) | Standard deployment patterns |
| **Observability** | Prometheus + Grafana | Metrics and monitoring |
| **Data stores** | Postgres, Redis, S3/IPFS | Relational, queue, object storage |

**Language coherence:** Rust (contracts) + Go (services) + TypeScript (clients) — three languages playing to strengths.

### Version Guards (§0.5 - Updated 2026-08-05)

**CRITICAL:** The Stellar toolchain had breaking changes in late 2025 / early 2026. Using stale knowledge will generate broken code.

| Component | Version | Breaking Change Context |
|-----------|---------|-------------------------|
| **Stellar CLI** | 27.0.0+ (2026-06-17) | Renamed from `soroban-cli`; commands now `stellar contract build` |
| **soroban-sdk** | 26.1.0 (2026-06-08) | Must build via `stellar contract build`, never `cargo build`; requires Rust 1.84+ for wasm32v1-none |
| **Go Stellar SDK** | `github.com/stellar/go-stellar-sdk` (2026-04-07) | Repo migrated; old `github.com/stellar/go` is DEPRECATED |
| **Rust toolchain** | 1.84+ | wasm32v1-none target required; wasm32-unknown-unknown fails on 1.82+ |
| **Protocol** | 26+ (testnet/mainnet) | Protocol 27 support in CLI 27.x |

**Correct build commands (contracts):**
```bash
stellar contract build   # ✓ CORRECT
soroban contract build   # ✗ WRONG - old CLI name
cargo build --target wasm32-unknown-unknown  # ✗ WRONG - unsupported target
```

**Correct Go imports:**
```go
import "github.com/stellar/go-stellar-sdk/clients/horizonclient"  // ✓ CORRECT
import "github.com/stellar/go/clients/horizonclient"              // ✗ WRONG - deprecated
```

### Dependency Direction (STRICT - §3)

Enforced by `scripts/check-boundaries.sh` in CI:

```
contracts/  →  depends only on soroban-sdk (root of dependency tree)
services/   →  depends on @aidflow/contract-types (published), Horizon SDK
clients/    →  depends on @aidflow/contract-sdk (published), services (HTTP/GraphQL)
```

**Rule:** `contracts/` NEVER imports `services/` or `clients/`. Services and clients consume contracts only via **published, versioned packages** — never relative paths.

### Published Packages (from monorepo)

| Package | Source | Consumers | Stability |
|---------|--------|-----------|-----------|
| `aidflow-contract-types` (Rust crate) | `contracts/types/` | services | `@stable` |
| `@aidflow/contract-sdk` (npm) | `contracts/sdk/` | clients | `@stable` |
| `@aidflow/indexer-client` (npm) | `services/indexer/client/` | clients | `@beta` |

**Stability tiers (tag every public interface):**
- `@stable` — external-dependable; breaking change requires major version bump; audit before change
- `@beta` — may change; consumers must pin exact versions
- `@internal` — monorepo-only; never published

### Contract Interfaces (§5 - Implement From These)

```rust
// Config / AccessControl
fn init(env, admin: Address /*multisig*/, oracle: Address);
fn set_admin(env, new_admin: Address);      // require_auth(admin)
fn set_oracle(env, oracle: Address);        // require_auth(admin)
fn pause(env); 
fn unpause(env); 
fn is_paused(env) -> bool;

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

**Events:** program_created, funded, milestone_attested{evidence_hash}, released, voucher_issued, voucher_claimed, voucher_redeemed, voucher_expired, refunded, paused, unpaused.

**Design principles:**
- Batch issuance: ≤100 recipients per transaction, paginate larger batches
- Lazy claim/redeem: pull-based to minimize gas
- Store 32-byte hashes only (evidence off-chain)
- Explicit TTL/rent per voucher
- Fee sponsorship via relayer service
- Upgradability: `update_current_contract_wasm` behind multisig + 48-72h timelock + `upgrade_scheduled` event
- Schema versioning: `schema_version` field + lazy migration on touch

---

## Proposed Solution

Build AidFlow in 9 sequential modules following the checkpoint protocol (§9). Each module scaffolds a layer, verifies it builds/tests, reports status with summary output, then proceeds automatically to the next. Use official tooling (stellar-cli for contracts, vite/expo scaffolders for clients, go mod tidy for services). Flag uncertain APIs with `AGENT-FLAG` comments. Generate contributor guardrails early (Module 1) to prevent stale-knowledge PRs from reaching the review queue.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        CLIENTS                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Donor        │  │ Beneficiary  │  │ Merchant     │     │
│  │ Dashboard    │  │ PWA          │  │ App          │     │
│  │ (Vite+React) │  │ (Vite+React) │  │ (Expo/RN)    │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘     │
│         │                  │                  │              │
│         └──────────────────┼──────────────────┘              │
│                            │                                 │
│                   @aidflow/contract-sdk (TS)                │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┼─────────────────────────────────┐
│                     SERVICES (Go)                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │Onboarding│  │  Oracle  │  │ Indexer  │  │ Relayer  │   │
│  │(CSV/PII) │  │(Attest)  │  │(Events)  │  │(Fees)    │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │               │             │          │
│       └─────────────┼───────────────┼─────────────┘          │
│                     │               │                        │
│              github.com/stellar/go-stellar-sdk              │
└─────────────────────┼───────────────┼──────────────────────┘
                      │               │
┌─────────────────────┼───────────────┼──────────────────────┐
│            STELLAR/SOROBAN CONTRACTS (Rust)                │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Config/    │  │   Escrow     │  │  Voucher     │     │
│  │ AccessControl│  │  (Programs)  │  │  Registry    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│  ┌──────────────┐                                          │
│  │  Merchant    │   soroban-sdk 26.1 + stellar-cli 27.x   │
│  │  Registry    │                                          │
│  └──────────────┘                                          │
└────────────────────────────────────────────────────────────┘
                            │
                      Stellar Network
                    (Testnet → Mainnet)
```

### Build Sequence (Modules 1-9)

Each module:
1. Scaffolds its layer following spec
2. Runs verification command(s)
3. Reports "Module N: ✓" with summary (key files, command output)
4. Auto-proceeds to next module (or STOPS on ✗)

**Module 1:** Repository foundation + contributor guardrails  
**Module 2:** Contracts types + Config/AccessControl  
**Module 3:** Contracts Escrow (with unit tests)  
**Module 4:** Contracts VoucherRegistry + MerchantRegistry  
**Module 5:** Contract SDK (TypeScript)  
**Module 6:** Services indexer + relayer (Go, new SDK path)  
**Module 7:** Services onboarding + oracle (Go, new SDK path)  
**Module 8:** Clients (donor dashboard, PWA, merchant app)  
**Module 9:** MVP issues + docker-compose infrastructure  

---

## Task Breakdown

### Task 1: Repository Foundation & Contributor Guardrails

**Objective:** Create monorepo structure with npm workspaces, Apache-2.0 license, .gitignore, root README, Makefile, CI skeleton, and contributor-agent guardrails to prevent stale-knowledge PRs.

**Implementation:**

1. **Root structure:**
   - Create `package.json` with npm workspaces: `["contracts/sdk", "clients/*"]`
   - Create `LICENSE` file (Apache-2.0 full text)
   - Create `.gitignore` (node_modules, target/, dist/, .env, *.wasm, bin/)
   - Create `Makefile` with targets: `lint`, `test`, `build`, `clean`, `check-boundaries`
   - Create root `README.md`:
     - Title: "AidFlow — Transparent Aid Disbursement on Stellar"
     - Badges (build status placeholders, license)
     - Quick start (docker-compose up)
     - Links to docs/, CONTRIBUTING.md
     - Clone command: `git clone https://github.com/stellar-impact/aidflow.git`
     - Repo metadata references `stellar-impact/aidflow` throughout

2. **Contributor guardrails (§7.1):**
   - `.github/PULL_REQUEST_TEMPLATE.md`:
     ```markdown
     ## Checklist
     - [ ] I ran the issue's Verification commands locally and they pass
     - [ ] Tests pass: `make test` (or the issue-specific command)
     - [ ] I read the issue's Technical Context and used the specified APIs/files
     - [ ] (If using an AI agent) I confirmed it used pinned versions (§0.5 of docs/AIDFLOW_MASTER_PROMPT.md)
     
     Closes #<issue-number>
     ```
   
   - `CONTRIBUTING.md` with section "Using AI Agents to Complete Issues":
     - Always verify locally first; CI failures may result in PR being closed
     - Pin versions exactly: `soroban-sdk` 26.1, `stellar-cli` 27.x, `go-stellar-sdk` new path
     - Read issue's Technical Context; don't let agent invent APIs
     - If agent's code won't compile, ask in issue comments — don't submit broken PRs
     - **Common agent mistakes to watch for:**
       - `soroban contract build` → should be `stellar contract build`
       - `github.com/stellar/go` → should be `github.com/stellar/go-stellar-sdk`
       - `cargo build` for contracts → should be `stellar contract build`
     - Include general contribution guidelines (fork, branch, PR process)
   
   - `.github/workflows/pr-ci-guard.yml`:
     - Triggers on `workflow_run` completion from contracts-ci, services-ci, clients-ci
     - If conclusion == 'failure', posts idempotent comment on associated PR
     - Comment reminds contributor to run verification locally, check pinned versions
     - Uses `actions/github-script@v7` to update existing bot comment
   
   - `.github/ISSUE_TEMPLATE/drips_wave_contribution.md`:
     - Template with labels `["Stellar Wave", "needs-triage"]`
     - Sections: Description, Acceptance Criteria, Technical Context
     - **Verification block** (like §6) with tailored commands
     - "Using an AI agent?" reminder paragraph

3. **CI workflow skeletons:**
   - `.github/workflows/boundaries-check.yml`:
     - Runs on every PR
     - Job: `check-boundaries`
     - Steps: checkout, run `./scripts/check-boundaries.sh`
     - Fails if contracts/ imports services/ or relative paths detected
   
   - `.github/workflows/contracts-ci.yml`:
     - Triggers: push to main, PR
     - Jobs: fmt (cargo fmt --check), clippy (cargo clippy -- -D warnings), test (cargo test --all), build (stellar contract build), audit (cargo audit)
     - Uses Rust 1.84+, installs stellar-cli 27.x
   
   - `.github/workflows/services-ci.yml`:
     - Jobs: fmt (go fmt ./...), lint (golangci-lint), test (go test ./...), build (go build ./...), docker-build
     - Uses Go 1.23+
   
   - `.github/workflows/clients-ci.yml`:
     - Jobs: lint (eslint), test (vitest/jest), build (vite build / expo), e2e (Playwright on main branch only)
     - Uses Node 20+
   
   - `.github/workflows/deploy-testnet.yml`:
     - Triggers: manual (workflow_dispatch), push to main
     - Builds contracts, deploys to Stellar testnet, initializes with admin/oracle
     - Placeholder for now (will be fleshed out post-MVP)

4. **Documentation structure:**
   - `docs/ARCHITECTURE.md` (stub with overview, will be fleshed out in issue)
   - `docs/EXTRACTION_PLAN.md`:
     - Phase 1: Extract contracts/ to `stellar-impact/aidflow-contracts` (Wave 7-8 post-audit)
     - Phase 2: Optionally extract indexer if external demand
     - Phase 3: No further splits
     - Process: `git filter-repo`, publish packages, update monorepo to consume from registry
   - `docs/CONTRACT_INTERFACES.md`: Copy of §5 interface definitions with stability markers
   - `docs/API.md` (stub for services API specs)
   - `docs/SECURITY.md` (stub with security policy, will add audit checklist in issue)
   - `docs/adr/` (Architecture Decision Records):
     - `0001-use-soroban-for-escrow.md` (why Soroban vs EVM)
     - `0002-monorepo-with-extraction-plan.md` (why monorepo now, extract later)

5. **Boundary enforcement script:**
   - `scripts/check-boundaries.sh`:
     ```bash
     #!/bin/bash
     set -e
     
     echo "Checking dependency boundaries..."
     
     # Fail if contracts/ imports services/ or clients/
     if grep -r "use.*services::" contracts/ 2>/dev/null || \
        grep -r "use.*clients::" contracts/ 2>/dev/null || \
        grep -r "import.*services/" contracts/sdk/ 2>/dev/null; then
         echo "ERROR: contracts/ must not import services/ or clients/"
         exit 1
     fi
     
     # Fail if services/ or clients/ use relative paths into contracts/ (must use published packages)
     if grep -r "use.*\\.\\./\\.\\./contracts/" services/ 2>/dev/null || \
        grep -r "import.*\\.\\./\\.\\./contracts/" clients/ 2>/dev/null; then
         echo "ERROR: services/ and clients/ must use published packages, not relative paths"
         exit 1
     fi
     
     echo "✓ Dependency boundaries OK"
     ```
   - Make executable: `chmod +x scripts/check-boundaries.sh`

6. **Directory tree:**
   ```
   contracts/
     types/ config/ escrow/ voucher_registry/ merchant_registry/ sdk/
     Cargo.toml EXTRACTION.md scripts/{build,test,deploy-testnet}.sh
   services/
     cmd/{onboarding,oracle,indexer,relayer}/
     internal/{stellar,crypto,db}/
     pkg/indexer-client/
     migrations/
     go.mod EXTRACTION.md scripts/{build,test}.sh
   clients/
     donor-dashboard/ beneficiary-pwa/ merchant-app/
     README.md
   infra/
     docker-compose.yml
     k8s/{base,overlays/{testnet,mainnet}}/
     helm/
   docs/
     {ARCHITECTURE,EXTRACTION_PLAN,CONTRACT_INTERFACES,API,SECURITY}.md
     adr/
     issues/mvp/ (for Task 9)
   scripts/
     {setup-dev,seed-testnet,check-boundaries}.sh
   .github/
     workflows/ ISSUE_TEMPLATE/ PULL_REQUEST_TEMPLATE.md
   ```

**Technical Context:**
- License: Apache-2.0 (permissive, aligns with Stellar ecosystem)
- Workspaces: npm workspaces for TS packages, Cargo workspace for contracts, Go modules for services
- GitHub: `stellar-impact` org, `aidflow` repo — use in all clone URLs, package metadata
- Boundary script: Parse imports/use statements, fail CI on violations
- CI: Use latest stable actions (`actions/checkout@v4`, `actions/setup-go@v5`, etc.)

**Tests:**
- `make lint` runs without errors (may be no-op initially)
- `./scripts/check-boundaries.sh` passes (no violations yet)
- Directory structure present: `ls -d contracts/ services/ clients/ infra/ docs/ scripts/ .github/`
- Templates exist: `cat .github/PULL_REQUEST_TEMPLATE.md`, `cat CONTRIBUTING.md | grep "Using AI Agents"`

**Verification Commands:**
```bash
make lint
./scripts/check-boundaries.sh
tree -L 2 -I 'node_modules|target'
```

**Demo:**
After completion, running `tree -L 3 -I 'node_modules|target'` shows complete repo skeleton. `cat .github/PULL_REQUEST_TEMPLATE.md` displays verification checklist. `cat CONTRIBUTING.md | grep -A 10 "Using AI Agents"` shows guardrails section warning about pinned versions and common mistakes (`soroban`→`stellar` CLI, old Go import path, cargo build trap).

---

### Task 2: Contracts - Types & Config/AccessControl Module

**Objective:** Scaffold Rust contracts workspace with shared types crate and Config/AccessControl contract implementing admin multisig, oracle address, and circuit breaker (pause/unpause). Verify with `stellar contract build` and unit tests.

**Implementation:**

1. **Contracts workspace:**
   - `contracts/Cargo.toml`:
     ```toml
     [workspace]
     members = ["types", "config", "escrow", "voucher_registry", "merchant_registry"]
     resolver = "2"
     
     [workspace.dependencies]
     soroban-sdk = "26.1"
     ```

2. **Shared types crate:**
   - `contracts/types/Cargo.toml`:
     ```toml
     [package]
     name = "aidflow-contract-types"
     version = "0.1.0"
     edition = "2021"
     
     [dependencies]
     soroban-sdk = { workspace = true }
     
     [lib]
     crate-type = ["lib"]
     ```
   
   - `contracts/types/src/lib.rs`:
     - `#![no_std]` (Soroban environment)
     - `use soroban_sdk::{contracttype, Address, String, Vec};`
     - Define `#[contracttype] #[derive(Clone)] pub struct Milestone { pub id: u32, pub description: String, pub target_amount: i128 }`
     - Define `#[contracttype] #[derive(Clone, PartialEq)] pub enum ProgramStatus { Active, Completed, Refunded }`
     - Define `#[contracttype] #[derive(Clone, PartialEq)] pub enum VoucherStatus { Unclaimed, Claimed, Expired }`
     - Add `#[doc = "@stable"]` markers on all pub types per §3
     - If `contracttype` macro doesn't exist in 26.1, use `#[derive(soroban_sdk::contracttype)]` or FLAG

3. **Config/AccessControl contract:**
   - `contracts/config/Cargo.toml`:
     ```toml
     [package]
     name = "aidflow-config"
     version = "0.1.0"
     edition = "2021"
     
     [dependencies]
     soroban-sdk = { workspace = true }
     aidflow-contract-types = { path = "../types" }
     
     [lib]
     crate-type = ["cdylib"]
     ```
   
   - `contracts/config/src/lib.rs`:
     - `#![no_std]` + `use soroban_sdk::{contract, contractimpl, Address, Env, symbol_short};`
     - Storage keys: `const ADMIN: Symbol = symbol_short!("admin");` (similarly for ORACLE, PAUSED)
     - `#[contract] pub struct ConfigContract;`
     - `#[contractimpl] impl ConfigContract { ... }` with functions:
       - `pub fn init(env: Env, admin: Address, oracle: Address)` — store admin/oracle in instance storage
       - `pub fn set_admin(env: Env, new_admin: Address)` — `admin.require_auth();` then update storage
       - `pub fn set_oracle(env: Env, new_oracle: Address)` — admin auth required
       - `pub fn pause(env: Env)` — admin auth, set PAUSED=true
       - `pub fn unpause(env: Env)` — admin auth, set PAUSED=false
       - `pub fn is_paused(env: Env) -> bool` — read PAUSED flag
       - `pub fn get_admin(env: Env) -> Address` — helper for cross-contract calls
       - `pub fn get_oracle(env: Env) -> Address` — helper
     - Storage: `env.storage().instance()` for admin/oracle (contract-lifetime)
     - If `require_auth` method signature changed in 26.1, FLAG it: `// AGENT-FLAG: verify require_auth API in soroban-sdk 26.1`
   
   - **Unit tests** (in same file or `tests/` module):
     ```rust
     #[cfg(test)]
     mod tests {
         use super::*;
         use soroban_sdk::testutils::{Address as _, Ledger};
         
         #[test]
         fn test_init_sets_admin() {
             let env = Env::default();
             let contract_id = env.register_contract(None, ConfigContract);
             let client = ConfigContractClient::new(&env, &contract_id);
             let admin = Address::generate(&env);
             let oracle = Address::generate(&env);
             
             client.init(&admin, &oracle);
             
             assert_eq!(client.get_admin(), admin);
         }
         
         #[test]
         #[should_panic(expected = "require_auth")]
         fn test_pause_requires_admin() {
             let env = Env::default();
             let contract_id = env.register_contract(None, ConfigContract);
             let client = ConfigContractClient::new(&env, &contract_id);
             let admin = Address::generate(&env);
             let oracle = Address::generate(&env);
             let non_admin = Address::generate(&env);
             
             client.init(&admin, &oracle);
             
             // Don't mock auths - should panic without admin auth
             client.pause();
         }
     }
     ```
     - If test API changed (e.g., `testutils` module), FLAG and adapt or use placeholder

4. **Build and test scripts:**
   - `contracts/scripts/build.sh`:
     ```bash
     #!/bin/bash
     set -e
     cd "$(dirname "$0")/.."
     
     echo "Building contracts with stellar-cli 27.x..."
     stellar contract build --package aidflow-config
     # Add more packages as they're created
     
     echo "✓ Contracts built successfully"
     ls -lh target/wasm32-unknown-unknown/release/*.wasm
     ```
   
   - `contracts/scripts/test.sh`:
     ```bash
     #!/bin/bash
     set -e
     cd "$(dirname "$0")/.."
     
     echo "Running contract tests..."
     cargo test --all
     
     echo "✓ All tests passed"
     ```
   
   - Make executable: `chmod +x contracts/scripts/{build,test}.sh`

**Technical Context:**
- **Stellar CLI 27.x:** Command is `stellar contract build`, NOT `soroban contract build`
- **soroban-sdk 26.1.0:** Docs at https://docs.rs/soroban-sdk/26.1.0/
- **Rust 1.84+:** Required for wasm32v1-none target (automatically selected by stellar-cli)
- **Storage types:**
  - `env.storage().instance()` — contract instance lifetime, no TTL management needed
  - `env.storage().persistent()` — needs explicit TTL extension, use for PAUSED if desired
- **Authorization:** `address.require_auth()` enforces auth before mutations
- **Symbols:** `symbol_short!("key")` for storage keys ≤9 chars; use `Symbol::new()` for longer

**Tests:**
- `cd contracts && cargo test --package aidflow-config` passes both unit tests
- `cd contracts && stellar contract build --package aidflow-config` produces `target/wasm32-unknown-unknown/release/aidflow_config.wasm`
- No warnings about deprecated APIs or auth failures

**Verification Commands:**
```bash
cd contracts
cargo test --package aidflow-config
stellar contract build --package aidflow-config
cargo clippy --package aidflow-config -- -D warnings
```

**Demo:**
After completion, `ls -lh contracts/target/wasm32-unknown-unknown/release/*.wasm` shows compiled wasm. `cargo test --package aidflow-config -- --nocapture` output displays test assertions passing (init sets admin, unauthorized pause fails). `grep "AGENT-FLAG" contracts/config/src/lib.rs` surfaces any flagged APIs for review.

---

### Task 3: Contracts - Escrow Module with Unit Tests

**Objective:** Implement Escrow contract managing programs, milestones, funding, attestation (oracle), and release (admin). Verify with `stellar contract build` and comprehensive unit tests.

**Implementation:**

1. **Escrow contract:**
   - `contracts/escrow/Cargo.toml`:
     ```toml
     [package]
     name = "aidflow-escrow"
     version = "0.1.0"
     edition = "2021"
     
     [dependencies]
     soroban-sdk = { workspace = true }
     aidflow-contract-types = { path = "../types" }
     
     [lib]
     crate-type = ["cdylib"]
     ```
   
   - `contracts/escrow/src/lib.rs`:
     - `#![no_std]` + imports
     - Define `#[contracttype] struct Program { id: u64, funder: Address, token: Address, milestones: Vec<Milestone>, status: ProgramStatus, funded_amount: i128, released_amount: i128 }`
     - Define `#[contracttype] struct AttestationRecord { milestone_id: u32, evidence_hash: BytesN<32>, attested_at: u64 }`
     - Storage keys: `PROGRAM_COUNTER`, `PROGRAMS` (map u64→Program), `ATTESTATIONS` (map (program_id, milestone_id)→AttestationRecord)
     - `#[contract] pub struct EscrowContract;`
     - `#[contractimpl] impl EscrowContract { ... }` with functions:
       
       - `pub fn init(env: Env, config_contract: Address)` — store config contract address for auth checks
       
       - `pub fn create_program(env: Env, funder: Address, token: Address, milestones: Vec<Milestone>) -> u64`:
         - `funder.require_auth()`
         - Increment PROGRAM_COUNTER, get new program_id
         - Create Program struct with id, funder, token, milestones, status=Active, funded/released=0
         - Store in PROGRAMS map
         - Emit event `env.events().publish((symbol_short!("created"),), program_id)`
         - Return program_id
       
       - `pub fn fund(env: Env, program_id: u64, amount: i128)`:
         - Load program, verify exists and status=Active
         - `program.funder.require_auth()`
         - Transfer tokens from funder to contract: `TokenClient::new(&env, &program.token).transfer(&program.funder, &env.current_contract_address(), &amount)` (FLAG if API changed)
         - Update program.funded_amount += amount
         - Store updated program
         - Emit `funded` event
       
       - `pub fn attest_milestone(env: Env, program_id: u64, milestone_id: u32, evidence_hash: BytesN<32>)`:
         - Load config contract, get oracle address: `ConfigContractClient::new(&env, &config_addr).get_oracle()` (FLAG if client gen changed)
         - `oracle.require_auth()`
         - Verify program exists, milestone_id valid
         - Store AttestationRecord with hash + timestamp
         - Emit `milestone_attested{program_id, milestone_id, evidence_hash}` event
       
       - `pub fn release(env: Env, program_id: u64, milestone_id: u32)`:
         - Load config contract, get admin address
         - `admin.require_auth()`
         - Verify milestone attested (load AttestationRecord)
         - Get milestone target_amount from program
         - Transfer tokens to VoucherRegistry contract (or mark ready if not integrated): `TokenClient::new(...).transfer(&env.current_contract_address(), &voucher_registry, &milestone.target_amount)`
         - Update program.released_amount += milestone.target_amount
         - If all milestones released, set status=Completed
         - Emit `released{program_id, milestone_id, amount}` event
       
       - `pub fn refund_unspent(env: Env, program_id: u64)`:
         - Load config, get admin, require auth
         - Load program, calculate unspent = funded_amount - released_amount
         - Transfer unspent back to funder
         - Set status=Refunded
         - Emit `refunded{program_id, amount: unspent}` event
       
       - `pub fn get_program(env: Env, program_id: u64) -> Program` — read-only query
   
   - **Unit tests** (3+ tests):
     ```rust
     #[cfg(test)]
     mod tests {
         use super::*;
         use soroban_sdk::{testutils::Address as _, token, Env};
         
         #[test]
         fn test_create_program_returns_id() {
             let env = Env::default();
             let contract_id = env.register_contract(None, EscrowContract);
             let client = EscrowContractClient::new(&env, &contract_id);
             let funder = Address::generate(&env);
             let token = Address::generate(&env);
             let config = Address::generate(&env);
             
             env.mock_all_auths();
             client.init(&config);
             
             let milestones = vec![&env, Milestone { id: 1, description: String::from_str(&env, "M1"), target_amount: 1000 }];
             let program_id = client.create_program(&funder, &token, &milestones);
             
             assert_eq!(program_id, 1);
             let program = client.get_program(&program_id);
             assert_eq!(program.funder, funder);
         }
         
         #[test]
         fn test_fund_increases_balance() {
             // Setup escrow, create program, fund it, assert funded_amount updated
             // FLAG: Need to register and initialize token contract for transfer test
         }
         
         #[test]
         #[should_panic]
         fn test_attest_requires_oracle_auth() {
             // Create program, attempt attest without oracle auth, should panic
         }
     }
     ```
     - If `TokenClient` API changed or token test utils missing, FLAG

2. **Update build script:**
   - Modify `contracts/scripts/build.sh` to build escrow package:
     ```bash
     stellar contract build --package aidflow-config
     stellar contract build --package aidflow-escrow
     ```

**Technical Context:**
- **Soroban storage keys:** Use `symbol_short!("prog")` or similar for map prefix
- **Token transfers:** `TokenClient::new(&env, &token_address).transfer(&from, &to, &amount)` pattern (FLAG exact signature if uncertain)
- **Oracle/admin address:** Read from config contract via cross-contract call `ConfigClient::new(&env, &config_address).get_oracle()` (requires config contract address stored at init)
- **Milestone struct:** From types crate: `id: u32`, `description: String`, `target_amount: i128`
- **Program struct:** id, funder, token, milestones Vec, status enum, funded_amount, released_amount
- **Events:** `env.events().publish((symbol_short!("created"),), program_id)` pattern (FLAG exact macro syntax if uncertain)
- **Cross-contract integration:** release() transfers to VoucherRegistry — may stub this initially with comment `// TODO: integrate VoucherRegistry.issue_batch` and just update released_amount

**Tests:**
- `cargo test --package aidflow-escrow` passes 3+ tests
- `stellar contract build --package aidflow-escrow` produces wasm
- `cargo clippy --package aidflow-escrow -- -D warnings` has no warnings

**Verification Commands:**
```bash
cd contracts
cargo test --package aidflow-escrow
stellar contract build --package aidflow-escrow
cargo clippy --package aidflow-escrow -- -D warnings
```

**Demo:**
`cargo test --package aidflow-escrow -- --show-output` displays test names passing (create returns ID, fund increases balance, attest requires oracle). `wc -l contracts/escrow/src/lib.rs` shows substantive implementation (~200-300 lines with tests). `grep "AGENT-FLAG" contracts/escrow/src/lib.rs` surfaces any uncertain APIs flagged for review (e.g., TokenClient signature, config client gen, event macro).

---

### Task 4: Contracts - VoucherRegistry & MerchantRegistry Modules

**Objective:** Implement VoucherRegistry (batch issuance, claim, expiry) and MerchantRegistry (register, deactivate, redeem) contracts. Verify with `stellar contract build` and unit tests for both.

**Implementation:**

1. **VoucherRegistry contract:**
   - `contracts/voucher_registry/Cargo.toml`:
     ```toml
     [package]
     name = "aidflow-voucher-registry"
     version = "0.1.0"
     edition = "2021"
     
     [dependencies]
     soroban-sdk = { workspace = true }
     aidflow-contract-types = { path = "../types" }
     
     [lib]
     crate-type = ["cdylib"]
     ```
   
   - `contracts/voucher_registry/src/lib.rs`:
     - Define `#[contracttype] struct Voucher { id: u64, program_id: u64, recipient: Address, amount: i128, status: VoucherStatus, expiry: u64, claimed_at: Option<u64> }`
     - Storage: `VOUCHER_COUNTER`, `VOUCHERS` (map u64→Voucher), `ESCROW_CONTRACT`, `TOKEN_ADDRESS`
     - `#[contract] pub struct VoucherRegistryContract;`
     - `#[contractimpl] impl VoucherRegistryContract { ... }`:
       
       - `pub fn init(env: Env, admin: Address, escrow_contract: Address, token: Address)` — store escrow/token addresses
       
       - `pub fn issue_batch(env: Env, program_id: u64, recipients: Vec<(Address, i128)>, expiry: u64) -> Vec<u64>`:
         - Load escrow_contract address, `escrow_contract.require_auth()` (only escrow can issue)
         - Enforce `recipients.len() <= 100` (panic if exceeded)
         - For each recipient:
           - Increment VOUCHER_COUNTER, get new voucher_id
           - Create Voucher struct (id, program_id, recipient, amount, status=Unclaimed, expiry, claimed_at=None)
           - Store in VOUCHERS map
           - Emit `voucher_issued{voucher_id, program_id, recipient, amount}` event
         - Return Vec of voucher_ids
       
       - `pub fn claim(env: Env, voucher_id: u64)`:
         - Load voucher, verify exists
         - `voucher.recipient.require_auth()`
         - Verify status=Unclaimed and not expired (`env.ledger().timestamp() <= voucher.expiry`)
         - Transfer tokens from contract to recipient: `TokenClient::new(&env, &token_address).transfer(&env.current_contract_address(), &voucher.recipient, &voucher.amount)`
         - Update voucher: status=Claimed, claimed_at=current timestamp
         - Emit `voucher_claimed{voucher_id, recipient, amount}` event
       
       - `pub fn expire(env: Env, voucher_id: u64)`:
         - Load voucher
         - Verify `env.ledger().timestamp() > voucher.expiry` and status=Unclaimed
         - Transfer tokens back to escrow contract: `TokenClient.transfer(&env.current_contract_address(), &escrow_contract, &voucher.amount)`
         - Update voucher: status=Expired
         - Emit `voucher_expired{voucher_id, amount}` event
       
       - `pub fn get_voucher(env: Env, voucher_id: u64) -> Voucher` — read-only query
   
   - **Unit tests** (2+ tests):
     ```rust
     #[test]
     fn test_issue_batch_creates_vouchers() {
         // Setup voucher registry, mock escrow auth, call issue_batch with 2 recipients, verify 2 vouchers created
     }
     
     #[test]
     fn test_claim_transfers_tokens() {
         // Issue voucher, mock recipient auth, claim it, verify status=Claimed
         // FLAG: Token transfer in test may need token contract registration
     }
     
     #[test]
     #[should_panic]
     fn test_issue_batch_rejects_over_100() {
         // Attempt issue_batch with 101 recipients, should panic
     }
     ```

2. **MerchantRegistry contract:**
   - `contracts/merchant_registry/Cargo.toml`:
     ```toml
     [package]
     name = "aidflow-merchant-registry"
     version = "0.1.0"
     edition = "2021"
     
     [dependencies]
     soroban-sdk = { workspace = true }
     aidflow-contract-types = { path = "../types" }
     
     [lib]
     crate-type = ["cdylib"]
     ```
   
   - `contracts/merchant_registry/src/lib.rs`:
     - Define `#[contracttype] struct Merchant { address: Address, payout: Address, category: u32, active: bool }`
     - Storage: `MERCHANTS` (map Address→Merchant), `ADMIN`, `TOKEN_ADDRESS`
     - `#[contract] pub struct MerchantRegistryContract;`
     - `#[contractimpl] impl MerchantRegistryContract { ... }`:
       
       - `pub fn init(env: Env, admin: Address, token: Address)` — store admin/token
       
       - `pub fn register(env: Env, merchant: Address, payout: Address, category: u32)`:
         - Load admin, `admin.require_auth()`
         - Create Merchant struct (address=merchant, payout, category, active=true)
         - Store in MERCHANTS map
         - Emit `merchant_registered{merchant, category}` event
       
       - `pub fn deactivate(env: Env, merchant: Address)`:
         - Load admin, `admin.require_auth()`
         - Load merchant entry, set active=false
         - Update storage
         - Emit `merchant_deactivated{merchant}` event
       
       - `pub fn redeem(env: Env, from: Address, merchant: Address, amount: i128)`:
         - `from.require_auth()` (beneficiary authorizes transfer)
         - Load merchant entry, verify active=true (panic if inactive)
         - Transfer tokens from beneficiary to merchant payout: `TokenClient::new(&env, &token_address).transfer(&from, &merchant.payout, &amount)`
         - Emit `voucher_redeemed{from, merchant, amount}` event (or `merchant_redeem` event)
       
       - `pub fn is_active(env: Env, merchant: Address) -> bool`:
         - Load merchant, return active flag (or false if not found)
   
   - **Unit tests** (2+ tests):
     ```rust
     #[test]
     fn test_register_stores_merchant() {
         // Setup merchant registry, register merchant, verify is_active=true
     }
     
     #[test]
     #[should_panic]
     fn test_redeem_requires_active_merchant() {
         // Register merchant, deactivate it, attempt redeem, should panic
     }
     ```

3. **Update build script:**
   - Modify `contracts/scripts/build.sh` to build all contracts:
     ```bash
     stellar contract build --package aidflow-config
     stellar contract build --package aidflow-escrow
     stellar contract build --package aidflow-voucher-registry
     stellar contract build --package aidflow-merchant-registry
     ```

**Technical Context:**
- **Batch limit:** Check `recipients.len() <= 100` and return error/panic if exceeded per §5 design
- **Expiry check:** `env.ledger().timestamp()` returns block timestamp (FLAG if method name changed in 26.1)
- **Cross-contract auth:** `escrow_contract.require_auth()` when issue_batch called (FLAG pattern if uncertain)
- **Token transfers:** Same `TokenClient` pattern as escrow (from→to)
- **Merchant struct:** address, payout_address (where funds go), category (u32 enum like food/health/edu), active bool
- **VoucherView:** Can return Voucher directly or define a separate View type if sensitive fields
- **Storage:** Use persistent storage with TTL for vouchers (extend TTL on claim/expire), instance storage for admin/config

**Tests:**
- `cargo test --all` in contracts/ passes all tests (config + escrow + voucher + merchant)
- `stellar contract build` (no specific package flag, or build all explicitly) produces 4 wasm files
- `cargo clippy --all -- -D warnings` has no warnings

**Verification Commands:**
```bash
cd contracts
cargo test --all
stellar contract build --package aidflow-voucher-registry
stellar contract build --package aidflow-merchant-registry
cargo clippy --all -- -D warnings
```

**Demo:**
`ls contracts/target/wasm32-unknown-unknown/release/*.wasm` shows 4 wasm files (config, escrow, voucher_registry, merchant_registry). `cargo test --all -- --test-threads=1` runs all contract tests sequentially and displays pass count. `grep -r "AGENT-FLAG" contracts/` lists any flagged APIs for review (e.g., escrow auth pattern, timestamp method).

---

### Task 5: Contract SDK - TypeScript Client Library

**Objective:** Create TypeScript SDK package `@aidflow/contract-sdk` that wraps contract invocations with typed interfaces. Published as npm package, consumed by all clients. Verify with `npm run build`.

**Implementation:**

1. **Package setup:**
   - `contracts/sdk/package.json`:
     ```json
     {
       "name": "@aidflow/contract-sdk",
       "version": "0.1.0-alpha",
       "description": "TypeScript SDK for AidFlow Soroban contracts",
       "type": "module",
       "main": "./dist/index.js",
       "types": "./dist/index.d.ts",
       "exports": {
         ".": {
           "import": "./dist/index.js",
           "types": "./dist/index.d.ts"
         }
       },
       "repository": {
         "type": "git",
         "url": "https://github.com/stellar-impact/aidflow.git",
         "directory": "contracts/sdk"
       },
       "keywords": ["stellar", "soroban", "aidflow", "contracts"],
       "license": "Apache-2.0",
       "scripts": {
         "build": "tsup src/index.ts --format esm --dts",
         "prepublishOnly": "npm run build"
       },
       "dependencies": {
         "@stellar/stellar-sdk": "^13.0.0"
       },
       "devDependencies": {
         "typescript": "~5.7.0",
         "tsup": "^8.0.0"
       }
     }
     ```
     - Note: `@stellar/stellar-sdk` version should be latest stable (13.x or 14.x as of 2026, FLAG if uncertain)
   
   - `contracts/sdk/tsconfig.json`:
     ```json
     {
       "compilerOptions": {
         "target": "ES2022",
         "module": "ESNext",
         "lib": ["ES2022"],
         "declaration": true,
         "outDir": "./dist",
         "strict": true,
         "esModuleInterop": true,
         "skipLibCheck": true,
         "moduleResolution": "bundler"
       },
       "include": ["src/**/*"]
     }
     ```

2. **Type definitions:**
   - `contracts/sdk/src/types.ts`:
     ```typescript
     /**
      * @stable
      * Core types mirroring Rust contract types
      */
     
     export interface Milestone {
       id: number;
       description: string;
       target_amount: bigint;
     }
     
     export enum ProgramStatus {
       Active = "Active",
       Completed = "Completed",
       Refunded = "Refunded"
     }
     
     export interface Program {
       id: bigint;
       funder: string;
       token: string;
       milestones: Milestone[];
       status: ProgramStatus;
       funded_amount: bigint;
       released_amount: bigint;
     }
     
     export enum VoucherStatus {
       Unclaimed = "Unclaimed",
       Claimed = "Claimed",
       Expired = "Expired"
     }
     
     export interface Voucher {
       id: bigint;
       program_id: bigint;
       recipient: string;
       amount: bigint;
       status: VoucherStatus;
       expiry: bigint;
       claimed_at?: bigint;
     }
     
     export interface Merchant {
       address: string;
       payout: string;
       category: number;
       active: boolean;
     }
     ```

3. **Contract client classes:**
   - `contracts/sdk/src/config.ts`:
     ```typescript
     /** @stable */
     import { Contract, Server } from '@stellar/stellar-sdk';
     // AGENT-FLAG: Verify Contract invocation API in @stellar/stellar-sdk v13+
     
     export class ConfigContract {
       private contract: Contract;
       private server: Server;
       
       constructor(contractId: string, server: Server) {
         this.contract = new Contract(contractId);
         this.server = server;
         // FLAG: Verify Contract constructor signature
       }
       
       async init(admin: string, oracle: string): Promise<string> {
         // FLAG: Placeholder - verify Soroban contract invocation pattern
         // Should construct transaction with contract.call('init', admin, oracle)
         // Sign, submit to server, return tx hash
         throw new Error('Not implemented - verify stellar-sdk Soroban API');
       }
       
       async setAdmin(newAdmin: string): Promise<string> {
         // Similar pattern
         throw new Error('Not implemented');
       }
       
       async pause(): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async isPaused(): Promise<boolean> {
         // Read-only query, may use different API (simulateTransaction?)
         throw new Error('Not implemented');
       }
       
       async getAdmin(): Promise<string> {
         throw new Error('Not implemented');
       }
     }
     ```
     - **Important:** The exact API for invoking Soroban contracts from JS is uncertain. The agent should FLAG this and use placeholder implementations. Common patterns might be:
       - `Contract.call(methodName, ...args)` → build operation
       - `TransactionBuilder` with `Operation.invokeContractFunction`
       - Or newer `contract.call()` helper methods
       - Read operations might use `server.simulateTransaction()` or `server.call()`
     - Add JSDoc `@stable` markers
   
   - `contracts/sdk/src/escrow.ts`:
     ```typescript
     /** @stable */
     import { Contract, Server } from '@stellar/stellar-sdk';
     import { Milestone, Program } from './types.js';
     
     export class EscrowContract {
       private contract: Contract;
       private server: Server;
       
       constructor(contractId: string, server: Server) {
         this.contract = new Contract(contractId);
         this.server = server;
       }
       
       async createProgram(funder: string, token: string, milestones: Milestone[]): Promise<bigint> {
         // AGENT-FLAG: Verify contract invocation API
         throw new Error('Not implemented - verify stellar-sdk Soroban API');
       }
       
       async fund(programId: bigint, amount: bigint): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async attestMilestone(programId: bigint, milestoneId: number, evidenceHash: Uint8Array): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async release(programId: bigint, milestoneId: number): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async refundUnspent(programId: bigint): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async getProgram(programId: bigint): Promise<Program> {
         throw new Error('Not implemented');
       }
     }
     ```
   
   - `contracts/sdk/src/voucher.ts`:
     ```typescript
     /** @stable */
     import { Contract, Server } from '@stellar/stellar-sdk';
     import { Voucher } from './types.js';
     
     export class VoucherRegistryContract {
       private contract: Contract;
       private server: Server;
       
       constructor(contractId: string, server: Server) {
         this.contract = new Contract(contractId);
         this.server = server;
       }
       
       async issueBatch(programId: bigint, recipients: [string, bigint][], expiry: bigint): Promise<bigint[]> {
         // AGENT-FLAG: Verify contract invocation API
         throw new Error('Not implemented');
       }
       
       async claim(voucherId: bigint): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async expire(voucherId: bigint): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async getVoucher(voucherId: bigint): Promise<Voucher> {
         throw new Error('Not implemented');
       }
     }
     ```
   
   - `contracts/sdk/src/merchant.ts`:
     ```typescript
     /** @stable */
     import { Contract, Server } from '@stellar/stellar-sdk';
     import { Merchant } from './types.js';
     
     export class MerchantRegistryContract {
       private contract: Contract;
       private server: Server;
       
       constructor(contractId: string, server: Server) {
         this.contract = new Contract(contractId);
         this.server = server;
       }
       
       async register(merchant: string, payout: string, category: number): Promise<string> {
         // AGENT-FLAG: Verify contract invocation API
         throw new Error('Not implemented');
       }
       
       async deactivate(merchant: string): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async redeem(from: string, merchant: string, amount: bigint): Promise<string> {
         throw new Error('Not implemented');
       }
       
       async isActive(merchant: string): Promise<boolean> {
         throw new Error('Not implemented');
       }
     }
     ```

4. **Index exports:**
   - `contracts/sdk/src/index.ts`:
     ```typescript
     /**
      * @aidflow/contract-sdk
      * @stable
      * 
      * TypeScript SDK for AidFlow Soroban contracts.
      * 
      * @example
      * ```typescript
      * import { EscrowContract, Server } from '@aidflow/contract-sdk';
      * 
      * const server = new Server('https://soroban-testnet.stellar.org');
      * const escrow = new EscrowContract('CONTRACT_ID', server);
      * const programId = await escrow.createProgram(funder, token, milestones);
      * ```
      */
     
     export * from './types.js';
     export { ConfigContract } from './config.js';
     export { EscrowContract } from './escrow.js';
     export { VoucherRegistryContract } from './voucher.js';
     export { MerchantRegistryContract } from './merchant.js';
     
     // Re-export commonly used stellar-sdk types
     export { Server, Keypair, Networks } from '@stellar/stellar-sdk';
     ```

5. **README:**
   - `contracts/sdk/README.md`:
     ```markdown
     # @aidflow/contract-sdk
     
     TypeScript SDK for interacting with AidFlow Soroban smart contracts on Stellar.
     
     ## Installation
     
     ```bash
     npm install @aidflow/contract-sdk
     ```
     
     ## Usage
     
     ```typescript
     import { EscrowContract, Server, Keypair, Networks } from '@aidflow/contract-sdk';
     
     const server = new Server('https://soroban-testnet.stellar.org');
     const keypair = Keypair.fromSecret('S...');
     
     const escrow = new EscrowContract('CONTRACT_ID_HERE', server);
     
     // Create a program
     const programId = await escrow.createProgram(
       keypair.publicKey(),
       'TOKEN_CONTRACT_ID',
       [{ id: 1, description: 'Milestone 1', target_amount: 1000n }]
     );
     ```
     
     ## API Reference
     
     See TypeScript types for full API documentation.
     
     ## Stability
     
     All exported types and classes are marked `@stable` and follow semantic versioning.
     
     ## Repository
     
     https://github.com/stellar-impact/aidflow
     ```

**Technical Context:**
- **Stellar SDK for JS:** `@stellar/stellar-sdk` is the current official package (not `stellar-sdk` or `stellar-base` alone)
- **Soroban client:** As of 2026, Soroban contract invocation from JS uses `@stellar/stellar-sdk` Contract/Soroban APIs (formerly separate `soroban-client` package may be merged). FLAG if uncertain.
- **Contract invocation pattern (generic—agent should verify):**
  - Build transaction with `Contract` class and operation
  - Sign with keypair
  - Submit to Horizon/Soroban RPC via `Server.sendTransaction()`
  - Example (may be outdated): `const contract = new Contract(contractId); const tx = contract.call('method_name', ...args); await server.sendTransaction(tx);`
  - FLAG exact syntax if uncertain
- **Read operations:** May use `server.simulateTransaction()` or dedicated query methods
- **Type mapping:** Rust `i128`/`u64` → JS `bigint`, `Address` → `string` (Stellar G... format), `Vec<T>` → `T[]`, `BytesN<32>` → `Uint8Array` or hex string
- **Mark all exported types/classes with `/** @stable */` JSDoc per §3**

**Tests:**
- `cd contracts/sdk && npm install` succeeds
- `npm run build` succeeds, produces `dist/index.js` and `dist/index.d.ts`
- `ls contracts/sdk/dist/` shows built files
- TypeScript compilation passes (no type errors)
- (No runtime tests yet since implementations are placeholders—real tests come in client integration)

**Verification Commands:**
```bash
cd contracts/sdk
npm install
npm run build
ls dist/
```

**Demo:**
`cat contracts/sdk/dist/index.d.ts | head -40` shows typed exports (ConfigContract, EscrowContract, types). `cat contracts/sdk/package.json` displays `@aidflow/contract-sdk` metadata with `stellar-impact/aidflow` repo URL. `grep "AGENT-FLAG" contracts/sdk/src/*.ts` surfaces uncertain Soroban client APIs flagged for review (Contract invocation pattern, simulateTransaction usage).

---

### Task 6: Services - Indexer & Relayer (Go, New SDK Path)

**Objective:** Create Go services for indexer (Horizon event stream → Postgres) and relayer (Redis queue → fee-bump sponsorship). Verify with `go build ./...` and `go test ./...`.

**Implementation:**

1. **Go module setup:**
   - `services/go.mod`:
     ```go
     module github.com/stellar-impact/aidflow/services
     
     go 1.23
     
     require (
         github.com/stellar/go-stellar-sdk v0.0.0 // AGENT-FLAG: Verify latest version tag
         github.com/jackc/pgx/v5 v5.7.0
         github.com/redis/go-redis/v9 v9.7.0
         github.com/prometheus/client_golang v1.20.0
     )
     ```
   - Run `go mod tidy` to resolve latest compatible versions
   - **CRITICAL:** Import path is `github.com/stellar/go-stellar-sdk`, NOT deprecated `github.com/stellar/go`

2. **Indexer service:**
   - `services/cmd/indexer/main.go`:
     ```go
     package main
     
     import (
         "context"
         "log"
         "net/http"
         "os"
         
         "github.com/prometheus/client_golang/prometheus/promhttp"
         "github.com/stellar/go-stellar-sdk/clients/horizonclient"
         // AGENT-FLAG: Verify horizonclient import path in new SDK
     )
     
     func main() {
         // Initialize Horizon client
         horizonURL := os.Getenv("HORIZON_URL") // https://horizon-testnet.stellar.org
         if horizonURL == "" {
             horizonURL = "https://horizon-testnet.stellar.org"
         }
         
         client := horizonclient.DefaultTestNetClient
         // AGENT-FLAG: Verify client initialization pattern
         
         // Initialize DB connection (placeholder)
         // db := initDB()
         
         // Start HTTP server
         http.HandleFunc("/health", healthHandler)
         http.Handle("/metrics", promhttp.Handler())
         
         go startEventStream(client)
         
         log.Println("Indexer starting on :8080")
         if err := http.ListenAndServe(":8080", nil); err != nil {
             log.Fatal(err)
         }
     }
     
     func healthHandler(w http.ResponseWriter, r *http.Request) {
         w.WriteHeader(http.StatusOK)
         w.Write([]byte("OK"))
     }
     
     func startEventStream(client *horizonclient.Client) {
         // TODO: Connect to Horizon, stream ledger events
         // Parse contract events (program_created, funded, milestone_attested, etc.)
         // Store in Postgres
         // AGENT-FLAG: Verify SSE streaming API in horizonclient
         log.Println("Event stream placeholder - not yet implemented")
     }
     ```
   
   - `services/internal/db/postgres.go`:
     ```go
     package db
     
     import (
         "context"
         "fmt"
         "os"
         
         "github.com/jackc/pgx/v5/pgxpool"
     )
     
     func NewPool(ctx context.Context) (*pgxpool.Pool, error) {
         connStr := os.Getenv("DATABASE_URL")
         if connStr == "" {
             connStr = "postgres://aidflow:password@localhost:5432/aidflow?sslmode=disable"
         }
         
         pool, err := pgxpool.New(ctx, connStr)
         if err != nil {
             return nil, fmt.Errorf("unable to create connection pool: %w", err)
         }
         
         return pool, nil
     }
     ```
   
   - `services/internal/db/schema.sql` (placeholder):
     ```sql
     -- Placeholder schema
     -- Will be migrated via golang-migrate in migrations/
     
     CREATE TABLE programs (
         id BIGINT PRIMARY KEY,
         funder TEXT NOT NULL,
         token TEXT NOT NULL,
         status TEXT NOT NULL,
         funded_amount BIGINT DEFAULT 0,
         released_amount BIGINT DEFAULT 0,
         created_at TIMESTAMPTZ DEFAULT NOW()
     );
     
     CREATE TABLE milestones (
         id SERIAL PRIMARY KEY,
         program_id BIGINT REFERENCES programs(id),
         milestone_id INT NOT NULL,
         description TEXT,
         target_amount BIGINT,
         attested_at TIMESTAMPTZ,
         evidence_hash BYTEA
     );
     
     CREATE TABLE vouchers (
         id BIGINT PRIMARY KEY,
         program_id BIGINT REFERENCES programs(id),
         recipient TEXT NOT NULL,
         amount BIGINT NOT NULL,
         status TEXT NOT NULL,
         expiry BIGINT NOT NULL,
         claimed_at TIMESTAMPTZ
     );
     
     CREATE TABLE merchants (
         address TEXT PRIMARY KEY,
         payout TEXT NOT NULL,
         category INT,
         active BOOLEAN DEFAULT true
     );
     ```

3. **Relayer service:**
   - `services/cmd/relayer/main.go`:
     ```go
     package main
     
     import (
         "context"
         "log"
         "net/http"
         "os"
         
         "github.com/prometheus/client_golang/prometheus/promhttp"
         "github.com/redis/go-redis/v9"
     )
     
     func main() {
         // Initialize Redis client
         redisURL := os.Getenv("REDIS_URL")
         if redisURL == "" {
             redisURL = "localhost:6379"
         }
         
         rdb := redis.NewClient(&redis.Options{
             Addr: redisURL,
         })
         
         ctx := context.Background()
         if err := rdb.Ping(ctx).Err(); err != nil {
             log.Fatalf("Redis connection failed: %v", err)
         }
         
         // Start HTTP server
         http.HandleFunc("/health", healthHandler)
         http.Handle("/metrics", promhttp.Handler())
         
         go startQueueConsumer(rdb)
         
         log.Println("Relayer starting on :8081")
         if err := http.ListenAndServe(":8081", nil); err != nil {
             log.Fatal(err)
         }
     }
     
     func healthHandler(w http.ResponseWriter, r *http.Request) {
         w.WriteHeader(http.StatusOK)
         w.Write([]byte("OK"))
     }
     
     func startQueueConsumer(rdb *redis.Client) {
         // TODO: Consume fee-bump requests from Redis queue
         // Sign with sponsor keypair
         // Submit to Horizon
         // AGENT-FLAG: Verify transaction submission API in go-stellar-sdk
         log.Println("Queue consumer placeholder - not yet implemented")
     }
     ```

4. **Stellar client wrapper:**
   - `services/internal/stellar/client.go`:
     ```go
     package stellar
     
     import (
         "github.com/stellar/go-stellar-sdk/clients/horizonclient"
         // AGENT-FLAG: Verify import path - should be github.com/stellar/go-stellar-sdk/clients/horizonclient
     )
     
     // NewClient creates a Horizon client for the specified network
     func NewClient(network string) *horizonclient.Client {
         if network == "testnet" {
             return horizonclient.DefaultTestNetClient
         }
         return horizonclient.DefaultPublicNetClient
         // AGENT-FLAG: Verify DefaultTestNetClient exists in new SDK
     }
     ```

5. **Migrations:**
   - `services/migrations/001_initial_schema.up.sql`:
     ```sql
     -- Copy from internal/db/schema.sql
     CREATE TABLE programs ( ... );
     CREATE TABLE milestones ( ... );
     CREATE TABLE vouchers ( ... );
     CREATE TABLE merchants ( ... );
     ```
   
   - `services/migrations/001_initial_schema.down.sql`:
     ```sql
     DROP TABLE IF EXISTS merchants;
     DROP TABLE IF EXISTS vouchers;
     DROP TABLE IF EXISTS milestones;
     DROP TABLE IF EXISTS programs;
     ```

6. **Build and test scripts:**
   - `services/scripts/build.sh`:
     ```bash
     #!/bin/bash
     set -e
     cd "$(dirname "$0")/.."
     
     echo "Building services..."
     go build -o bin/indexer ./cmd/indexer
     go build -o bin/relayer ./cmd/relayer
     
     echo "✓ Services built successfully"
     ls -lh bin/
     ```
   
   - `services/scripts/test.sh`:
     ```bash
     #!/bin/bash
     set -e
     cd "$(dirname "$0")/.."
     
     echo "Running service tests..."
     go test ./...
     
     echo "✓ All tests passed"
     ```
   
   - Make executable: `chmod +x services/scripts/{build,test}.sh`

**Technical Context:**
- **NEW Go SDK import path (2026-04-07 migration):** `github.com/stellar/go-stellar-sdk`, NOT `github.com/stellar/go`
- **Horizon client import:** `import "github.com/stellar/go-stellar-sdk/clients/horizonclient"` (FLAG if path wrong)
- **Client initialization:** `horizonclient.DefaultTestNetClient` or `horizonclient.Client{HorizonURL: "..."}` (FLAG if API changed)
- **SSE streaming:** Horizon provides Server-Sent Events for ledger/transaction/operation streams. Pattern: `client.StreamLedgers(ctx, cursor, handler)` or similar (FLAG exact API)
- **pgx:** `github.com/jackc/pgx/v5` for Postgres (pure Go, high performance)
- **Redis:** `github.com/redis/go-redis/v9` standard client
- **Prometheus:** `github.com/prometheus/client_golang/prometheus` for metrics, `promhttp.Handler()` exposes `/metrics`
- **golang-migrate:** Use `migrate` CLI for running migrations: `migrate -path migrations -database postgres://... up`

**Tests:**
- `cd services && go build ./...` compiles without errors
- `go test ./...` passes (no real tests yet, but package structure valid)
- `go mod verify` succeeds
- `./scripts/build.sh` produces `bin/indexer` and `bin/relayer` binaries
- `./bin/indexer` runs briefly (may exit if no config, but no compile errors)

**Verification Commands:**
```bash
cd services
go mod tidy
go build ./...
go test ./...
./scripts/build.sh
file bin/indexer bin/relayer
```

**Demo:**
`file services/bin/indexer services/bin/relayer` shows ELF/Mach-O binaries. `grep "github.com/stellar/go-stellar-sdk" services/go.mod` confirms new import path (not deprecated `stellar/go`). `go run cmd/indexer/main.go` (if run in background) → `curl http://localhost:8080/health` returns 200 OK. `grep "AGENT-FLAG" services/**/*.go` lists uncertain APIs (horizonclient initialization, SSE streaming, transaction submission).

---

### Task 7: Services - Onboarding & Oracle (Go, New SDK Path)

**Objective:** Create Go services for onboarding (CSV import, PII encryption) and oracle (evidence upload → attestation). Verify with `go build ./...` and unit tests.

**Implementation:**

1. **Onboarding service:**
   - `services/cmd/onboarding/main.go`:
     ```go
     package main
     
     import (
         "encoding/csv"
         "encoding/json"
         "io"
         "log"
         "net/http"
         
         "github.com/prometheus/client_golang/prometheus/promhttp"
         "github.com/stellar-impact/aidflow/services/internal/crypto"
         "github.com/stellar-impact/aidflow/services/internal/db"
     )
     
     func main() {
         // Initialize DB
         // pool, err := db.NewPool(context.Background())
         
         // HTTP handlers
         http.HandleFunc("/health", healthHandler)
         http.HandleFunc("/api/v1/beneficiaries/import", importHandler)
         http.Handle("/metrics", promhttp.Handler())
         
         log.Println("Onboarding service starting on :8082")
         if err := http.ListenAndServe(":8082", nil); err != nil {
             log.Fatal(err)
         }
     }
     
     func healthHandler(w http.ResponseWriter, r *http.Request) {
         w.WriteHeader(http.StatusOK)
         w.Write([]byte("OK"))
     }
     
     func importHandler(w http.ResponseWriter, r *http.Request) {
         if r.Method != http.MethodPost {
             http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
             return
         }
         
         // Parse multipart form
         file, _, err := r.FormFile("file")
         if err != nil {
             http.Error(w, "Missing file", http.StatusBadRequest)
             return
         }
         defer file.Close()
         
         // Parse CSV
         reader := csv.NewReader(file)
         records, err := reader.ReadAll()
         if err != nil {
             http.Error(w, "Invalid CSV", http.StatusBadRequest)
             return
         }
         
         // Process records (name, phone, wallet_address)
         imported := 0
         for i, record := range records {
             if i == 0 { // Skip header
                 continue
             }
             if len(record) < 3 {
                 continue
             }
             
             name, phone, wallet := record[0], record[1], record[2]
             
             // Encrypt PII
             pii := map[string]string{"name": name, "phone": phone}
             encryptedPII, err := crypto.EncryptPII(pii)
             if err != nil {
                 log.Printf("Encryption failed for row %d: %v", i, err)
                 continue
             }
             
             // TODO: Insert into Postgres beneficiaries table
             // INSERT INTO beneficiaries (wallet_address, encrypted_pii) VALUES (wallet, encryptedPII)
             log.Printf("Imported beneficiary: wallet=%s (PII encrypted)", wallet)
             imported++
         }
         
         w.WriteHeader(http.StatusOK)
         json.NewEncoder(w).Encode(map[string]int{"imported": imported})
     }
     ```

2. **Oracle service:**
   - `services/cmd/oracle/main.go`:
     ```go
     package main
     
     import (
         "crypto/sha256"
         "encoding/hex"
         "io"
         "log"
         "net/http"
         "os"
         "strconv"
         
         "github.com/prometheus/client_golang/prometheus/promhttp"
         "github.com/stellar-impact/aidflow/services/internal/stellar"
         "github.com/stellar/go-stellar-sdk/keypair"
         // AGENT-FLAG: Verify keypair import path in new SDK
     )
     
     func main() {
         // Load oracle keypair from env
         oracleSecret := os.Getenv("ORACLE_SECRET_KEY")
         if oracleSecret == "" {
             log.Fatal("ORACLE_SECRET_KEY not set")
         }
         
         kp, err := keypair.ParseFull(oracleSecret)
         if err != nil {
             log.Fatalf("Invalid oracle key: %v", err)
         }
         log.Printf("Oracle address: %s", kp.Address())
         
         // HTTP handlers
         http.HandleFunc("/health", healthHandler)
         http.HandleFunc("/api/v1/evidence", evidenceHandler)
         http.Handle("/metrics", promhttp.Handler())
         
         log.Println("Oracle service starting on :8083")
         if err := http.ListenAndServe(":8083", nil); err != nil {
             log.Fatal(err)
         }
     }
     
     func healthHandler(w http.ResponseWriter, r *http.Request) {
         w.WriteHeader(http.StatusOK)
         w.Write([]byte("OK"))
     }
     
     func evidenceHandler(w http.ResponseWriter, r *http.Request) {
         if r.Method != http.MethodPost {
             http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
             return
         }
         
         // Parse form
         if err := r.ParseMultipartForm(10 << 20); err != nil { // 10 MB max
             http.Error(w, "Invalid form", http.StatusBadRequest)
             return
         }
         
         programID, _ := strconv.ParseUint(r.FormValue("program_id"), 10, 64)
         milestoneID, _ := strconv.ParseUint(r.FormValue("milestone_id"), 10, 32)
         
         file, _, err := r.FormFile("file")
         if err != nil {
             http.Error(w, "Missing file", http.StatusBadRequest)
             return
         }
         defer file.Close()
         
         // Compute SHA-256 hash
         hasher := sha256.New()
         if _, err := io.Copy(hasher, file); err != nil {
             http.Error(w, "Hash computation failed", http.StatusInternalServerError)
             return
         }
         hash := hasher.Sum(nil)
         
         log.Printf("Evidence hash for program %d milestone %d: %s", programID, milestoneID, hex.EncodeToString(hash))
         
         // TODO: Upload file to S3
         // s3Client.PutObject(...)
         
         // TODO: Construct Soroban transaction calling Escrow.attest_milestone
         // tx, err := stellar.BuildAttestationTx(programID, uint32(milestoneID), hash, oracleKP)
         // Submit to Horizon
         // AGENT-FLAG: Verify Soroban contract invocation from Go
         
         w.WriteHeader(http.StatusOK)
         w.Write([]byte("Evidence recorded (placeholder)"))
     }
     ```

3. **Crypto helpers:**
   - `services/internal/crypto/envelope.go`:
     ```go
     package crypto
     
     import (
         "crypto/aes"
         "crypto/cipher"
         "crypto/rand"
         "encoding/json"
         "fmt"
         "io"
     )
     
     // EncryptPII encrypts PII data using AES-256-GCM envelope encryption
     // Returns encrypted ciphertext and nonce
     // TODO: Integrate AWS KMS or GCP KMS for data encryption key (DEK) management
     func EncryptPII(data map[string]string) ([]byte, error) {
         // Marshal to JSON
         plaintext, err := json.Marshal(data)
         if err != nil {
             return nil, err
         }
         
         // TODO: Fetch DEK from KMS, encrypt with KEK
         // For now, use a placeholder key (NOT PRODUCTION SAFE)
         key := make([]byte, 32) // 256-bit key
         if _, err := io.ReadFull(rand.Reader, key); err != nil {
             return nil, err
         }
         
         block, err := aes.NewCipher(key)
         if err != nil {
             return nil, err
         }
         
         gcm, err := cipher.NewGCM(block)
         if err != nil {
             return nil, err
         }
         
         nonce := make([]byte, gcm.NonceSize())
         if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
             return nil, err
         }
         
         ciphertext := gcm.Seal(nonce, nonce, plaintext, nil)
         return ciphertext, nil
     }
     
     // DecryptPII decrypts PII data
     // TODO: Implement with KMS DEK retrieval
     func DecryptPII(ciphertext []byte) (map[string]string, error) {
         return nil, fmt.Errorf("not implemented - integrate KMS")
     }
     ```

4. **Stellar transaction helpers:**
   - `services/internal/stellar/transaction.go`:
     ```go
     package stellar
     
     import (
         "fmt"
         
         "github.com/stellar/go-stellar-sdk/keypair"
         "github.com/stellar/go-stellar-sdk/txnbuild"
         // AGENT-FLAG: Verify txnbuild import path in new SDK
     )
     
     // BuildAttestationTx constructs a Soroban transaction calling Escrow.attest_milestone
     // AGENT-FLAG: Verify Soroban contract invocation from Go - InvokeHostFunction operation?
     func BuildAttestationTx(programID, milestoneID uint64, hash [32]byte, oracleKey *keypair.Full) (*txnbuild.Transaction, error) {
         // TODO: Verify Soroban contract invocation pattern in go-stellar-sdk
         // May need txnbuild.InvokeHostFunction with contract address and function name
         // Or newer helpers like txnbuild.InvokeContractFunction
         return nil, fmt.Errorf("not implemented - verify Soroban invocation API in go-stellar-sdk")
     }
     ```

5. **Migrations:**
   - `services/migrations/002_beneficiaries.up.sql`:
     ```sql
     CREATE TABLE beneficiaries (
         id SERIAL PRIMARY KEY,
         wallet_address TEXT UNIQUE NOT NULL,
         encrypted_pii JSONB NOT NULL,
         created_at TIMESTAMPTZ DEFAULT NOW()
     );
     
     CREATE INDEX idx_beneficiaries_wallet ON beneficiaries(wallet_address);
     ```
   
   - `services/migrations/002_beneficiaries.down.sql`:
     ```sql
     DROP TABLE IF EXISTS beneficiaries;
     ```

6. **Unit tests:**
   - `services/internal/crypto/envelope_test.go`:
     ```go
     package crypto
     
     import (
         "testing"
     )
     
     func TestEncryptPII(t *testing.T) {
         data := map[string]string{"name": "Alice", "phone": "+1234567890"}
         ciphertext, err := EncryptPII(data)
         if err != nil {
             t.Fatalf("Encryption failed: %v", err)
         }
         if len(ciphertext) == 0 {
             t.Fatal("Ciphertext is empty")
         }
         // TODO: Test decryption round-trip once KMS integrated
     }
     ```

7. **Update build script:**
   - Modify `services/scripts/build.sh`:
     ```bash
     go build -o bin/indexer ./cmd/indexer
     go build -o bin/relayer ./cmd/relayer
     go build -o bin/onboarding ./cmd/onboarding
     go build -o bin/oracle ./cmd/oracle
     ```

**Technical Context:**
- **Stellar SDK txnbuild:** `import "github.com/stellar/go-stellar-sdk/txnbuild"`, `import "github.com/stellar/go-stellar-sdk/keypair"`
- **Soroban contract invocation from Go:** May use `txnbuild.InvokeHostFunction` operation with contract address and function name (FLAG exact API if uncertain—may need `xdr.HostFunction` construction or newer helpers)
- **Keypair:** Oracle service loads keypair from env var `ORACLE_SECRET_KEY` (Stellar S-format secret key)
- **CSV parsing:** `encoding/csv` standard library, `reader.ReadAll()` returns `[][]string`
- **Envelope encryption:** AES-256-GCM with random nonce, KEK-ID references KMS key (AWS KMS or GCP KMS). For scaffold, use placeholder key with TODO comment.
- **SHA-256 hash:** `crypto/sha256` → `[32]byte`
- **S3 upload:** `github.com/aws/aws-sdk-go-v2/service/s3`, `s3.PutObjectInput` (stub for now)

**Tests:**
- `cd services && go build ./...` compiles onboarding + oracle
- `go test ./...` runs tests (crypto round-trip, CSV parse if added)
- `./scripts/build.sh` produces 4 binaries (indexer, relayer, onboarding, oracle)
- `go mod verify` passes

**Verification Commands:**
```bash
cd services
go build ./...
go test ./...
./scripts/build.sh
file bin/onboarding bin/oracle
```

**Demo:**
`curl -X POST http://localhost:8082/api/v1/beneficiaries/import -F "file=@test.csv"` (if server running) returns success or validation error. `ls services/bin/` shows 4 binaries. `grep "github.com/stellar/go-stellar-sdk" services/**/*.go | wc -l` confirms new import path usage throughout. `grep "AGENT-FLAG" services/**/*.go` lists flagged APIs (InvokeHostFunction, keypair methods, Soroban invocation pattern).

---

### Task 8: Clients - Donor Dashboard, Beneficiary PWA, Merchant App

**Objective:** Scaffold three client applications using official tooling (Vite for web, Expo for mobile). Integrate with `@aidflow/contract-sdk`. Verify with `npm run build` for each.

**Implementation:**

1. **Donor Dashboard (Vite + React + TypeScript):**
   
   - Scaffold with official tool:
     ```bash
     cd clients
     npm create vite@latest donor-dashboard -- --template react-ts
     cd donor-dashboard
     npm install
     ```
   
   - Install dependencies:
     ```bash
     npm install @tanstack/react-query @stellar/freighter-api recharts
     npm install @aidflow/contract-sdk@file:../../contracts/sdk
     ```
     - `@tanstack/react-query` for data fetching
     - `@stellar/freighter-api` for Freighter wallet integration (FLAG if package name wrong)
     - `recharts` for charts/visualization
     - `@aidflow/contract-sdk` via local workspace link
   
   - Create `src/App.tsx`:
     ```typescript
     import { useState } from 'react';
     import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
     
     const queryClient = new QueryClient();
     
     function App() {
       const [publicKey, setPublicKey] = useState<string | null>(null);
       
       const connectWallet = async () => {
         // AGENT-FLAG: Verify Freighter API - check @stellar/freighter-api or window.freighter
         try {
           // Placeholder - verify actual API
           // const { publicKey } = await window.freighter.getPublicKey();
           // setPublicKey(publicKey);
           console.log('Wallet connection placeholder');
           setPublicKey('GTEST...');
         } catch (err) {
           console.error('Wallet connection failed', err);
         }
       };
       
       return (
         <QueryClientProvider client={queryClient}>
           <div className="app">
             <header>
               <h1>AidFlow Donor Dashboard</h1>
               {!publicKey ? (
                 <button onClick={connectWallet}>Connect Wallet</button>
               ) : (
                 <div>Connected: {publicKey.slice(0, 8)}...</div>
               )}
             </header>
             
             {publicKey && (
               <main>
                 <section>
                   <h2>Create Program</h2>
                   <form>
                     <input placeholder="Token Address" />
                     <textarea placeholder="Milestones JSON" />
                     <button type="submit">Create</button>
                   </form>
                 </section>
                 
                 <section>
                   <h2>My Programs</h2>
                   <p>Program list placeholder (query from indexer)</p>
                 </section>
                 
                 <section>
                   <h2>Audit Timeline</h2>
                   <p>Timeline chart placeholder (Recharts)</p>
                 </section>
               </main>
             )}
           </div>
         </QueryClientProvider>
       );
     }
     
     export default App;
     ```
   
   - Create `vite.config.ts` with API proxy:
     ```typescript
     import { defineConfig } from 'vite';
     import react from '@vitejs/plugin-react';
     
     export default defineConfig({
       plugins: [react()],
       server: {
         proxy: {
           '/api': {
             target: 'http://localhost:8080',
             changeOrigin: true,
           },
         },
       },
     });
     ```
   
   - Update `package.json` to reference repo:
     ```json
     {
       "name": "aidflow-donor-dashboard",
       "repository": {
         "type": "git",
         "url": "https://github.com/stellar-impact/aidflow.git",
         "directory": "clients/donor-dashboard"
       }
     }
     ```

2. **Beneficiary PWA (Vite + React + TypeScript + Workbox):**
   
   - Scaffold:
     ```bash
     cd clients
     npm create vite@latest beneficiary-pwa -- --template react-ts
     cd beneficiary-pwa
     npm install
     ```
   
   - Install dependencies:
     ```bash
     npm install vite-plugin-pwa workbox-precaching @github/webauthn-json qrcode.react
     npm install @aidflow/contract-sdk@file:../../contracts/sdk
     ```
     - `vite-plugin-pwa` for service worker generation
     - `@github/webauthn-json` for passkey authentication
     - `qrcode.react` for QR code generation
   
   - Create `src/App.tsx`:
     ```typescript
     import { useState } from 'react';
     import QRCode from 'qrcode.react';
     
     function App() {
       const [walletAddress, setWalletAddress] = useState<string | null>(null);
       const [balance, setBalance] = useState<bigint>(0n);
       
       const registerPasskey = async () => {
         // AGENT-FLAG: Verify @github/webauthn-json API
         try {
           // Placeholder - verify actual API
           // const credential = await create({ ... });
           console.log('Passkey registration placeholder');
           setWalletAddress('GBEN...');
         } catch (err) {
           console.error('Passkey registration failed', err);
         }
       };
       
       return (
         <div className="app">
           <header>
             <h1>AidFlow Beneficiary</h1>
           </header>
           
           {!walletAddress ? (
             <div>
               <h2>Welcome</h2>
               <button onClick={registerPasskey}>Register with Passkey</button>
             </div>
           ) : (
             <main>
               <section>
                 <h2>Your Balance</h2>
                 <p>{balance.toString()} tokens</p>
               </section>
               
               <section>
                 <h2>Claim Voucher QR</h2>
                 <QRCode value={`aidflow://voucher/${walletAddress}`} />
               </section>
               
               <section>
                 <h2>Transaction History</h2>
                 <p>History placeholder</p>
               </section>
             </main>
           )}
         </div>
       );
     }
     
     export default App;
     ```
   
   - Add PWA plugin to `vite.config.ts`:
     ```typescript
     import { defineConfig } from 'vite';
     import react from '@vitejs/plugin-react';
     import { VitePWA } from 'vite-plugin-pwa';
     
     export default defineConfig({
       plugins: [
         react(),
         VitePWA({
           registerType: 'autoUpdate',
           includeAssets: ['favicon.ico'],
           manifest: {
             name: 'AidFlow Beneficiary',
             short_name: 'AidFlow',
             description: 'Transparent aid voucher platform',
             theme_color: '#ffffff',
             icons: [
               {
                 src: '/icon-192.png',
                 sizes: '192x192',
                 type: 'image/png'
               }
             ]
           },
           workbox: {
             runtimeCaching: [
               {
                 urlPattern: /^https:\/\/horizon-testnet\.stellar\.org\/.*/i,
                 handler: 'NetworkFirst',
                 options: {
                   cacheName: 'stellar-api-cache',
                 }
               }
             ]
           }
         }),
       ],
     });
     ```
   
   - Update `package.json` with repo:
     ```json
     {
       "name": "aidflow-beneficiary-pwa",
       "repository": {
         "type": "git",
         "url": "https://github.com/stellar-impact/aidflow.git",
         "directory": "clients/beneficiary-pwa"
       }
     }
     ```


3. **Merchant App (Expo + React Native):**
   
   - Scaffold:
     ```bash
     cd clients
     npx create-expo-app@latest merchant-app --template blank-typescript
     cd merchant-app
     npm install
     ```
   
   - Install dependencies:
     ```bash
     npm install expo-camera react-native-vision-camera @react-native-async-storage/async-storage
     npm install @aidflow/contract-sdk@file:../../contracts/sdk
     ```
     - `expo-camera` or `react-native-vision-camera` for QR scanning
     - `@react-native-async-storage/async-storage` for offline storage
   
   - Create `App.tsx`:
     ```typescript
     import React, { useState, useEffect } from 'react';
     import { StyleSheet, Text, View, Button, Alert } from 'react-native';
     
     export default function App() {
       const [hasPermission, setHasPermission] = useState<boolean | null>(null);
       const [scannedVoucher, setScannedVoucher] = useState<string | null>(null);
       
       useEffect(() => {
         (async () => {
           // AGENT-FLAG: Verify expo-camera permission API
           // const { status } = await Camera.requestCameraPermissionsAsync();
           // setHasPermission(status === 'granted');
           console.log('Camera permission placeholder');
           setHasPermission(true);
         })();
       }, []);
       
       const scanQR = () => {
         // TODO: Open camera view with QR scanner
         // On scan, parse voucher_id from QR data
         // Call MerchantRegistry.redeem via SDK
         console.log('QR scan placeholder');
         setScannedVoucher('voucher_12345');
       };
       
       const redeemVoucher = async () => {
         if (!scannedVoucher) return;
         
         try {
           // Placeholder - call contract SDK
           // await merchantRegistry.redeem(beneficiary, merchant, amount);
           Alert.alert('Success', 'Voucher redeemed!');
           setScannedVoucher(null);
         } catch (err) {
           Alert.alert('Error', 'Redemption failed');
         }
       };
       
       if (hasPermission === null) {
         return <View style={styles.container}><Text>Requesting camera permission...</Text></View>;
       }
       if (hasPermission === false) {
         return <View style={styles.container}><Text>No camera access</Text></View>;
       }
       
       return (
         <View style={styles.container}>
           <Text style={styles.title}>AidFlow Merchant</Text>
           
           {!scannedVoucher ? (
             <Button title="Scan Voucher QR" onPress={scanQR} />
           ) : (
             <>
               <Text>Scanned: {scannedVoucher}</Text>
               <Button title="Redeem" onPress={redeemVoucher} />
               <Button title="Cancel" onPress={() => setScannedVoucher(null)} />
             </>
           )}
           
           <Text style={styles.footer}>Offline sync enabled</Text>
         </View>
       );
     }
     
     const styles = StyleSheet.create({
       container: {
         flex: 1,
         backgroundColor: '#fff',
         alignItems: 'center',
         justifyContent: 'center',
         padding: 20,
       },
       title: {
         fontSize: 24,
         fontWeight: 'bold',
         marginBottom: 20,
       },
       footer: {
         marginTop: 40,
         color: '#888',
       },
     });
     ```
   
   - Update `package.json` with repo:
     ```json
     {
       "name": "aidflow-merchant-app",
       "repository": {
         "type": "git",
         "url": "https://github.com/stellar-impact/aidflow.git",
         "directory": "clients/merchant-app"
       }
     }
     ```

4. **Clients README:**
   - `clients/README.md`:
     ```markdown
     # AidFlow Clients
     
     ## Donor Dashboard (Web)
     
     Vite + React + TypeScript SPA for NGO donors.
     
     **Setup:**
     ```bash
     cd donor-dashboard
     npm install
     npm run dev
     ```
     
     **Build:**
     ```bash
     npm run build
     ```
     
     ## Beneficiary PWA (Web)
     
     Progressive Web App with offline support and passkey authentication.
     
     **Setup:**
     ```bash
     cd beneficiary-pwa
     npm install
     npm run dev
     ```
     
     **Build:**
     ```bash
     npm run build
     ```
     
     ## Merchant App (Mobile)
     
     Expo React Native app for QR scanning and voucher redemption.
     
     **Setup:**
     ```bash
     cd merchant-app
     npm install
     npm start
     ```
     
     **Run:**
     - iOS: `npm run ios`
     - Android: `npm run android`
     
     ## Shared Dependencies
     
     All clients depend on `@aidflow/contract-sdk` for contract interactions.
     ```
     ```

**Technical Context:**
- **Vite:** `npm create vite@latest` scaffolds with latest stable Vite (5.x or 6.x as of 2026)
- **Expo:** `npx create-expo-app@latest` uses current stable Expo SDK
- **Freighter wallet API:** `@stellar/freighter-api` package (FLAG if package name changed). API: `window.freighter.getPublicKey()`, `window.freighter.signTransaction()` (verify exact methods)
- **Workbox:** `vite-plugin-pwa` auto-generates service worker; `workbox-precaching` for offline assets
- **Passkey (WebAuthn):** `@github/webauthn-json` simplifies WebAuthn API. Methods: `create()`, `get()` for registration/authentication
- **QR code generation:** `qrcode.react` component: `<QRCode value="data" />`
- **QR code scanning (Expo):** `expo-camera` (`Camera` component with `barCodeScanned` prop) or `react-native-vision-camera` (more advanced). FLAG if API changed.
- **Offline storage:** `@react-native-async-storage/async-storage` for React Native. Web PWA uses IndexedDB via Workbox.
- **Contract SDK usage:** Import `@aidflow/contract-sdk`, instantiate clients: `new EscrowContract(contractId, server)`, call methods: `await escrow.createProgram(...)`
- **TanStack Query:** `useQuery` for fetching data, `useMutation` for transactions. Create `QueryClient`, wrap app in `QueryClientProvider`.

**Tests:**
- `cd clients/donor-dashboard && npm install && npm run build` succeeds (Vite build)
- `cd clients/beneficiary-pwa && npm install && npm run build` succeeds (includes service worker in dist/)
- `cd clients/merchant-app && npm install` succeeds (Expo may not have `build` script without EAS, that's fine)
- Each app runs in dev mode: `npm run dev` (donor/PWA) or `npm start` (merchant)
- `npm run lint` (if eslint configured) passes or warns non-critically

**Verification Commands:**
```bash
cd clients/donor-dashboard
npm install
npm run build
ls dist/

cd ../beneficiary-pwa
npm install
npm run build
ls dist/ | grep sw.js

cd ../merchant-app
npm install
# Expo doesn't need build for dev
```

**Demo:**
`ls clients/donor-dashboard/dist/` shows built assets (index.html, JS, CSS). `curl http://localhost:5173` (Vite dev server for donor dashboard) returns HTML. `cat clients/beneficiary-pwa/dist/sw.js | head -10` shows Workbox service worker. `cat clients/merchant-app/App.tsx | grep -i camera` confirms camera integration. `grep "AGENT-FLAG" clients/*/src/**/*.{ts,tsx}` lists uncertain APIs (Freighter methods, WebAuthn exact API, Expo camera permissions).

---

### Task 9: MVP Issue Backlog & Docker Compose Infrastructure

**Objective:** Generate ~15-20 MVP-focused GitHub issues (Wave 1-3) with full bodies per §6 format. Create docker-compose.yml for full local stack. Verify stack runs with `docker-compose up`.

**Implementation:**

1. **MVP Issue Generation (~15-20 issues):**
   
   Create `docs/issues/mvp/` directory with full issue bodies. Each issue follows §6 format:
   - Title: `[Complexity] Title`
   - Labels: `Stellar Wave`, complexity (`Trivial (100pt)` | `Medium (150pt)` | `High (200pt)`), area, `P0`/`P1`, optional `good first issue`
   - Sections: Description, Acceptance Criteria, Technical Context, Definition of Done, **Verification block**
   - Verification block MUST include:
     - Tailored commands per area (contracts/services/clients/boundaries)
     - "Using an AI agent?" warning paragraph citing §0.5 pinned versions
   
   **Trivial (100pt) issues (5-6 issues):**
   - `001-architecture-diagram.md` — Create Mermaid diagram in docs/ARCHITECTURE.md
   - `002-escrow-doc-comments.md` — Add rustdoc comments + `@stable` markers to Escrow contract
   - `003-eslint-setup.md` — Configure ESLint + Prettier for all TS clients
   - `004-csv-import-template.md` — Create example CSV template for beneficiary onboarding
   - `005-license-headers.md` — Add Apache-2.0 headers to all source files
   - `006-env-example.md` — Create .env.example files for each service

   **Medium (150pt) issues (8-10 issues):**
   - `007-escrow-create-program.md` — Implement + test Escrow.create_program in detail
   - `008-onboarding-csv-endpoint.md` — Complete onboarding CSV endpoint with DB insert
   - `009-indexer-horizon-sse.md` — Implement Horizon SSE event stream subscription
   - `010-passkey-wallet-pwa.md` — Complete passkey registration flow in beneficiary PWA
   - `011-merchant-qr-scanner.md` — Implement QR scanner with react-native-vision-camera
   - `012-relayer-fee-bump.md` — Implement relayer Redis queue consumer + fee-bump logic
   - `013-voucher-issue-batch.md` — Complete VoucherRegistry.issue_batch with tests
   - `014-donor-funding-ui.md` — Build donor dashboard funding form + contract integration
   - `015-beneficiary-postgres.md` — Complete beneficiaries table migration + encryption
   - `016-milestone-release-test.md` — Integration test: create program → attest → release

   **High (200pt) issues (2-4 issues):**
   - `017-oracle-bridge-e2e.md` — Full oracle flow: evidence upload (S3) → attestation → Horizon submit
   - `018-sep24-anchor-integration.md` — Integrate one testnet anchor for SEP-24 cash-out
   - `019-playwright-e2e-suite.md` — E2E test: donor creates program → beneficiary claims → merchant redeems
   - `020-multisig-admin-module.md` — (Optional) Implement multisig admin contract or stellar native multisig

2. **Issue Template (example for Medium issue):**
   
   `docs/issues/mvp/009-indexer-horizon-sse.md`:
   ```markdown
   # [Medium] Implement Indexer Horizon SSE Event Stream
   
   **Points:** 150
   **Labels:** `Stellar Wave`, `Medium (150pt)`, `services`, `P1`
   
   ## Description
   
   The indexer service needs to subscribe to Horizon's Server-Sent Events (SSE) stream to receive real-time ledger events. Parse contract events (program_created, funded, milestone_attested, released, voucher_issued, voucher_claimed, voucher_expired) and store them in Postgres for the GraphQL API.
   
   ## Acceptance Criteria
   
   - [ ] Indexer connects to Horizon SSE endpoint on startup
   - [ ] Parses Soroban contract events from transaction operations
   - [ ] Stores parsed events in Postgres (programs, milestones, vouchers tables)
   - [ ] Handles reconnection on stream interruption
   - [ ] Emits Prometheus metrics (events_processed_total, stream_reconnects_total)
   
   ## Technical Context
   
   - File: `services/cmd/indexer/main.go`, `services/internal/indexer/stream.go`
   - Horizon client: `github.com/stellar/go-stellar-sdk/clients/horizonclient`
   - SSE streaming API: `client.StreamTransactions(ctx, cursor, handler)` or similar (FLAG if method changed)
   - Contract events: Extract from transaction `Memo` or operation `InvokeHostFunction` result
   - Cursor persistence: Store last processed ledger in Postgres to resume after restart
   - Docs: https://developers.stellar.org/docs/data/horizon/api-reference/resources/stream-transactions
   
   ## Definition of Done
   
   - [ ] Tests pass: `go test ./internal/indexer/...`
   - [ ] Code reviewed and approved
   - [ ] Docs updated: add indexer setup instructions to README
   
   ## Verification (REQUIRED before PR — CI will reject failures)
   
   Run locally before opening a PR:
   - [ ] `cd services && go build ./cmd/indexer`
   - [ ] `go test ./internal/indexer/...`
   - [ ] `./bin/indexer` starts and connects to Horizon testnet (check logs)
   - [ ] Prometheus `/metrics` endpoint returns `aidflow_events_processed_total`
   
   > **Using an AI agent?** Paste this issue's Technical Context + these commands into it. Use the PINNED versions (§0.5 of docs/AIDFLOW_MASTER_PROMPT.md): `stellar-cli` 27.x, `soroban-sdk` 26.1, `github.com/stellar/go-stellar-sdk` (new import path, NOT deprecated `stellar/go`). Do NOT submit code that fails these checks locally.
   ```

3. **Issue Template (example for High issue):**
   
   `docs/issues/mvp/017-oracle-bridge-e2e.md`:
   ```markdown
   # [High] Oracle Bridge End-to-End Flow
   
   **Points:** 200
   **Labels:** `Stellar Wave`, `High (200pt)`, `services`, `contracts`, `P0`
   
   ## Description
   
   Complete the oracle service to handle the full evidence attestation flow: accept evidence file upload, store in S3, compute SHA-256 hash, construct Soroban transaction calling `Escrow.attest_milestone`, sign with oracle keypair, submit to Horizon, and return transaction hash.
   
   ## Acceptance Criteria
   
   - [ ] POST `/api/v1/evidence` endpoint accepts multipart form (program_id, milestone_id, file)
   - [ ] Uploads file to S3 bucket with key `evidence/{program_id}/{milestone_id}/{timestamp}`
   - [ ] Computes SHA-256 hash of file content
   - [ ] Constructs Soroban transaction using `txnbuild.InvokeHostFunction` (or equivalent)
   - [ ] Signs transaction with oracle keypair from env `ORACLE_SECRET_KEY`
   - [ ] Submits to Horizon testnet and returns transaction hash
   - [ ] Handles errors gracefully (invalid program, Horizon timeout, S3 failure)
   
   ## Technical Context
   
   - Files: `services/cmd/oracle/main.go`, `services/internal/stellar/transaction.go`
   - Soroban invocation: `txnbuild.InvokeHostFunction` with contract address + function name `attest_milestone` + args (program_id, milestone_id, hash)
   - Horizon submit: `client.SubmitTransaction(tx)` from `horizonclient.Client`
   - S3 SDK: `github.com/aws/aws-sdk-go-v2/service/s3`, use `PutObject`
   - Escrow contract address: Load from env `ESCROW_CONTRACT_ID`
   - Network passphrase: `networks.TestNetworkPassphrase`
   - Docs: https://developers.stellar.org/docs/smart-contracts/guides/transactions
   
   ## Definition of Done
   
   - [ ] Integration test: upload evidence → verify transaction on Horizon testnet
   - [ ] Code reviewed
   - [ ] Oracle README updated with API docs
   
   ## Verification (REQUIRED before PR — CI will reject failures)
   
   Run locally before opening a PR:
   - [ ] `cd services && go build ./cmd/oracle`
   - [ ] `go test ./internal/stellar/...`
   - [ ] Start oracle: `ORACLE_SECRET_KEY=S... ESCROW_CONTRACT_ID=C... ./bin/oracle`
   - [ ] `curl -X POST http://localhost:8083/api/v1/evidence -F program_id=1 -F milestone_id=1 -F file=@test.pdf`
   - [ ] Check Horizon testnet for transaction hash returned in response
   
   > **Using an AI agent?** Paste this issue's Technical Context + these commands into it. Use the PINNED versions (§0.5 of docs/AIDFLOW_MASTER_PROMPT.md): `github.com/stellar/go-stellar-sdk` (new import path), `soroban-sdk` 26.1. **Common mistakes:** `github.com/stellar/go` → should be `github.com/stellar/go-stellar-sdk`; verify `InvokeHostFunction` API exists in new SDK. Do NOT submit code that fails these checks locally.
   ```

4. **Generate all MVP issues:**
   - Create 15-20 issue files in `docs/issues/mvp/` following the format above
   - Each must include tailored Verification block with area-specific commands
   - All must include "Using an AI agent?" warning paragraph
   - Distribution: ~5-6 Trivial, ~8-10 Medium, ~2-4 High
   - Focus: Core contracts (Escrow, Voucher, Merchant), onboarding/oracle services, indexer basics, donor dashboard funding, one SEP-24 anchor integration, one E2E test


5. **Docker Compose Infrastructure:**
   
   `infra/docker-compose.yml`:
   ```yaml
   version: '3.8'
   
   services:
     # Stellar quickstart (testnet with Soroban support)
     stellar:
       image: stellar/quickstart:testing
       command: --testnet --enable-soroban-rpc
       ports:
         - "8000:8000"   # Horizon
         - "8001:8001"   # Soroban RPC
       environment:
         - ENABLE_SOROBAN_RPC=true
     
     # Postgres
     postgres:
       image: postgres:16
       environment:
         POSTGRES_USER: aidflow
         POSTGRES_PASSWORD: password
         POSTGRES_DB: aidflow
       ports:
         - "5432:5432"
       volumes:
         - postgres_data:/var/lib/postgresql/data
     
     # Redis
     redis:
       image: redis:7-alpine
       ports:
         - "6379:6379"
     
     # S3 (MinIO)
     minio:
       image: minio/minio:latest
       command: server /data --console-address ":9001"
       environment:
         MINIO_ROOT_USER: minioadmin
         MINIO_ROOT_PASSWORD: minioadmin
       ports:
         - "9000:9000"
         - "9001:9001"
       volumes:
         - minio_data:/data
     
     # Indexer service
     indexer:
       build:
         context: ../services
         dockerfile: cmd/indexer/Dockerfile
       environment:
         HORIZON_URL: http://stellar:8000
         DATABASE_URL: postgres://aidflow:password@postgres:5432/aidflow?sslmode=disable
       ports:
         - "8080:8080"
       depends_on:
         - stellar
         - postgres
     
     # Relayer service
     relayer:
       build:
         context: ../services
         dockerfile: cmd/relayer/Dockerfile
       environment:
         REDIS_URL: redis:6379
         HORIZON_URL: http://stellar:8000
       ports:
         - "8081:8081"
       depends_on:
         - stellar
         - redis
     
     # Onboarding service
     onboarding:
       build:
         context: ../services
         dockerfile: cmd/onboarding/Dockerfile
       environment:
         DATABASE_URL: postgres://aidflow:password@postgres:5432/aidflow?sslmode=disable
       ports:
         - "8082:8082"
       depends_on:
         - postgres
     
     # Oracle service
     oracle:
       build:
         context: ../services
         dockerfile: cmd/oracle/Dockerfile
       environment:
         HORIZON_URL: http://stellar:8000
         S3_ENDPOINT: http://minio:9000
         AWS_ACCESS_KEY_ID: minioadmin
         AWS_SECRET_ACCESS_KEY: minioadmin
         ORACLE_SECRET_KEY: ${ORACLE_SECRET_KEY}
         ESCROW_CONTRACT_ID: ${ESCROW_CONTRACT_ID}
       ports:
         - "8083:8083"
       depends_on:
         - stellar
         - minio
     
     # Donor dashboard (nginx serving static build)
     donor-dashboard:
       image: nginx:alpine
       volumes:
         - ../clients/donor-dashboard/dist:/usr/share/nginx/html:ro
       ports:
         - "3000:80"
     
     # Beneficiary PWA (nginx serving static build)
     beneficiary-pwa:
       image: nginx:alpine
       volumes:
         - ../clients/beneficiary-pwa/dist:/usr/share/nginx/html:ro
       ports:
         - "3001:80"
   
   volumes:
     postgres_data:
     minio_data:
   ```
   
   Create Dockerfiles for each service (placeholder):
   - `services/cmd/indexer/Dockerfile`:
     ```dockerfile
     FROM golang:1.23-alpine AS builder
     WORKDIR /app
     COPY go.mod go.sum ./
     RUN go mod download
     COPY . .
     RUN go build -o indexer ./cmd/indexer
     
     FROM alpine:latest
     RUN apk --no-cache add ca-certificates
     COPY --from=builder /app/indexer /indexer
     EXPOSE 8080
     CMD ["/indexer"]
     ```
   - Similar Dockerfiles for relayer, onboarding, oracle

6. **Kubernetes setup (base + testnet overlay):**
   
   `infra/k8s/base/kustomization.yaml`:
   ```yaml
   apiVersion: kustomize.config.k8s.io/v1beta1
   kind: Kustomization
   
   resources:
     - indexer-deployment.yaml
     - relayer-deployment.yaml
     - onboarding-deployment.yaml
     - oracle-deployment.yaml
     - postgres-statefulset.yaml
     - redis-deployment.yaml
     - services.yaml
   
   commonLabels:
     app: aidflow
   ```
   
   `infra/k8s/overlays/testnet/kustomization.yaml`:
   ```yaml
   apiVersion: kustomize.config.k8s.io/v1beta1
   kind: Kustomization
   
   bases:
     - ../../base
   
   namespace: aidflow-testnet
   
   patchesStrategicMerge:
     - horizon-url-patch.yaml
   
   configMapGenerator:
     - name: env-config
       literals:
         - HORIZON_URL=https://horizon-testnet.stellar.org
         - NETWORK=testnet
   ```
   
   `infra/k8s/overlays/mainnet/README.md`:
   ```markdown
   # Mainnet Overlay
   
   **Status:** Deferred until post-audit (Wave 7-8)
   
   This directory will contain Kustomize patches for mainnet deployment once contracts are audited and testnet deployment is validated.
   
   ## Planned Configuration
   
   - Horizon URL: https://horizon.stellar.org
   - Network: mainnet
   - Sealed secrets for production keypairs
   - Resource limits and autoscaling
   - Production monitoring and alerts
   ```

7. **Setup scripts:**
   
   `scripts/setup-dev.sh`:
   ```bash
   #!/bin/bash
   set -e
   
   echo "Setting up AidFlow development environment..."
   
   # Check prerequisites
   command -v docker >/dev/null 2>&1 || { echo "Docker required"; exit 1; }
   command -v npm >/dev/null 2>&1 || { echo "npm required"; exit 1; }
   command -v go >/dev/null 2>&1 || { echo "Go required"; exit 1; }
   command -v cargo >/dev/null 2>&1 || { echo "Rust/Cargo required"; exit 1; }
   
   # Install stellar-cli
   if ! command -v stellar >/dev/null 2>&1; then
       echo "Installing stellar-cli 27.x..."
       cargo install --locked stellar-cli --version '^27'
   fi
   
   # Build contracts
   echo "Building contracts..."
   cd contracts && ./scripts/build.sh && cd ..
   
   # Install contract SDK
   echo "Installing contract SDK..."
   cd contracts/sdk && npm install && npm run build && cd ../..
   
   # Build services
   echo "Building services..."
   cd services && go mod tidy && ./scripts/build.sh && cd ..
   
   # Install clients
   echo "Installing clients..."
   cd clients/donor-dashboard && npm install && cd ../..
   cd clients/beneficiary-pwa && npm install && cd ../..
   cd clients/merchant-app && npm install && cd ../..
   
   echo "✓ Development environment ready!"
   echo "Run 'docker-compose -f infra/docker-compose.yml up' to start services"
   ```
   
   `scripts/seed-testnet.sh`:
   ```bash
   #!/bin/bash
   set -e
   
   echo "Deploying and seeding AidFlow contracts on testnet..."
   
   # Deploy contracts
   cd contracts
   stellar contract deploy --wasm target/wasm32-unknown-unknown/release/aidflow_config.wasm --network testnet
   stellar contract deploy --wasm target/wasm32-unknown-unknown/release/aidflow_escrow.wasm --network testnet
   # ... etc
   
   # Initialize contracts
   # stellar contract invoke --id CONFIG_ID --fn init -- --admin ADMIN_ADDR --oracle ORACLE_ADDR
   
   echo "✓ Contracts deployed to testnet"
   ```

**Technical Context:**
- **Docker Compose:** Uses official images (stellar/quickstart:testing, postgres:16, redis:7, minio, nginx)
- **Stellar quickstart:** Includes Horizon + Soroban RPC for local development
- **Service Dockerfiles:** Multi-stage builds (Go builder → alpine runtime)
- **Client serving:** Nginx serves static builds from dist/ directories
- **Kubernetes:** Kustomize for base + overlays pattern (testnet fleshed out, mainnet stubbed)
- **Setup script:** Checks prerequisites, builds all layers, provides instructions

**Tests:**
- `docker-compose -f infra/docker-compose.yml config` validates YAML syntax
- `docker-compose -f infra/docker-compose.yml up -d` starts all services
- `curl http://localhost:8080/health` (indexer), `curl http://localhost:8081/health` (relayer), etc. return 200
- `curl http://localhost:8000` (Stellar Horizon) returns JSON
- `docker-compose -f infra/docker-compose.yml logs` shows no critical errors

**Verification Commands:**
```bash
cd infra
docker-compose config
docker-compose up -d
sleep 10
curl http://localhost:8080/health
curl http://localhost:8081/health
curl http://localhost:8000
docker-compose logs --tail=50
docker-compose down
```

**Demo:**
After running `docker-compose up`, services are accessible:
- Horizon: http://localhost:8000
- Indexer: http://localhost:8080/health → 200 OK
- Donor dashboard: http://localhost:3000 → renders React app
- Postgres: `psql postgres://aidflow:password@localhost:5432/aidflow` connects
- MinIO console: http://localhost:9001

MVP issues in `docs/issues/mvp/` (15-20 markdown files) are ready to be imported into GitHub with labels and milestones.

---

## Success Criteria

After completing all 9 tasks:

✅ **Repository structure:** Complete monorepo with contracts/, services/, clients/, infra/, docs/, scripts/, .github/  
✅ **Contracts build:** `cd contracts && stellar contract build` produces 4 wasm files  
✅ **Services build:** `cd services && go build ./...` compiles 4 binaries  
✅ **Clients build:** Each client builds successfully (donor dashboard, PWA with service worker, merchant app)  
✅ **Boundary check passes:** `./scripts/check-boundaries.sh` enforces dependency direction  
✅ **CI workflows:** All workflow YAML files present and syntactically valid  
✅ **Contributor guardrails:** PR template, CONTRIBUTING AI section, CI auto-comment workflow, issue templates with Verification blocks  
✅ **Docker Compose stack:** `docker-compose up` runs full local stack (Stellar, Postgres, Redis, S3, 4 services, 2 web clients)  
✅ **MVP issues:** 15-20 full GitHub issue bodies in docs/issues/mvp/ with proper format, labels, Verification blocks  
✅ **Documentation:** ARCHITECTURE, EXTRACTION_PLAN, CONTRACT_INTERFACES, API, SECURITY stubs in docs/  
✅ **All AGENT-FLAGs documented:** Uncertain APIs flagged for human review (Soroban client patterns, Horizon SSE, Go SDK methods)

**Production-ready indicators:**
- Tests green across modules (contracts, services pass; clients build)
- Issues clear enough for cold pickup (detailed Technical Context, specific file paths, API docs links)
- All issues tagged with complexity points + area + Stellar Wave label
- CI/CD present (contracts-ci, services-ci, clients-ci, boundaries-check, deploy-testnet)
- Observability hooks (Prometheus /metrics in all services)
- Upgrade paths documented (contract upgrade strategy in CONTRACT_INTERFACES.md)
- Extraction plan clear (EXTRACTION_PLAN.md with Phase 1 contracts separation)

---

## Notes for Execution

**For the implementing agent:**
1. Build sequentially: Module 1 → verify → Module 2 → verify → ... → Module 9
2. After each module, report "Module N: ✓" with summary (key files created + verification command output)
3. On build failure (✗), STOP immediately and surface the full error — do NOT proceed or work around
4. Flag uncertain APIs with `// AGENT-FLAG: verify <API>` comments — do NOT guess
5. Use official scaffolders (vite, expo) — do NOT hardcode JS package versions
6. Use `go mod tidy` to resolve Go dependencies
7. Verify commands before reporting ✓:
   - Contracts: `stellar contract build && cargo test --all`
   - Services: `go build ./... && go test ./...`
   - Clients: `npm run build` for each
   - Boundaries: `./scripts/check-boundaries.sh`
   - Docker: `docker-compose config && docker-compose up -d` (smoke test)

**For human review (post-scaffold):**
1. Review all AGENT-FLAG comments (grep for them)
2. Verify Soroban SDK 26.1 API usage in contracts (storage, auth, events)
3. Verify Go SDK new import path throughout services
4. Test docker-compose stack fully (all services healthy, can create test program)
5. Review MVP issues for completeness before GitHub import
6. Run full CI locally if possible (contracts-ci, services-ci, clients-ci)

---

*End of Implementation Plan*
