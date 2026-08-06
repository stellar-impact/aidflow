# AidFlow Client Applications

This directory contains all client-facing applications for AidFlow.

## Applications

### Donor Dashboard (`donor-dashboard/`)
**Stack:** Vite + React + TypeScript + TanStack Query

NGO donor interface for creating programs, funding escrows, and viewing audit timelines.

**Setup:**
```bash
cd donor-dashboard
npm install
npm run dev
```

### Beneficiary PWA (`beneficiary-pwa/`)
**Stack:** Vite + React + TypeScript + Workbox

Progressive Web App for beneficiaries to claim vouchers, view balances, and generate QR codes for redemption. Includes offline support and passkey authentication.

**Setup:**
```bash
cd beneficiary-pwa
npm install
npm run dev
```

### Merchant App (`merchant-app/`)
**Stack:** Expo React Native

Mobile app for merchants to scan beneficiary QR codes and redeem vouchers. Includes offline transaction queue and SEP-24 anchor cash-out integration.

**Setup:**
```bash
cd merchant-app
npm install
npm start
```

## Shared Dependencies

All clients depend on `@aidflow/contract-sdk` for contract interactions.

## Development

```bash
# Install all client dependencies from root
npm install

# Build all clients
npm run build --workspaces

# Run tests
npm test --workspaces
```

## Deployment

- **Donor Dashboard:** Static hosting (Vercel, Netlify)
- **Beneficiary PWA:** Static hosting with service worker
- **Merchant App:** Expo Application Services (EAS) or self-hosted

---

See individual app README files for detailed setup instructions.
