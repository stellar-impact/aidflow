# AidFlow

**Transparent, conditional aid disbursement on Stellar/Soroban**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban-purple)](https://stellar.org)
[![Drips Network](https://img.shields.io/badge/Drips-Stellar%20Wave-green)](https://drips.network)

---

## Overview

AidFlow is a transparent, auditable platform for conditional aid disbursement built on Stellar/Soroban smart contracts. NGOs fund milestone-gated escrows, beneficiaries claim stablecoin vouchers, and merchants redeem to local fiat — with every movement publicly verifiable.

### Key Features

- **Milestone-Gated Escrows** — Funds released only when field evidence is verified
- **Two-Key Release** — Oracle attests, admin multisig releases (no single point of failure)
- **Voucher System** — Batch issuance, lazy pull-based claims, expiry handling
- **Merchant Integration** — Allowlist registry with SEP-24 anchor cash-out
- **Privacy-First** — On-chain: asset movements only. Off-chain: encrypted PII (GDPR-friendly)
- **Gas Efficient** — Batch operations, fee sponsorship for beneficiaries

### Architecture

```
Clients (Vite+React, Expo)
    ↓
@aidflow/contract-sdk (TypeScript)
    ↓
Services (Go: indexer, oracle, onboarding, relayer)
    ↓
Contracts (Rust: Config, Escrow, VoucherRegistry, MerchantRegistry)
    ↓
Stellar Network (Testnet → Mainnet)
```

---

## Quick Start

### Prerequisites

- **Node.js** 20+
- **Go** 1.23+
- **Rust** 1.84+ with `wasm32v1-none` target
- **stellar-cli** 27.x (`cargo install --locked stellar-cli --version '^27'`)
- **Docker** & **Docker Compose** (for local stack)

### Installation

```bash
git clone https://github.com/stellar-impact/aidflow.git
cd aidflow
make install
```

### Local Development

Start the full stack (Stellar quickstart, Postgres, Redis, S3, services):

```bash
docker-compose -f infra/docker-compose.yml up
```

Or run components individually:

```bash
# Build contracts
cd contracts && ./scripts/build.sh

# Build services
cd services && ./scripts/build.sh

# Build clients
cd clients/donor-dashboard && npm run dev
```

### Running Tests

```bash
# All tests
make test

# Contracts only
cd contracts && cargo test --all

# Services only
cd services && go test ./...

# Clients only
npm test --workspaces
```

---

## Repository Structure

```
aidflow/
├── contracts/          # Soroban smart contracts (Rust)
│   ├── types/         # Shared types (@stable)
│   ├── config/        # Admin multisig + oracle
│   ├── escrow/        # Programs, milestones, funding, release
│   ├── voucher_registry/  # Batch issuance, claims, expiry
│   ├── merchant_registry/ # Allowlist, redemption
│   └── sdk/           # @aidflow/contract-sdk (TypeScript)
├── services/          # Backend services (Go)
│   ├── cmd/
│   │   ├── indexer/   # Horizon event stream → Postgres
│   │   ├── relayer/   # Fee-bump sponsorship
│   │   ├── onboarding/ # Beneficiary CSV import + PII encryption
│   │   └── oracle/    # Evidence upload → attestation
│   └── internal/
├── clients/           # Web & mobile apps
│   ├── donor-dashboard/   # Vite + React + TypeScript
│   ├── beneficiary-pwa/   # PWA with offline support + passkey
│   └── merchant-app/      # Expo React Native (QR scanner)
├── infra/             # Infrastructure as Code
│   ├── docker-compose.yml  # Full local stack
│   └── k8s/               # Kubernetes + Kustomize
├── docs/              # Documentation
│   ├── ARCHITECTURE.md
│   ├── EXTRACTION_PLAN.md
│   ├── CONTRACT_INTERFACES.md
│   └── adr/           # Architecture Decision Records
└── scripts/           # Development scripts
```

---

## Documentation

- [**Architecture**](docs/ARCHITECTURE.md) — System design and data flow
- [**Contract Interfaces**](docs/CONTRACT_INTERFACES.md) — Soroban API reference
- [**API Documentation**](docs/API.md) — Services API specs
- [**Extraction Plan**](docs/EXTRACTION_PLAN.md) — Modular monorepo strategy
- [**Security Policy**](docs/SECURITY.md) — Audit checklist and vulnerability reporting
- [**ADRs**](docs/adr/) — Architecture Decision Records

---

## Contributing

We welcome contributions from the community! Please read our [**Contributing Guide**](CONTRIBUTING.md) before submitting PRs.

**Using AI agents to complete issues?** See the [AI Agent Guidelines](CONTRIBUTING.md#using-ai-agents-to-complete-issues) section for critical version pinning and verification requirements.

### Development Workflow

1. **Fork** the repository
2. **Create a branch** from `main`: `git checkout -b feature/your-feature`
3. **Make your changes** following the style guides
4. **Run tests locally**: `make test && ./scripts/check-boundaries.sh`
5. **Commit** with descriptive messages
6. **Push** and open a **Pull Request**

---

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| **Smart Contracts** | Rust + soroban-sdk | 26.1+ |
| **Build Tool** | stellar-cli | 27.x |
| **Backend Services** | Go | 1.23+ |
| **Stellar SDK (Go)** | github.com/stellar/go-stellar-sdk | Latest |
| **Frontend (Web)** | Vite + React + TypeScript | Latest |
| **Mobile** | Expo React Native | Latest |
| **Infrastructure** | Docker Compose, Kubernetes | — |

> **Note:** See [docs/AIDFLOW_MASTER_PROMPT.md](AIDFLOW_MASTER_PROMPT.md) §0.5 for exact pinned versions and anti-drift guards.

---

## Roadmap

### MVP (Wave 1-3) ✅ In Progress
- Core contracts (Config, Escrow, VoucherRegistry, MerchantRegistry)
- Indexer + Oracle + Onboarding services
- Donor dashboard + Beneficiary PWA
- Local dev stack (docker-compose)
- Testnet deployment

### Beta (Wave 4-6)
- Merchant app (QR scanner)
- SEP-24 anchor integration (cash-out)
- GraphQL API + subscriptions
- E2E test suite (Playwright)
- Prometheus metrics + Grafana dashboards

### Audit Prep (Wave 7-8)
- Security audit (contracts + services)
- Contract extraction to `stellar-impact/aidflow-contracts`
- Mainnet deployment preparation
- Load testing + chaos engineering

---

## License

Copyright 2026 Stellar Impact

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

---

## Links

- **Repository:** https://github.com/stellar-impact/aidflow
- **Drips Network:** https://drips.network
- **Stellar Developers:** https://developers.stellar.org
- **Soroban Docs:** https://soroban.stellar.org

---

**Built with ❤️ for transparent aid distribution**
