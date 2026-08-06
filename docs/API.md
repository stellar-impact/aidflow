# AidFlow Services API Documentation

**Status:** Draft  
**Version:** 0.1.0

---

## Overview

This document specifies the HTTP APIs for AidFlow backend services. All services expose `/health` and `/metrics` endpoints for monitoring.

## Base URLs

| Service | Local | Testnet | Mainnet |
|---------|-------|---------|---------|
| Indexer | `http://localhost:8080` | TBD | TBD |
| Relayer | `http://localhost:8081` | TBD | TBD |
| Onboarding | `http://localhost:8082` | TBD | TBD |
| Oracle | `http://localhost:8083` | TBD | TBD |

---

## Common Endpoints

### Health Check

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "OK"
}
```

**Status Codes:**
- `200 OK` — Service is healthy
- `503 Service Unavailable` — Service is degraded

### Metrics

**Endpoint:** `GET /metrics`

**Response:** Prometheus-formatted metrics

**Example:**
```
# HELP aidflow_events_processed_total Total events processed by indexer
# TYPE aidflow_events_processed_total counter
aidflow_events_processed_total{contract="escrow"} 1234
```

---

## Indexer Service

### Query Program

**Endpoint:** `GET /api/v1/programs/:id`

**Response:**
```json
{
  "id": "1",
  "funder": "GABC...",
  "token": "CXYZ...",
  "status": "Active",
  "funded_amount": "10000000000",
  "released_amount": "0",
  "milestones": [
    {
      "id": 1,
      "description": "Build well",
      "target_amount": "5000000000",
      "attested_at": null
    }
  ]
}
```

### Query Voucher

**Endpoint:** `GET /api/v1/vouchers/:id`

**Response:**
```json
{
  "id": "42",
  "program_id": "1",
  "recipient": "GDEF...",
  "amount": "1000000000",
  "status": "Unclaimed",
  "expiry": 1735689600,
  "claimed_at": null
}
```

### GraphQL Endpoint (Future)

**Endpoint:** `POST /graphql`

*(Schema TBD in future issue)*

---

## Onboarding Service

### Import Beneficiaries

**Endpoint:** `POST /api/v1/beneficiaries/import`

**Content-Type:** `multipart/form-data`

**Parameters:**
- `file` (required) — CSV file with columns: `name`, `phone`, `wallet_address`

**Response:**
```json
{
  "imported": 150,
  "failed": 2,
  "errors": [
    {
      "row": 23,
      "error": "Invalid wallet address"
    }
  ]
}
```

**Status Codes:**
- `200 OK` — Import successful
- `400 Bad Request` — Invalid CSV format
- `500 Internal Server Error` — Server error

**CSV Format:**
```csv
name,phone,wallet_address
John Doe,+1234567890,GABC...
Jane Smith,+0987654321,GDEF...
```

---

## Oracle Service

### Submit Evidence

**Endpoint:** `POST /api/v1/evidence`

**Content-Type:** `multipart/form-data`

**Parameters:**
- `program_id` (required) — Program ID
- `milestone_id` (required) — Milestone ID
- `file` (required) — Evidence file (image, PDF, max 10MB)

**Response:**
```json
{
  "evidence_hash": "abc123...",
  "s3_url": "s3://aidflow-evidence/1/1/2026-08-06-...",
  "transaction_hash": "def456...",
  "status": "attested"
}
```

**Status Codes:**
- `200 OK` — Evidence submitted and attested
- `400 Bad Request` — Invalid parameters or file
- `413 Payload Too Large` — File exceeds 10MB
- `500 Internal Server Error` — Server or Horizon error

---

## Relayer Service

### Submit Fee-Bump Request

**Endpoint:** `POST /api/v1/fee-bump`

**Request Body:**
```json
{
  "transaction_xdr": "AAAAAgAAAA...",
  "source_account": "GABC..."
}
```

**Response:**
```json
{
  "fee_bump_xdr": "AAAAAgAAAA...",
  "transaction_hash": "ghi789...",
  "status": "submitted"
}
```

**Status Codes:**
- `200 OK` — Fee-bump submitted
- `400 Bad Request` — Invalid transaction XDR
- `429 Too Many Requests` — Rate limit exceeded
- `500 Internal Server Error` — Horizon submission failed

---

## Authentication

### Service-to-Service

All internal service APIs use mutual TLS (mTLS) in production.

### Client APIs

- **Onboarding:** Admin API key (header: `X-API-Key`)
- **Oracle:** Oracle API key (header: `X-API-Key`)
- **Indexer:** Public (read-only)
- **Relayer:** Rate-limited by IP

---

## Rate Limits

| Service | Endpoint | Limit |
|---------|----------|-------|
| Relayer | `/api/v1/fee-bump` | 10 req/min per IP |
| Oracle | `/api/v1/evidence` | 100 req/hour per API key |
| Onboarding | `/api/v1/beneficiaries/import` | 10 req/hour per API key |

---

## Error Responses

All services return errors in this format:

```json
{
  "error": {
    "code": "INVALID_PARAMETER",
    "message": "Missing required field: program_id",
    "details": {}
  }
}
```

**Common Error Codes:**
- `INVALID_PARAMETER` — Bad request
- `UNAUTHORIZED` — Missing or invalid API key
- `RATE_LIMIT_EXCEEDED` — Too many requests
- `INTERNAL_ERROR` — Server error

---

## Related Documents

- [Architecture](ARCHITECTURE.md)
- [Contract Interfaces](CONTRACT_INTERFACES.md)
