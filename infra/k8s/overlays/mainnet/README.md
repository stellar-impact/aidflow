# Mainnet Overlay

**Status:** Deferred until post-audit (Wave 7-8)

This directory will contain Kustomize patches for mainnet deployment once contracts are audited and testnet deployment is validated.

## Planned Configuration

- **Horizon URL:** https://horizon.stellar.org
- **Network:** mainnet
- **Sealed secrets** for production keypairs
- **Resource limits** and autoscaling
- **Production monitoring** and alerts

## Prerequisites

Before mainnet deployment:

- [ ] Professional security audit completed
- [ ] All critical and high vulnerabilities fixed
- [ ] Testnet deployment running for 1+ month
- [ ] Incident response plan documented
- [ ] Admin multisig configured and tested
- [ ] Oracle key rotation procedure documented
- [ ] Monitoring and alerting operational
- [ ] Bug bounty program launched

---

**Do not deploy to mainnet until all prerequisites are met.**

See `docs/SECURITY.md` for full pre-mainnet checklist.
