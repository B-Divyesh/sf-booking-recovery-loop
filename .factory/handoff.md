# Polish 2 handoff — Booking Recovery Loop

**Work order:** `booking-recovery-loop-polish-2`  
**Repair commit:** `7e7194b0f1a0d4f0585e55fadf324bbe2ba903b0`

## Delivered

- Durable automatic abandoned-booking recovery after 15 minutes.
- Durable session reminders after authenticated payment confirmation.
- Consent withdrawal stops queued delivery; provider failures retry and remain visible.
- Isolated `?demo=1` sample, persistent banner, reset, legal routes, metadata,
  route focus, mobile layout, accessibility, and product-specific twilight rail UI.
- Claims ledger expanded to 19 executable entries and plain-word consent wording.
- Exact $29/month price is visible. Checkout is truthfully marked unavailable
  because the factory billing registry currently returns 404 for this slug.

## Verification

From a clean clone (`/tmp/booking-recovery-clean.7Jf6LM`):

```text
npm ci                              passed, 0 vulnerabilities
npm test                            10 passed
npm run build                       passed; dist/ produced
npm run check:backend               15 Rust tests passed
npm run test:e2e                    26 Playwright tests passed
npm run test:deployment             passed
npm run check:size                  JS 12,316 B gzip; CSS 21,906 B raw
```

The claim commands in `.factory/claims.json` were executed individually from
the repair tree; the full clean-clone browser suite includes every browser
claim. Accessibility coverage uses axe on every public route, keyboard and
skip-link use, 390 px reflow, 200% text reflow, focus, offline state, privacy
request logging, and the designed HTTP 404. Local screenshots are in
`.factory/evidence/polish-2-local/`.

## Deploy and live check

The container deployed through the factory work-order configuration. A cold
live check at `https://booking-recovery-loop.sociobot.in` passed:

```text
GET /health                         200; build 7e7194b0f1a0d4f0585e55fadf324bbe2ba903b0
GET /missing-page                   404
verify-url                          title/lang/h1/main/alts/buttons/console pass
verify-url cold load                549 ms; zero console errors
live 390 px ?demo=1                 banner, 3 tickets, no horizontal overflow
live 390 px /                       $29 price, unavailable checkout state, legal links, no overflow
```

Live evidence is in `.factory/evidence/polish-2-live/`, including the desktop
and mobile screenshots, health response, 404 response, and verifier JSON.
`@axe-core/cli` could not launch its own Chrome binary in this worker; the
Playwright axe integration completed in the 26-test browser suite instead.

## Known external boundary

The Sociobot billing product registration is factory-owned and currently
returns `404` for this slug. This repair does not present a dead checkout link.
Register the `$29/month` product with Sociobot billing to enable checkout.
Practices also supply their own hosted deposit and delivery connections; card
details never enter this product.
