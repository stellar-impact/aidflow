# AidFlow Architecture

**Status:** Living document  
**Last Updated:** 2026-08-06

---

## Overview

AidFlow is a transparent, conditional aid disbursement platform built on Stellar/Soroban. This document describes the system architecture, component interactions, and design decisions.

## System Architecture

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

## Core Components

### Smart Contracts (Rust + Soroban)

**Config/AccessControl** — Admin multisig, oracle registration, circuit breaker  
**Escrow** — Programs, milestones, funding, attestation, release  
**VoucherRegistry** — Batch issuance, pull-based claims, expiry  
**MerchantRegistry** — Allowlist, redemption authorization

### Backend Services (Go)

**Indexer** — Horizon SSE → Postgres, real-time event processing  
**Oracle** — Evidence upload → S3 → attestation submission  
**Onboarding** — CSV import, PII encryption, wallet provisioning  
**Relayer** — Fee-bump sponsorship for zero-fee beneficiary experience

### Client Applications

**Donor Dashboard** — Program creation, funding, audit timeline  
**Beneficiary PWA** — Passkey auth, offline-first, voucher management  
**Merchant App** — QR scanning, redemption, SEP-24 cash-out

## Data Flow

*(Detailed flows will be added in architecture diagram issue)*

## Privacy & Security

- **On-Chain:** Asset movements only, no PII
- **Off-Chain:** Encrypted PII in Postgres (AES-256-GCM + KMS)
- **Access Control:** Two-key release (oracle attests, admin releases)

## Related Documents

- [Contract Interfaces](CONTRACT_INTERFACES.md)
- [Extraction Plan](EXTRACTION_PLAN.md)
- [ADRs](adr/)
