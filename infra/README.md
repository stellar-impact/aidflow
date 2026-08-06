# AidFlow Infrastructure

This directory contains infrastructure-as-code for deploying AidFlow.

## Local Development

### Docker Compose

Full local stack including Stellar quickstart, Postgres, Redis, S3 (MinIO), and all services.

```bash
docker-compose up
```

**Services:**
- Horizon: http://localhost:8000
- Soroban RPC: http://localhost:8001
- Indexer: http://localhost:8080
- Relayer: http://localhost:8081
- Onboarding: http://localhost:8082
- Oracle: http://localhost:8083
- Postgres: localhost:5432
- Redis: localhost:6379
- MinIO: http://localhost:9000 (console: 9001)

## Kubernetes

### Base Configuration

Common Kubernetes resources (deployments, services, configmaps).

```bash
kubectl apply -k k8s/base
```

### Overlays

**Testnet:**
```bash
kubectl apply -k k8s/overlays/testnet
```

**Mainnet:** (deferred until post-audit)
```bash
kubectl apply -k k8s/overlays/mainnet
```

## Helm Charts

*(Planned for Wave 6)*

Helm charts for easier deployment and configuration management.

```bash
helm install aidflow ./helm/aidflow --namespace aidflow
```

## Monitoring

All services expose Prometheus metrics on `/metrics`.

**Grafana Dashboards:** (Planned for Wave 5)
- Contract events dashboard
- Service health dashboard
- Transaction volume dashboard

## Secrets Management

### Local Development
Use `.env` files (gitignored).

### Production
- Kubernetes: Sealed Secrets
- Helm: External Secrets Operator

**Required Secrets:**
- `DEPLOYER_SECRET_KEY` — Contract deployer keypair
- `ORACLE_SECRET_KEY` — Oracle service keypair
- `ADMIN_MULTISIG_KEYS` — Admin multisig keypairs
- `DATABASE_URL` — Postgres connection string
- `REDIS_URL` — Redis connection string
- `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` — S3 credentials

---

See specific overlay README files for environment-specific instructions.
