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

The container deploy was started through the factory work-order configuration
for `booking-recovery-loop` at this commit. Append the cold live response,
health build SHA, and screenshot evidence here after the deployment completes.

## Known external boundary

The Sociobot billing product registration is factory-owned and currently
returns `404` for this slug. This repair does not present a dead checkout link.
Register the `$29/month` product with Sociobot billing to enable checkout.
Practices also supply their own hosted deposit and delivery connections; card
details never enter this product.
