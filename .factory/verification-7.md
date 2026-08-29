# Verification 7 — FAIL

**Candidate:** `cc5ce2c8289510bdd73e1133d5c1e99c5eab0cf9`  
**Live URL:** <https://booking-recovery-loop.sociobot.in>  
**Verified:** 2026-08-29 UTC  
**Decision:** **FAIL — do not release.**

## Release blockers

### P0 — the product cannot perform its central recovery/reminder job in production

The researched minimum product requires automatic consent-gated email/SMS
recovery, a reminder delivery receipt, and SMS/email fallback. Fresh live
evidence contradicts that capability:

- `/start` says: “Live email and SMS delivery are not enabled in this
  deployment.” It further says the product will not accept client contact data
  for a provider connection until a credentialed provider is configured.
- The checked candidate implements `delivery_not_connected` for a real
  recovery/reminder when `delivery_webhook_url` is empty
  (`backend/src/routes/practice.rs`). The production form supplies that field
  as a hidden empty value.
- The only end-to-end receipt observable without a provider is the demo's
  explicitly simulated receipt. It cannot establish recovery of a real
  abandoned booking, a reminder, or an SMS fallback.

This is an honest disclosure, but it means the product does not meet the
brief's real job-to-be-done. Unit claims use an in-process provider fixture;
they do not prove a configured production delivery path.

### P1 — no product-owned Stripe deposit collection or Stripe verification

The public booking product asks each practice to paste an arbitrary “Hosted
deposit URL”. The server accepts a generic HTTPS URL and redirects there; it
does not create a per-booking Stripe Checkout session or verify a signed Stripe
event. Its generic provider callback is not a Stripe integration. This falls
short of the brief's Stripe deposit requirement and leaves the paid-booking
completion chain outside the product's accountable boundary.

## Required pre-release remediation

1. Provision an approved, credentialed email and SMS delivery adapter; wire
   provider-status/webhook authentication, consent-gated automatic recovery,
   one-bounce SMS fallback, and durable receipts. Add a deployed-path test,
   not only an in-process fixture.
2. Implement Stripe-hosted session deposit creation per booking and verified
   Stripe webhook processing, or explicitly revise the accepted product scope
   before release. Do not rely on a pasted arbitrary payment URL.
3. Re-run the full verification against the deployed revision after those
   external boundaries are configured.

## Mandatory claims gate — PASS

`.factory/claims.json` exists and has 24 entries. Every listed command was
run individually from this clean checkout. All passed; the complete command
output is at
[`claims-verification-7.log`](verification-artifacts/claims-verification-7.log).

This includes demo isolation/lifetime/reset, all demo privacy assertions,
recorded consent, payment state integrity, encryption and tenant separation,
export/delete, delivery-fallback fixture behavior, automatic job fixture
behavior, double-booking, and forwarded-IP limiting.

## Cold first read — PASS

A new browser context opened `/` at desktop width. The first screen says it
“Recover[s] unfinished paid-session bookings”, names “solo coaches, tutors,
and consultants”, and makes **Try it with sample data** the obvious first
action. The adjacent explanation says that it opens three fictional bookings
and can be reset. The cold-load capture is
[`cold-live-desktop.png`](verification-artifacts/cold-live-desktop.png).

## Local verification — PASS

Run from a fresh `npm ci` install:

| Check | Result |
| --- | --- |
| `npm test` | PASS — 10 tests |
| `npm run check:backend` | PASS — rustfmt plus 23 Rust tests |
| `npm run test:deployment` | PASS |
| `npm run test:e2e` | PASS — 27 Playwright tests (`test-results/.last-run.json`) |
| `npm run build` | PASS — typecheck and production Vite build |
| `npm run check:size` | PASS — JS 79,017 bytes gzip; CSS 22,252 bytes |
| `npm run build:backend` | PASS — optimized Rust binary |
| `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings` | PASS |

The optimized binary started with only `PORT=4181` supplied. It generated or
used its local defaults, served `/health`, and shut down cleanly. Docker was
not installed in this verification container, so an image build was not
independently runnable here.

## Live deployment verification — PASS except blockers above

- `GET /health` returned 200 and
  `{"status":"ok","build_sha":"cc5ce2c8289510bdd73e1133d5c1e99c5eab0cf9"}`;
  the live deployment is the candidate.
- A fresh demo context made only same-origin document, asset, font, and
  `/api/v1/demo/workspaces` requests. It made no payment, messaging, Entra,
  billing, or AI request. Desktop and 390px captures are
  [`live-demo-desktop.png`](verification-artifacts/live-demo-desktop.png) and
  [`live-demo-mobile.png`](verification-artifacts/live-demo-mobile.png).
- Axe found no serious or critical findings on `/`, `/demo`, `/privacy`,
  `/terms`, `/start`, `/app`, or `/404`; each had exactly one `h1` and one
  `main`. No console/page errors occurred. Under reduced motion, the 390px
  and desktop demo screens loaded cleanly. Keyboard focus begins at the skip
  link and visibly uses a 3px blue outline.
- All discovered internal links returned 200. The browser sign-in redirect
  was exactly the Sociobot Entra External ID authority
  `https://sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/...`.
- The public Sociobot checkout link now returns 303 to a Dodo checkout; this
  prior deployment concern is resolved.
- Response headers include CSP with `frame-ancestors 'none'`, `nosniff`,
  `DENY`, strict-origin referrer policy, and restrictive permissions policy.
  HTML is `no-cache`; hashed assets are `public, max-age=31536000, immutable`.
- Rate limiting was observed live on a fresh reserved test IP: requests 1–12
  to `POST /api/v1/demo/workspaces` returned 201 with limit 12; request 13
  returned **429** with `Retry-After: 60`, `X-RateLimit-Limit: 12`, and
  `X-RateLimit-Remaining: 0`. Twenty concurrent reads of one isolated demo
  workspace all returned 200 for the same workspace ID.

## Scope notes

This is a web-with-backend product, not a library/CLI or PWA; package-consumer,
service-worker update, and offline-reload checks do not apply. No real Entra
credential was available, so QA verified the required sign-in authority and
PKCE redirect initiation but did not complete an account login.

No product code was changed during this verification.
