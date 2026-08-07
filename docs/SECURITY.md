# Security Policy

**Last Updated:** 2026-08-06

---

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in AidFlow, please report it privately.

### How to Report

**Email:** security@stellarimpact.org (PGP key: TBD)

**Do NOT:**
- Open a public GitHub issue for security vulnerabilities
- Disclose the vulnerability publicly before we've had a chance to address it

**Please Include:**
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment:** Within 48 hours
- **Initial Assessment:** Within 1 week
- **Fix Timeline:** Depends on severity (see below)

---

## Severity Levels

| Severity | Description | Response Time | Examples |
|----------|-------------|---------------|----------|
| **Critical** | Loss of funds, unauthorized admin access | 24-48 hours | Contract re-entrancy, admin key compromise |
| **High** | Data breach, service disruption | 1 week | PII exposure, Oracle compromise |
| **Medium** | Limited impact, workaround available | 2 weeks | DoS on specific endpoint, minor info leak |
| **Low** | Minimal impact | 1 month | Non-sensitive info disclosure |

---

## Security Audit Status

| Component | Status | Auditor | Report | Date |
|-----------|--------|---------|--------|------|
| **Contracts** | Planned | TBD | TBD | Q3 2026 |
| **Services** | Planned | TBD | TBD | Q3 2026 |

**Pre-Audit Warning:** AidFlow is currently in MVP development. DO NOT use on mainnet with real funds until after professional security audit.

---

## Security Best Practices

### For Contributors

1. **Never commit secrets:**
   - Use `.env` files (gitignored)
   - Use GitHub Secrets for CI/CD
   - Rotate credentials if accidentally committed

2. **Follow secure coding patterns:**
   - Parameterized SQL queries (no string interpolation)
   - Input validation on all endpoints
   - Proper error handling (don't leak sensitive info)

3. **Dependency hygiene:**
   - Pin exact versions
   - Review dependencies before adding
   - Run `cargo audit` and `npm audit` regularly

4. **Review smart contract changes carefully:**
   - Access control logic
   - Arithmetic operations (overflow/underflow)
   - External contract calls
   - Storage mutations

### For Deployers

1. **Testnet first, always:**
   - Deploy to testnet
   - Run extensive tests
   - Monitor for 1+ week
   - Only then consider mainnet

2. **Key management:**
   - Use hardware wallets for admin multisig
   - Rotate oracle keys quarterly
   - Store backups securely (encrypted, multi-location)

3. **Monitoring:**
   - Set up alerts for unusual activity
   - Monitor contract events
   - Track service metrics (Prometheus/Grafana)

4. **Incident response plan:**
   - Circuit breaker (pause contracts)
   - Rollback procedure
   - Communication plan

---

## Known Limitations (Pre-Audit)

1. **Contract Upgrade Timelock:** Not yet implemented (planned for Beta)
2. **Oracle Key Rotation:** Manual process (automation planned for Beta)
3. **Rate Limiting:** Basic implementation (needs tuning)
4. **PII Encryption:** Placeholder KMS integration (need full implementation)

---

## Security Features

### Smart Contracts

- **Two-Key Release:** Oracle attests, admin releases (no single point of failure)
- **Circuit Breaker:** Admin can pause all operations
- **Batch Limits:** Max 100 vouchers per transaction (DoS mitigation)
- **Lazy Claims:** Pull-based (no push overhead or griefing)
- **Event Logging:** All critical actions emit events

### Services

- **PII Encryption:** AES-256-GCM envelope encryption with KMS
- **API Rate Limiting:** Per-IP and per-key limits
- **Input Validation:** All endpoints validate input
- **Secure Defaults:** No debug endpoints in production

### Infrastructure

- **mTLS:** Service-to-service authentication
- **Secrets Management:** Kubernetes secrets / GitHub Secrets
- **Network Segmentation:** Services in private subnets
- **Logging:** Structured logs (no PII in logs)

---

## Vulnerability Disclosure Policy

After a vulnerability is fixed:

1. **Coordinated Disclosure:** We notify affected users privately
2. **Public Disclosure:** We publish details 30 days after fix is deployed
3. **Credit:** We credit the reporter (unless they prefer anonymity)
4. **Bug Bounty:** Planned for post-audit

---

## Security Checklist (Pre-Mainnet)

- [ ] Professional security audit completed
- [ ] All critical and high vulnerabilities fixed
- [ ] Testnet deployment running for 1+ month
- [ ] Incident response plan documented
- [ ] Admin multisig configured and tested
- [ ] Oracle key rotation procedure documented
- [ ] Monitoring and alerting operational
- [ ] Bug bounty program launched

---

## Contact

- **Security Team:** security@stellarimpact.org
- **General Inquiries:** hello@stellarimpact.org
- **GitHub Issues:** https://github.com/stellar-impact/aidflow/issues (non-security only)

---

## Related Documents

- [Architecture](ARCHITECTURE.md)
- [Contract Interfaces](CONTRACT_INTERFACES.md)
- [Contributing Guide](../CONTRIBUTING.md)
