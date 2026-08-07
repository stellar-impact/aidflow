# ADR 0001: Use Soroban for Escrow

**Date:** 2026-08-06  
**Status:** Accepted  
**Deciders:** Architecture team

---

## Context

AidFlow requires smart contracts for milestone-gated escrow, voucher management, and merchant redemption. We evaluated multiple blockchain platforms for implementing these contracts.

### Requirements

1. **Low Transaction Costs:** Beneficiaries should not pay gas fees
2. **Fast Finality:** <5 seconds for voucher claims
3. **Asset Support:** Native stablecoin integration
4. **Developer Experience:** Mature tooling and SDK
5. **Auditable:** Public verification of all aid movements
6. **Upgradability:** Contract upgrades without data migration

---

## Decision

We will use **Stellar Soroban smart contracts** for all escrow, voucher, and merchant logic.

---

## Rationale

### Why Soroban?

**1. Native Stellar Asset Integration**
- Soroban contracts can directly interact with Stellar assets (USDC, other stablecoins)
- No bridging or wrapping required
- Lower friction for NGOs already using Stellar ecosystem

**2. Low Transaction Costs**
- Average tx cost: ~$0.001 (vs $1-50 on Ethereum)
- Fee sponsorship built-in (relayer can pay on behalf of beneficiaries)
- Predictable costs (no gas price auctions)

**3. Fast Finality**
- 5-second block time
- Immediate confirmation for beneficiaries
- Good UX for offline-first PWA

**4. Rust + WebAssembly**
- Memory-safe language (fewer vulnerabilities)
- Familiar tooling (Cargo, Clippy)
- WASM target (portable, auditable bytecode)

**5. Stellar Ecosystem**
- Existing anchor network for fiat on/off-ramps (SEP-24)
- Identity framework (SEP-10/12) for merchant KYC
- Active developer community

**6. Contract Upgradability**
- `update_current_contract_wasm` operation
- Can fix bugs without redeploying from scratch
- Critical for pre-audit MVP iteration

### Alternatives Considered

**Ethereum + Solidity**
- ❌ High gas costs ($10-100 per tx)
- ❌ Fee sponsorship complex (GSN, relayers)
- ✅ Mature audit ecosystem
- ✅ More developer mindshare
- **Rejected:** Gas costs prohibitive for aid distribution

**Polygon/L2s**
- ✅ Lower gas than Ethereum ($0.01-1)
- ❌ Still requires bridging for stablecoins
- ❌ Finality slower (15-30s)
- **Rejected:** Cost still too high, bridging friction

**Solana + Anchor**
- ✅ Very low costs ($0.00001)
- ✅ Fast finality (400ms)
- ❌ Less mature stablecoin ecosystem
- ❌ More complex programming model
- ❌ Higher node requirements (centralization risk)
- **Rejected:** Stellar's stablecoin integration better

**Algorand**
- ✅ Low costs
- ✅ Fast finality
- ❌ Smaller ecosystem
- ❌ Less anchor integration
- **Rejected:** Stellar more established for aid use case

---

## Consequences

### Positive

- **Cost-Effective:** Enables zero-fee experience for beneficiaries
- **Fast UX:** <5s claim time for good mobile experience
- **Stablecoin Native:** Easy USDC integration
- **Audit-Ready:** Rust + formal verification tools available
- **Stellar Anchors:** Ready SEP-24 integration for cash-out

### Negative

- **Smaller Ecosystem:** Fewer developers than Ethereum
- **Newer Platform:** Soroban launched 2023 (less battle-tested)
- **Audit Firms:** Fewer Soroban-specialized auditors
- **Tooling Maturity:** Some rough edges (improving rapidly)

### Mitigations

- **Audit:** Hire auditor with Rust+WASM experience (pre-mainnet)
- **Testnet First:** Extensive testing before mainnet (1+ month)
- **Circuit Breaker:** Admin pause function for emergencies
- **Gradual Rollout:** Start with small pilot programs

---

## Validation

This decision will be validated by:

1. **Testnet Deployment:** Successfully deploying MVP to Stellar testnet
2. **Cost Analysis:** Measuring actual transaction costs <$0.01
3. **Performance:** Confirming <5s finality for claims
4. **Developer Feedback:** Team comfortable with Rust/Soroban SDK

If validation fails, we will revisit this decision before mainnet deployment.

---

## References

- [Soroban Documentation](https://soroban.stellar.org)
- [Stellar Anchor Ecosystem](https://stellar.org/anchors)
- [soroban-sdk 26.1](https://docs.rs/soroban-sdk/26.1.0/)
