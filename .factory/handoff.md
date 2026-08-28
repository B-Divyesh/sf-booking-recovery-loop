# Repair handoff — repository QA repaired, product release still blocked

**Work order:** `booking-recovery-loop-repair-1`

**Base report:** `841dd239418a4f5b8204a1838282521a85fc50e9`

**Rejected candidate:** `d03d83db200435a8582ea5fac676139abfb139cb`

**Date:** 2026-08-28 UTC

## Outcome

The reproducible M1 defects from the independent report are repaired and have
exact regression coverage. The candidate is still not the complete product in
the researched brief: authenticated practices, a real booking/deposit path,
provider-backed reminders and fallback, data export/deletion, and the paid
subscription are the planned M2–M5 work and do not exist in this repository.
This handoff therefore does **not** claim product release readiness.

The external paid boundary is also not provisioned. On 2026-08-28,
`GET https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout`
returned `404 {"error":"enabled factory product","status":404}`. Repository
rules prohibit registering billing infrastructure from this repair. Entra OIDC
discovery is reachable, but no production auth code has been shipped and the
callback registration has not been confirmed.

## Repairs completed

- Concurrent recovery writes now use conflict-tolerant insertion. Eight valid,
  uniquely keyed requests return eight `200` responses and create one message.
- The Playwright server allowance is 360 seconds, covering a cold Rust/sqlx
  build instead of failing the first claim at 120 seconds.
- `.factory/claims.json` now registers the token entropy/TTL, first-forwarded-IP
  rate limit and retry header, no-account/no-payment demo entry, data isolation,
  reset, consent, receipt, and same-origin privacy claims.
- The isolation claim inserts a non-demo fixture, attempts both read and
  mutation through the demo API, and confirms the fixture is unchanged.
- At 390 px with text enlarged to 200%, layout width remains 390 px. Headings,
  navigation, and the build identifier wrap; footer links are at least 44 px.
- Hashed assets and self-hosted fonts return
  `Cache-Control: public, max-age=31536000, immutable`; HTML is `no-cache`.
- Unknown routes return HTTP 404 while rendering the accessible product-native
  not-found screen. `/404` remains a directly reachable 200 route.
- The expected no-consent action stops before `fetch`, so it produces no
  Chromium console error; the API still independently enforces the 409 guard.
- Both Dockerfiles use `rust:1-slim`; the configured root `Dockerfile` now
  exists, accepts `BUILD_SHA`, builds without `.git`, and runs non-root.
- Response headers, no-service-worker/offline messaging, desktop/mobile axe,
  keyboard operation, and no-console behavior have browser regressions.

## Verification evidence

Clean dependency install: `npm ci` — 62 packages, zero vulnerabilities.

Commands passed:

```text
npm test                                      2 files, 9 tests
npm run check:backend                         9 Rust tests
npm run test:e2e                              17 Chromium tests
npm run build                                 dist/ produced
npm run check:size                            JS 8,392 B gzip; CSS 19,123 B raw
npm run build:backend                         optimized release build
```

Every exact command in `.factory/claims.json` passed individually. The cold
Rust build completed in 73 seconds, inside the new 360-second harness limit.
The runtime was also started with only `PORT=4180`; `/health` returned 200 and
`{"status":"ok","build_sha":"dev"}`.

Local Chromium evidence is in `.factory/repair-evidence/`:

- `desktop.png` and `mobile390.png`: full demo at 1366×900 and 390×844;
  both recorded zero console errors and no horizontal overflow.
- `local-404.headers`: unknown route is `HTTP/1.1 404 Not Found`.
- `local-asset.headers`: immutable cache policy plus CSP/security headers.
- `lighthouse-local.json`: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 1.1 s, LCP 1.5 s, CLS 0, TBT 0 ms.

Docker/Podman is not installed in the worker. The root Dockerfile is verified
by the factory ACR remote build during deployment.

## Remaining release blocker and next work

The independent report's critical scope finding cannot be honestly closed by
patching the M1 demo. Complete M2–M5 in `.factory/plan.md`: Entra authentication
and tenant isolation; registered Sociobot $29/month subscription; branded
session page and Stripe-hosted client deposit; scheduled consent-aware provider
delivery/fallback; encrypted contact persistence; receipts/outcomes; and
self-service export/deletion. Register the billing product and Entra callback
`https://booking-recovery-loop.sociobot.in/auth/callback` through the factory
before claiming those flows are live.

The product deliberately has no service worker or offline-work claim. The
shell explains the offline demo state and the browser test confirms no service
worker is registered. Package/consumer testing is not applicable to this
container web product. No AI feature is warranted by the brief, and none was
added.
