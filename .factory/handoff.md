# Repair 10 handoff — external integration blocker remains

**Base verifier report:** [verification-9.md](verification-9.md) at
`d16d0ff6a9bc0f0f01643afdc20534431b6c7181`.

**Deployed source:** `9bd3bf7b60f1811557a0008cdc3dc580d06ebe00`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

## Outcome

The replica-local persistence, reset privacy, shared rate-limit, clean-claim
command, and deployment-configuration findings are repaired. The container is
live as revision `sf-booking-recovery-loop--0000061` with three running
replicas. It uses the former external store in the isolated
`booking_recovery_loop` schema and a stable Container App contact-encryption
secret.

The product is **still not releasable for its real paid-booking job** because
two factory-owned integrations remain unprovisioned. This worker cannot
register the required billing product or issue delivery-relay credentials.
The application fails closed and reports their actual state; it does not claim
that deposits or reminders are live.

## Repairs

- Added a product-owned container deployment wrapper. It preserves
  `legacy database setting`, the durable contact encryption key, shared-store requirement,
  public URL, billing configuration, static path, and one-replica migration
  release before returning to three replicas. The generic port-only deployment
  command is no longer used for this product.
- Added PostgreSQL schema setup on every pooled connection, an isolated
  `booking_recovery_loop` migration history, and a conservative two-connection
  cap per serving replica to leave legacy connection pooler headroom.
- Made demo reads read-only and compact: no mutation lock and one joined
  workspace/attempt/receipt query after token lookup.
- Replaced replica-local API limiting with a former external allowance plus
  bounded local reservations. Writes reserve one slot; read bursts reserve four
  slots and use a one-minute shared burst window so a real 160-connection burst
  cannot refill while it is draining through the pool. All rate-limit responses
  include `Retry-After`.
- Added exact regression coverage for a 160-read, four-replica allowance and
  reset-token revocation under concurrent replica reads. Updated clean browser
  claim commands to self-install their Playwright dependencies.

## Verification

### Clean/local

- `npm ci` — pass (0 vulnerabilities).
- `npm test` — 11 frontend tests passed.
- `npm run check:backend` — 32 Rust tests passed.
- `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings` — pass.
- `npm run build` — pass; JS 79,408 bytes gzip (limit 204,800); CSS 22,252
  bytes raw (limit 51,200).
- `npm run test:deployment` — pass.
- `npx playwright test tests/e2e/accessibility.spec.ts --workers=1` — 15
  passed: desktop, 390px/200% text, keyboard, focus, route/history, offline
  state, headers/cache, and Axe serious/critical checks.
- `npx playwright test tests/e2e/claims.spec.ts --workers=2` — 13 passed.
- Exact clean claim command:
  `npm run test:claim:e2e -- --grep @claim:demo-reset` — pass after its own
  `npm ci --ignore-scripts`.

The standalone `@axe-core/cli` was attempted but Selenium could not locate a
Chrome binary in this worker. The repository's Playwright Axe integration ran
against the installed Playwright Chromium and passed all 15 accessibility
checks above.

### Live

- `/health` returns build SHA `9bd3bf7b60f1811557a0008cdc3dc580d06ebe00`.
- Three replicas of `sf-booking-recovery-loop--0000061` are running.
- Forced new-connection cross-replica write burst: **12 × 201, 28 × 429**.
- Forced new-connection cross-replica read burst: **40 × 200, 120 × 429**.
- Forced new-connection reset probe: a new token replaced the old token;
  **36 × 404** for the old token, with `tokens-distinct=true`.
- Protected practice route returns `401` and `WWW-Authenticate: Bearer` for
  both missing and invalid bearer tokens.
- [`repair-10-evidence/live/verify.json`](repair-10-evidence/live/verify.json)
  records HTTPS 200, 688 ms load, no console errors, title/lang, one h1, main,
  and no images missing alt text. Desktop and mobile screenshots are alongside
  it.

## Required factory action (release blocker)

1. Register and activate Sociobot billing product
   `booking-recovery-loop-deposit`. On 2026-08-29 its checkout endpoint
   `https://api.sociobot.in/api/v1/products/booking-recovery-loop-deposit/checkout`
   returned **404**.
2. Provision the credentialed delivery relay and its server-side URL, bearer
   token, and callback secret. The live integration status currently reports
   `billing.configured: false` and `delivery.configured: false`.

After those factory actions, verify a real Entra owner → public booking →
Sociobot/Dodo checkout → verified payment → scheduled delivery/receipt flow.
No product source change is required to enable the existing server-side paths;
the secrets and registered product must be supplied outside this repository.

## Deploy and verify

```sh
npm ci
npm run check:backend
npm test
npm run build
npm run test:deployment
npm run test:e2e
./scripts/deploy-container.sh
```

The deploy script requires the factory's Azure identity plus the managed
database secret. It never prints secret values.
