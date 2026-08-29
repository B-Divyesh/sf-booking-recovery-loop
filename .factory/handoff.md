# Polish 1 handoff — Booking Recovery Loop

**Work order:** `booking-recovery-loop-polish-1`

**Date:** 2026-08-29

**Live URL:** `https://booking-recovery-loop.sociobot.in`

**Verified live build:** `e504d7f743c661f457aa52c80b3c315492cc3ffe`

## What changed

- Rewrote the first screen and replaced both metaphor headings with concrete section names.
- Preserved the twilight appointment-rail identity and improved the 390 px navigation.
- Kept `/demo` and `/?demo=1` isolated, bannered, resettable, and same-origin only.
- Added `/start`, `/app`, `/app/settings/data`, and dynamic `/b/<slug>` and payment-return routes.
- Added owner-scoped practices, AES-256-GCM contact encryption, booking attempts, timestamped consent, slot protection, hosted-payment handoff and authenticated confirmation, provider receipts, one permitted SMS fallback, JSON export, and deletion.
- Expanded `.factory/claims.json` from 8 to 16 claims with one exact command per promise.
- Updated metadata, canonicals, sitemap, legal copy, README, catalog description, design notes, component inventory, and copy audit.

## Verification evidence

Fresh clone at `e504d7f743c661f457aa52c80b3c315492cc3ffe`:

- all 16 claim commands passed individually;
- `npm test`: 10 passed;
- backend: 14 passed, including migration reversal and production claims;
- Playwright: 25 passed across claims, keyboard, mobile, offline, privacy, focus/history, axe, caching, and 404 behaviour;
- build produced `dist/`; initial JS was 12,116 bytes gzip and CSS was 21,564 bytes raw.

Logs: `.factory/evidence/polish-1-clean/claims.log` and `full-suite.log`.

Local Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100, SEO 100, LCP 1.6 s, CLS 0, total 137 KiB. Live Lighthouse: 100/100/100/100, LCP 1.7 s, CLS 0, total 134 KiB.

Cold live verification returned `passed: true` and proved:

- exact landing title and h1, no console errors, and no 390 px overflow;
- three demo tickets, persistent banner, simulated receipt, token rotation, and same-origin-only requests;
- creation and deletion of a fictional real practice;
- owner token issuance, public session page, recorded email consent, hosted-payment navigation, JSON export, and deleted-key invalidation;
- all named routes return 200 and an unknown path returns 404;
- 100 concurrent `/health` requests returned 200;
- demo writes 1–12 returned 201, then write 13 returned 429 with `Retry-After: 60`;
- deployed image `e504d7f743c6` runs at one replica.

Primary live record: `.factory/evidence/polish-1-live/live-check.json`.

## Run and verify

```sh
npm ci
npm test
npm run check:backend
npm run test:deployment
npm run test:e2e
npm run build
npm run check:size
```

Live verifier:

```sh
node scripts/verify-live.mjs https://booking-recovery-loop.sociobot.in .factory/evidence/polish-1-live
```

## Needs operator action

The required Sociobot billing product is not enabled. A cold request to
`https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout`
returns HTTP 404 with `{"error":"enabled factory product"}`; evidence is in
`.factory/evidence/polish-1-live/sociobot-checkout.*`. The product does not show
a broken checkout or claim the $29 subscription is purchasable.

The shared Entra callback is also not registered from this repository. This
release uses a generated 256-bit private practice key instead. Register the
callback and billing product before replacing that access model or advertising
the paid subscription.

No source TODOs or fake payment/delivery successes remain. A real delivery is
recorded only after the configured HTTPS provider accepts it. A deposit becomes
paid only after an authenticated provider callback.
