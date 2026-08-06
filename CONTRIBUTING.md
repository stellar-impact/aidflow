# Contributing to AidFlow

Thank you for your interest in contributing to AidFlow! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Using AI Agents to Complete Issues](#using-ai-agents-to-complete-issues)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Pull Request Process](#pull-request-process)
- [Commit Message Guidelines](#commit-message-guidelines)

---

## Code of Conduct

This project adheres to a code of conduct that all contributors are expected to follow. Please be respectful, inclusive, and professional in all interactions.

---

## Getting Started

### Prerequisites

Ensure you have the following installed:

- **Node.js** 20+ and **npm** 10+
- **Go** 1.23+
- **Rust** 1.84+ with `wasm32v1-none` target
- **stellar-cli** 27.x: `cargo install --locked stellar-cli --version '^27'`
- **Docker** and **Docker Compose**

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/aidflow.git
   cd aidflow
   ```
3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/stellar-impact/aidflow.git
   ```

### Install Dependencies

```bash
make install
```

### Run Tests

```bash
make test
```

---

## Development Workflow

1. **Sync with upstream:**
   ```bash
   git checkout main
   git pull upstream main
   ```

2. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```
   
   Branch naming conventions:
   - `feature/` — new features
   - `fix/` — bug fixes
   - `docs/` — documentation updates
   - `refactor/` — code refactoring

3. **Make your changes** following the coding standards below

4. **Run tests and linting:**
   ```bash
   make test
   make lint
   ./scripts/check-boundaries.sh
   ```

5. **Commit your changes** (see [Commit Message Guidelines](#commit-message-guidelines))

6. **Push to your fork:**
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Open a Pull Request** from your fork to `stellar-impact/aidflow:main`

---

## Using AI Agents to Complete Issues

Many contributors use AI agents (like Kiro, Cursor, GitHub Copilot, or custom Claude/GPT setups) to help complete issues. This is perfectly fine, but there are critical requirements to prevent broken PRs from reaching the review queue.

### ⚠️ Critical: Version Pinning

The Stellar/Soroban toolchain had **breaking changes in late 2025 / early 2026**. An AI agent using stale training knowledge will generate broken code. Always verify the agent uses these exact versions:

| Component | Correct Version | Common Mistake |
|-----------|----------------|----------------|
| **CLI** | `stellar-cli` 27.x | `soroban-cli` (old name) |
| **Build Command** | `stellar contract build` | `soroban contract build` |
| **Soroban SDK** | `soroban-sdk` 26.1+ | Using 25.x or older |
| **Rust Target** | `wasm32v1-none` (automatic via stellar-cli) | `cargo build --target wasm32-unknown-unknown` |
| **Go SDK** | `github.com/stellar/go-stellar-sdk` | `github.com/stellar/go` (DEPRECATED) |

### 🚨 Common AI Agent Mistakes

Watch for these in generated code:

1. **Wrong CLI command:**
   - ❌ `soroban contract build`
   - ✅ `stellar contract build`

2. **Wrong Go import path:**
   - ❌ `import "github.com/stellar/go/clients/horizonclient"`
   - ✅ `import "github.com/stellar/go-stellar-sdk/clients/horizonclient"`

3. **Wrong build method for contracts:**
   - ❌ `cargo build --target wasm32-unknown-unknown`
   - ✅ `stellar contract build`

4. **Outdated soroban-sdk APIs:**
   - Check https://docs.rs/soroban-sdk/26.1.0/ for correct API
   - If unsure, add a comment: `// AGENT-FLAG: verify this API exists in soroban-sdk 26.1`

### ✅ Required Before Submitting a PR

**Always run the issue's Verification commands locally and confirm they pass.**

Example verification commands by area:

**Contracts:**
```bash
cd contracts
stellar contract build --package aidflow-escrow
cargo test --package aidflow-escrow
cargo clippy --package aidflow-escrow -- -D warnings
```

**Services:**
```bash
cd services
go build ./cmd/indexer
go test ./...
```

**Clients:**
```bash
cd clients/donor-dashboard
npm run build
npm test
```

**Boundary Check:**
```bash
./scripts/check-boundaries.sh
```

### 📋 AI Agent Workflow Checklist

When using an AI agent to resolve an issue:

1. [ ] Read the issue's **Technical Context** section completely
2. [ ] Paste the **Verification commands** into your agent
3. [ ] Explicitly instruct the agent to use pinned versions (reference `docs/AIDFLOW_MASTER_PROMPT.md §0.5`)
4. [ ] Run verification commands **before** opening a PR
5. [ ] If commands fail, debug locally — **do not submit broken code**
6. [ ] If agent invents an API you can't verify, ask in issue comments — **do not guess**

### 🛑 CI Failures

If your PR fails CI:
- Review the CI logs carefully
- Re-run verification commands locally
- Check that you're using pinned versions
- Fix issues and force-push to your branch

PRs will not be merged until CI passes. Branch protection blocks merge on failing checks; the CI guard bot will comment with guidance on what to fix.

---

## Coding Standards

### Rust (Contracts)

- Follow official Rust style: `cargo fmt`
- Lint with Clippy: `cargo clippy -- -D warnings`
- Add rustdoc comments for public APIs
- Mark stability with `#[doc = "@stable"]` or `#[doc = "@beta"]`
- Use `stellar contract build`, never `cargo build`

### Go (Services)

- Follow official Go style: `go fmt`
- Lint with golangci-lint (if available)
- Use structured logging
- Write table-driven tests
- Import path: `github.com/stellar/go-stellar-sdk` (NOT `github.com/stellar/go`)

### TypeScript (Clients + Contract SDK)

- Use ESLint + Prettier
- Prefer functional components (React)
- Use TypeScript strict mode
- Add JSDoc comments for public APIs
- Mark stability: `/** @stable */` or `/** @beta */`

### General

- Keep functions small and focused
- Write descriptive variable names
- Add comments for complex logic
- Update documentation when changing behavior

---

## Testing Requirements

### Contracts

- Unit tests for each function
- Integration tests for multi-contract flows
- Minimum 80% coverage for core logic

### Services

- Unit tests for business logic
- Integration tests for database operations
- HTTP handler tests
- Minimum 70% coverage

### Clients

- Component tests (React Testing Library / Vitest)
- E2E tests for critical flows (Playwright)
- Snapshot tests for UI components

---

## Pull Request Process

1. **Fill out the PR template completely**
   - Link the related issue
   - Check all verification boxes
   - Describe your testing approach

2. **Ensure CI passes:**
   - contracts-ci: fmt, clippy, test, build
   - services-ci: fmt, lint, test, build
   - clients-ci: lint, test, build
   - boundaries-check: dependency direction

3. **Request review** from maintainers

4. **Address feedback promptly**
   - Make requested changes
   - Respond to comments
   - Re-request review after changes

5. **Squash commits** if requested before merge

---

## Commit Message Guidelines

Use conventional commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(escrow): implement milestone attestation

Add attest_milestone function that stores evidence hash
and emits attestation event. Requires oracle authentication.

Closes #42
```

```
fix(indexer): handle reconnection on Horizon SSE disconnect

Add exponential backoff retry logic when SSE stream drops.
Store last processed ledger to resume from correct position.

Fixes #103
```

---

## Questions?

- Open a discussion on GitHub
- Ask in issue comments
- Tag maintainers: @stellar-impact

---

Thank you for contributing to AidFlow! 🎉
