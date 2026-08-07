# Security Policy

AidFlow custodies funds and handles sensitive beneficiary data, so we treat security
as a first-class concern and welcome responsible disclosure.

## Reporting a Vulnerability

**Please report security vulnerabilities privately by email to
<security@stellarimpact.org>.**

Do **not** open a public GitHub issue for a security vulnerability, and please do not
disclose it publicly before we have had a chance to address it.

When reporting, please include:

- A description of the vulnerability
- Steps to reproduce
- Potential impact
- A suggested fix, if you have one

We aim to acknowledge reports within **48 hours** and to complete an initial assessment
within **one week**. Remediation timelines depend on severity.

## Scope and Status

AidFlow is in active pre-audit development. **Do not use it on mainnet with real funds
until after a professional security audit.** On-chain state is deliberately limited to
asset movements and 32-byte evidence commitments — no personal data is ever stored
on-chain.

## Full Policy

The complete security policy — severity levels, response timelines, audit status,
best practices for contributors and deployers, known pre-audit limitations, and the
coordinated disclosure process — is maintained in
[**docs/SECURITY.md**](docs/SECURITY.md).
