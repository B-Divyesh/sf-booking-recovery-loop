# Polish 3 handoff — Booking Recovery Loop

**Work order:** `booking-recovery-loop-polish-3`
**Reviewed candidate:** `256bde53b0e8107421ceda018d4b3a61203ce894`
**Deployed source:** `15bd99b9765cdbfc6cf25316948b37615323cf25`
**Live URL:** https://booking-recovery-loop.sociobot.in

## What changed

- Rewrote the landing headline as **“Recover unfinished paid-session
  bookings”** and updated home metadata. The direct `/?demo=1` path keeps its
  persistent isolated-data banner, reset control, and start-for-real exit.
- Added a durable, measured 15-minute recovery-schedule claim, a complete
  practice-data-inventory claim, a no-card-data browser claim, and a distinct
  automatic-reminder claim test. The privacy page now names every stored
  record type consistently.
- Added an owner-only **Send delivery test** action. It sends a connection-test
  payload without recipient or client data and never creates a booking.
- Preserved the product's twilight appointment-rail visual system, route
  titles, focus handoff, designed 404, legal links, mobile layout, and local
  demo namespace.
- Added the live ingress probe to `scripts/verify-live.mjs`. It requires twelve
  accepted demo writes and a thirteenth `429` with `Retry-After`.

## Verification evidence

All 23 declared claim commands were run individually in a clean clone at
`/tmp/booking-recovery-final-clean.icZuvD`, after `npm ci`. The clean clone also
passed:

```text
npm test                  10 passed
npm run check:backend     19 passed
npm run test:deployment   passed
npm run build             dist/ produced
npm run check:size        JS 12,457 bytes gzip; CSS 21,906 bytes raw
npm run test:e2e          27 passed
```

Final live verification against deployed `15bd99b`:

```text
GET /health               200; build_sha 15bd99b9765cdbfc6cf25316948b37615323cf25
scripts/verify-live.mjs   passed: demo, reset, real setup, booking, consent,
                           export/delete, routes, mobile, console, ingress limit
verify-url.sh             passed: title, lang, h1, main, image alt, no console errors
live axe                  0 violations on / desktop; /demo, /privacy, /terms at 390 px
Lighthouse mobile         Performance 100; Accessibility 100; Best Practices 100; SEO 100
LCP / CLS                 1.51 s / 0
```

Final evidence is committed under `.factory/repair-evidence/`:

- `polish-3-live-final/live-check.json` — cold live workflow and 12/13 ingress result.
- `polish-3-verify-url-final/verify.json` — semantic and console verifier result.
- `polish-3-live-final-axe.json` — live axe scans.
- `polish-3-live-final-lighthouse.json` — Lighthouse report.
- `polish-3-live-copy-check.json` — first-screen, privacy, timing, and card-boundary copy.

## Known external limitation / operator action

The factory billing registry has not enabled the required product. A direct
check of `https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout`
returned HTTP `404` with `{"error":"enabled factory product","status":404}`;
the captured headers and body are in
`.factory/repair-evidence/polish-3-billing-check.*`.

The repository must not create or alter billing products. To open the stated
$29 monthly plan, the factory billing operator must register and enable the
`booking-recovery-loop` Sociobot product. The UI deliberately says checkout is
unavailable rather than exposing a dead payment link. A practice can already
connect its existing hosted payment page and delivery endpoint, verify that
delivery connection without client data, and use the recovery workflow.

This external registration and a first-party, non-developer messaging-provider
connection are the remaining parts of review finding `F-3-2`; they cannot be
completed from this repository or with the runtime credentials supplied to the
container.
