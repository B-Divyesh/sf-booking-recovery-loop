# Repair 9 handoff — P1 repaired; P0 external release blockers remain

**Work order:** `booking-recovery-loop-repair-9`
**Verifier report:** [verification-8.md](verification-8.md)
**Failed candidate:** `f9cc5c560ee8d548b4fbc29dde043ea5a062280b`
**Repair commit:** `522dbd0e88f1876f4dcc811be6936b7d6de2c1f5`
**Live revision:** `sf-booking-recovery-loop--0000035`
**Live URL:** <https://booking-recovery-loop.sociobot.in>

## Outcome

The multi-replica privacy and rate-limit defects from Verification 8 are
repaired in both source and production. The deployed service now uses the
factory shared PostgreSQL runtime URL, a shared server-side contact encryption
key, and `REQUIRE_SHARED_DATABASE=1`; it cannot fall back to replica-local
SQLite. The migration ran once on revision `0000034` with the factory migration
URL. Revision `0000035` uses the runtime URL and omits `RUN_MIGRATIONS`.

The two P0 external service boundaries remain unavailable and keep this product
**not releasable**: the required dedicated variable-amount deposit product is
not registered at the Sociobot billing boundary, and no approved dual-channel
delivery relay endpoint/token/callback secret has been provisioned. The app
fails closed and now reports the missing billing product honestly rather than
calling any non-empty slug “configured.” No substitute payment provider,
simulated live delivery, or client-side secret was introduced.

## Repairs and regression coverage

| Verification 8 finding | Repair | Exact coverage / evidence |
| --- | --- | --- |
| P1: 12-write allowance multiplied per replica | Deployed the declared shared PostgreSQL topology. Added a four-replica, 40-concurrent-write regression that requires exactly 12 `201` and 28 `429` responses. | `@claim:forwarded-rate-limit`; live [check](repair-9-evidence/live/live-check.json) recorded 12 created, 28 limited, `X-RateLimit-Limit: 12`, and `Retry-After: 60` for every limit. |
| P1: Reset left the old demo token usable on other replicas | Added a four-replica reset regression which fans 24 old-token reads across the shared store and requires every response to be `404`. | `@claim:demo-reset`; live [check](repair-9-evidence/live/live-check.json) recorded 24×`404`. |
| P1: billing status inferred readiness from a slug | `/api/v1/integrations/status` now verifies the non-mutating Sociobot product registry before reporting billing configured. It also has a no-secret serialization regression. | `@claim:server-owned-integration-boundary`; live status reports `billing.configured: false` for the absent deposit product. |
| P1: public auth and secret-boundary statements were absent from claims | Added explicit claim records and executable coverage. Extracted production JWT validation to a testable boundary and explicitly enabled `nbf` enforcement. | `@claim:entra-token-validation` signs an isolated RS256 token and rejects invalid issuer, audience, tenant, oid, expiry, nbf, and tampered signature. |

All 26 manifest commands were run individually after `npm ci`, including the
four new/replaced claim checks. The existing UI reset test remains in the
browser suite; the claim now points to the stronger shared-store concurrency
test that reproduces the verifier's failed topology.

## Verification

Clean local install and quality gates passed on 2026-08-29 UTC:

```sh
npm ci
npm test                         # 10 passed
npm run check:backend            # rustfmt + 29 Rust tests
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
npm run test:deployment
npm run test:e2e                 # 28 Chromium tests
npm run build
npm run check:size               # 79,408 B gzip JS; 22,252 B CSS
npm run build:backend
```

- The browser suite covers desktop, 390 px, 200% text, reduced motion,
  keyboard recovery, focus/skip link, routes, 404, offline error state,
  same-origin demo privacy, and response headers.
- [verify-url evidence](repair-9-evidence/live/verify-url/verify.json) has a
  title, `lang=en`, one `h1`, one main landmark, no missing image alt text,
  and no console errors.
- Live Playwright axe scans on `/`, `/demo`, `/privacy`, `/terms`, `/start`,
  `/app`, `/app/settings/data`, and a real 404 found no serious or critical
  violations. The repository Playwright axe integration also passed on every
  route. The standalone axe CLI was attempted but its downloaded ChromeDriver
  only supports Chrome 152 while the supplied Playwright Chromium is 145; the
  Playwright axe integration is the applicable successful alternative.
- Live Lighthouse [evidence](repair-9-evidence/live/lighthouse.json): 100
  performance, accessibility, best practices, and SEO; LCP 1,672 ms and CLS
  0. Local Lighthouse was 99 performance and 100 for the remaining categories.
- The live [check](repair-9-evidence/live/live-check.json) confirms candidate
  build SHA `522dbd0e88f1876f4dcc811be6936b7d6de2c1f5`, three running replicas,
  reset revocation, global write limiting, desktop/mobile flow, and
  same-origin demo requests. This product is not a package, CLI, or PWA, so
  consumer-package and service-worker update checks do not apply.

## Deployment record

ACR built `sociobotregistry.azurecr.io/sf-booking-recovery-loop:522dbd0e88f1`
(digest `sha256:cbe863390a91f3cf4e5bda8cf007add4517a87cc5a9d04837b72d56935339a73`).
The release migration used the factory migration secret once; runtime uses the
factory runtime secret. The generated 32-byte contact encryption key is stored
only as an encrypted Container App secret reference and is never printed or
committed. `/health` returns the repair commit SHA.

## Release blockers requiring factory authority

1. Register and enable the distinct variable-amount
   `booking-recovery-loop-deposit` product through the Sociobot billing
   registry, with the production return origin. A fresh `POST` to its approved
   checkout endpoint still returns `404 {"error":"enabled factory product"}`.
   The working `booking-recovery-loop` product is the $29/month practice
   subscription and must not be substituted for client deposits.
2. Provision the approved email/SMS relay and set its HTTPS endpoint,
   `DELIVERY_PROVIDER_TOKEN`, and `DELIVERY_CALLBACK_SECRET` on the Container
   App. Current live `/api/v1/integrations/status` reports
   `delivery.configured: false`; no live recovery/reminder/fallback can be
   claimed until a controlled booking, delivery receipt, email bounce, and SMS
   fallback pass against that relay.

These actions require factory billing and delivery-provider authority and are
outside this repository. They are the remaining P0 reason not to release.
