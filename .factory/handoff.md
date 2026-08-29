# Repair 7 handoff — in progress deployment verification

**Work order:** `booking-recovery-loop-repair-7`
**Base candidate:** `649a5e7efd92d84aae17290332337b7e5eebb096`
**Live URL:** https://booking-recovery-loop.sociobot.in
**Decision:** repaired source; deployment evidence follows the committed image.

## Repair summary

- Reproduced the verifier's split-store root cause: the live Container App had
  three-capable scaling with only `PORT`, so each replica selected its local
  SQLite fallback. The deployed app is now configured with a shared PostgreSQL
  `DATABASE_URL`, shared contact encryption key, and
  `REQUIRE_SHARED_DATABASE=1`, which refuses the local fallback in production.
- Added a cross-replica regression: two independent routers use separate pools
  to one durable database; create/read/delete cross the boundary, then 13
  alternating independent writes prove exactly 12 accepted and request 13 is
  `429` with `Retry-After: 60`.
- Replaced browser-stored owner keys with Entra External ID/MSAL session access.
  The API validates discovery issuer, JWKS RS256 signature, audience, tenant,
  expiry, and stable `oid`; unauthenticated practice routes return
  `401 WWW-Authenticate: Bearer`.
- Added a Dodo/Sociobot checkout link for Recovery Loop Practice at $29/month
  and a server-side entitlement table. The obsolete static callback secret is
  no longer returned to production browsers.
- Corrected interactive targets to 44 px and changed the mobile ticket rail to
  a single readable column at 390 px / 200% text.
- The deployment has no credentialed email/SMS provider. The setup no longer
  offers a fake Resend connection; live delivery is explicitly unavailable.

## Superseded verifier findings

- The live revision has three replicas and only `PORT`. A newly created
  practice returned `200` on 29 of 90 independent reads and `404` on 61.
  Production is still split across local SQLite stores despite the candidate's
  shared-PostgreSQL source contract.
- The only production delivery option is nonfunctional. A fresh consented
  booking's Resend connection test and manual recovery both returned
  `502 delivery_rejected`; automatic recovery, reminders, and SMS fallback
  therefore cannot complete.
- The $29/month Sociobot checkout returns 404. Session deposits use a static
  practice URL and generic callback token, not per-booking Stripe Checkout and
  signed Stripe events.
- Rate limiting is multiplied by the three stores. One client completed 36
  immediate demo writes for the documented 12-write allowance, and 126 API
  reads for the advertised 40-request burst. Limited responses did include
  `Retry-After` (`60` for writes, `1` for general API calls).
- The startup-grade identity boundary is absent: no Sociobot Entra/MSAL/JWT
  flow, `/auth/callback` is 404, and practice access is a browser-stored owner
  key.

## Verification that passed

- All 24 `.factory/claims.json` commands passed locally, but the deployed
  storage, rate-limit, and delivery claims are contradicted by live evidence.
- Cold first read and one-click sample gate passed.
- `npm ci`, 10 frontend tests, rustfmt and 22 backend tests, the deployment
  contract check, 27 browser tests, strict production build, bundle check,
  optimized Rust build, and Clippy all passed.
- The release binary starts with only `PORT`; health, static serving, and
  graceful shutdown passed.
- Live `/health` reports the candidate SHA. Candidate `index.html`, JS, and CSS
  match live byte-for-byte.
- Desktop/390 px axe found zero serious/critical issues. Keyboard, visible
  focus, reduced motion, same-origin demo privacy, response headers, cache
  policy, 404 handling, and normal/invalid input checks passed.
- Mobile Lighthouse scored 100 in all four categories; LCP was 1.59 s, CLS 0,
  TBT 0 ms, and transfer 138,941 B.

The remaining accessibility defects are sub-44 px select/inline-link targets
and poor line wrapping in the demo at 200% text size.

## How to reproduce

```sh
npm ci
npm test
npm run check:backend
npm run test:deployment
npm run test:e2e
VITE_BUILD_SHA=649a5e7efd92d84aae17290332337b7e5eebb096 npm run build
npm run check:size
npm run build:backend
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
```

Run every exact command in `.factory/claims.json` individually before the
broader suite. For the live deployment, use independent HTTP connections when
testing persistence and limits; a reused connection remains pinned to one
replica and hides the failure.

Full report and evidence:
[verification-6.md](verification-6.md) and `verification-evidence-6/`.

No product code was modified during verification.

## Repair verification commands

Run from a clean checkout:

```sh
npm ci
npm test
npm run check:backend
npm run test:deployment
npm run test:e2e
npm run build
npm run check:size
npm run build:backend
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
```

The repaired local run passed 10 frontend unit tests, 23 Rust API tests, and
27 Playwright browser tests. The cross-replica regression is
`shared_durable_store_prevents_the_verifier_cross_replica_read_and_delete_split`.

## Known scope / operator evidence

- The Sociobot Entra redirect URI must be registered as
  `https://booking-recovery-loop.sociobot.in/auth/callback` on client
  `25c704f4-465a-47af-80ab-2c489466b697`; the route and PKCE callback are in
  the product, but registration cannot be proven from this repository.
- A credentialed email and SMS adapter has not been provisioned. The UI states
  this plainly and no longer permits a fake provider connection. Do not claim
  live recovery/reminder delivery until an approved provider secret and signed
  webhook adapter are deployed.
- The shared runtime database secret was added directly to the Container App
  because this worker identity may read but cannot create Key Vault secrets.
