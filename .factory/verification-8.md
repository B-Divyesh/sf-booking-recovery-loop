# Verification 8 — FAIL

**Candidate:** `f9cc5c560ee8d548b4fbc29dde043ea5a062280b`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

**Verified:** 2026-08-29 UTC

**Decision:** **FAIL — do not release.**

The candidate is polished, accessible, fast, and byte-for-byte deployed, but
the live service still cannot perform its paid-booking recovery job. Fresh
multi-replica checks also falsify the reset and 12-write-limit claims.

## Release-blocking findings

### P0 — live client deposits and message delivery are unavailable

The researched minimum product requires a hosted client deposit, automatic
email/SMS recovery, reminder receipts, and an email-bounce SMS fallback. The
live candidate cannot complete either external boundary:

- `GET /api/v1/integrations/status` returned 200 with
  `delivery.configured: false`.
- An exact representative `POST` to
  `https://api.sociobot.in/api/v1/products/booking-recovery-loop-deposit/checkout`
  returned 404 and `{"error":"enabled factory product","status":404}`.
- A GET to the same dedicated deposit product also returned 404.
- The endpoint reports `billing.configured: true` merely because the product
  slug is non-empty, even though that configured product does not exist.
- The separate `$29/month` practice-subscription checkout works and returns
  303 to `checkout.dodopayments.com`; it is not a client deposit and cannot be
  substituted.

The backend deliberately rolls a booking back when its dedicated checkout is
rejected. It also fails message sends closed while delivery is unconfigured.
Therefore a real client cannot finish a new paid booking, and the service
cannot send the recovery/reminder/fallback messages that define the product.
The deterministic local provider fixtures pass, but they do not make the live
job work.

### P1 — the documented 12-write allowance multiplies across replicas

The sequential live probe passed: requests 1–12 from one forwarded IP returned
201, request 13 returned 429, and `Retry-After: 60` was present.

The required concurrent/multi-replica probe failed. Forty simultaneous
`POST /api/v1/demo/workspaces` calls from one fresh `X-Forwarded-For` address
produced:

| Result | Count |
| --- | ---: |
| 201 Created | 36 |
| 429 Too Many Requests | 4 |

The 429 responses correctly contained `Retry-After: 60` and
`X-RateLimit-Limit: 12`, but one client received three replicas' allowances
before limiting. The observed effective allowance was **36**, not the
documented **12**. This contradicts claim `forwarded-rate-limit` and the
mandatory backend rate-limit contract.

A separate 100-request concurrent GET probe did return 429 with
`Retry-After: 1` and limit 40, but allowed 82 requests. This is consistent with
replica-local 40-request counters plus one-second window rollover; it does not
repair the write-limit failure.

### P1 — Reset demo does not revoke the old token across replicas

The normal browser reset rotates to a fresh token and redraws the three seed
bookings. Under cross-replica traffic, the old workspace remains accessible:

1. Create one demo token.
2. Issue 24 concurrent reads so the portable token is handled across replicas.
3. Call `POST /api/v1/demo/reset`; it returns 200 and a different token.
4. Issue 24 reads with the old token.

Observed result: **16 old-token reads returned 200** and only 8 returned 404.
The successful reads exposed two still-live old workspace IDs. This directly
contradicts claim `demo-reset`, whose sandbox says the old workspace returns
404, and the privacy copy saying reset makes the current workspace
inaccessible.

The same probe showed internal workspace IDs varying across replicas. A
rotated `recovered` token did preserve the visible recovered state and one
receipt across 20 concurrent reads, so the portable state marker works; global
revocation does not.

### P1 — security/privacy claims exist outside the claim manifest

The README and setup copy make additional reliance claims that have no exact
entry in `.factory/claims.json`, including:

- “Subscription entitlement state is stored server-side; the browser never
  receives a billing secret.”
- the asserted Entra discovery/issuer/JWKS/audience/tenant/expiry validation;
- provider addresses, access tokens, and callback secrets never entering the
  form or browser storage.

Related tests exist, but the claims contract requires each public claim to have
one explicit claim ID and test. These statements must be added to the manifest
with observable tests or narrowed/removed.

## Mandatory claims gate

`.factory/claims.json` exists with 24 entries. Per the instruction to run the
manifest before anything else, the first invocation was attempted before npm
dependencies were installed. The 14 Rust commands passed; the 10 Playwright
commands could not start because local `@playwright/test` was not yet present.
After the required clean `npm ci`, every manifest command was rerun separately
and **all 24 passed**.

This proves the deterministic single-process sandboxes, but fresh live
evidence above falsifies two deployed claims (`forwarded-rate-limit` and
`demo-reset`). A local passing fixture is not sufficient when the public
multi-replica result differs.

## Cold first-read gate — PASS

A new 1440×900 browser context opened `/` with no prior state. The first
viewport says:

- what: “Recover unfinished paid-session bookings”;
- for whom: “solo coaches, tutors, and consultants”;
- first action: **Try it with sample data**;
- outcome: “See three fictional bookings, then reset the sample at any time.”

The action is visible and opens the complete three-booking demo in one click.
The demo banner remains visible and says sample data is not saved as real
practice data.

## Clean local verification

The checkout was clean and exactly at the candidate before verification.
`npm ci` installed 64 packages with zero audit vulnerabilities.

| Check | Result |
| --- | --- |
| All 24 claim commands, individually after install | PASS |
| `npm test` | PASS — 10 tests |
| `npm run check:backend` | PASS — rustfmt and 24 Rust tests |
| `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings` | PASS |
| `npm run test:deployment` | PASS |
| `npm run test:e2e` | PASS — 28 Chromium tests |
| `npm run build` | PASS — strict typecheck and Vite production build |
| Candidate-stamped frontend build | PASS |
| `npm run check:size` | PASS |
| `npm run build:backend` | PASS |
| `cargo build --release --locked` with candidate SHA | PASS |

Candidate-stamped output was 79,444 bytes gzip JavaScript and 22,252 bytes raw
CSS. The two self-hosted fonts total 70,616 bytes. The release binary started
with only `PORT` plus a process `PATH`, returned 200 from `/health` with the
candidate SHA, logged generated/persisted config sources without secret values,
and shut down cleanly. Docker is not installed in this verifier container, so
the Dockerfile could not be executed; inspection confirms multi-stage builds,
`rust:1-slim`, `ARG BUILD_SHA=dev`, no `.git` dependency, a non-root distroless
runtime, and port 8080.

## Live deployment identity — PASS

- `/health` returned 200 with build SHA
  `f9cc5c560ee8d548b4fbc29dde043ea5a062280b`.
- The footer showed the same complete SHA.
- A candidate-stamped local build produced `index-DpRNaxEx.js`; its SHA-256
  was `17f8439c4a0d3827072318777ae557885aec8aa05e07c7ceccb8d41256314ee4`,
  exactly matching the live JavaScript bytes.
- Local and live CSS SHA-256 both equal
  `1a980601b6b1504be6686eb0c91197a83acf0e59616a9f42b01c6dc624b7cc31`.

The public deployment is the candidate; this is not a stale-revision failure.

## Functional and failure-path checks

- The live demo opened three fictional bookings with no account or payment.
- Maya's permitted keyboard-triggered sample recovery produced one timestamped
  simulated email receipt.
- Jordan's no-consent API recovery returned 409 `consent_required`; a reload
  retained `unfinished` with zero receipts.
- A missing idempotency key returned 400 `idempotency_key_required`.
- An invalid demo token returned 400 `demo_token_required`.
- Practice setup rejected a one-character name, invalid slug, 14-minute
  duration, negative deposit, and two-letter currency through native form
  validation. The documented lower bounds (15 minutes, zero deposit) and upper
  bounds (480 minutes, 1,000,000 minor units) were valid.
- An unauthenticated practice request returned 401 with
  `WWW-Authenticate: Bearer`.
- An unsigned provider callback returned 401 and did not accept the payload.
- Twenty concurrent reads of a rotated recovered demo token all returned 200
  and retained the recovered state and one receipt.

The complete real workflow could not be run live because the dedicated deposit
product and delivery provider are unavailable, which is the P0 finding above.

## Accessibility, mobile, and browser quality — PASS

- Fresh axe scans found zero serious or critical findings on `/`, `/demo`,
  `/privacy`, `/terms`, `/start`, `/app`, `/app/settings/data`, and the real
  404 response.
- Every route had `lang="en"`, exactly one `h1`, and exactly one `main`.
- Keyboard focus began on “Skip to main content” with a visible 3 px sky-blue
  outline; Enter moved focus to `<main>`. The full demo recovery worked with
  keyboard activation.
- At 390×844 with reduced motion, the demo had no horizontal overflow, no
  interactive target under 44 px, and no console errors.
- At 390 px with root text at 200%, there was no horizontal overflow or target
  below 44 px and the content remained available.
- Valid routes and demo actions produced no console or page errors. Loading the
  intentional 404 document produced the expected browser 404 console message.
- All discovered internal links returned 200; the practice subscription link
  returned the expected 303 hosted-checkout redirect.

Fresh mobile Lighthouse: performance 100, accessibility 100, best practices
100, SEO 100; FCP 1.2 s, LCP 1.7 s, TBT 0 ms, CLS 0, total transfer 153 KiB.

## Privacy, headers, and caching — PASS except findings above

The complete fresh demo/recovery request log contained only
`https://booking-recovery-loop.sociobot.in`; no payment, messaging, Entra,
billing, AI, font-CDN, or analytics request occurred. The page stores only its
demo token for the sample flow.

Responses include CSP with `frame-ancestors 'none'`, `nosniff`, `DENY`, a
strict-origin referrer policy, and a restrictive permissions policy. HTML and
404s use `no-cache`. Hashed JS/CSS and self-hosted fonts use
`public, max-age=31536000, immutable`. `robots.txt` and `sitemap.xml` are live.

Sign-in redirects only to the required Sociobot tenant:

- authority: `sociobotcustomers.ciamlogin.com`;
- tenant: `35c6fe40-0ec0-46b6-98c6-213ad4de6650`;
- client: `25c704f4-465a-47af-80ab-2c489466b697`;
- redirect: `https://booking-recovery-loop.sociobot.in/auth/callback`;
- authorization code flow with PKCE S256 and `openid profile email` scopes.

No test user credential was available, so an actual Entra login was not
completed. This product is not a library, CLI, or PWA; consumer-package and
service-worker/offline-install checks do not apply.

## Required remediation

1. Provision and verify the approved credentialed email/SMS delivery adapter,
   then complete a controlled live email-bounce-to-SMS flow with durable
   receipts.
2. Register the dedicated variable-amount
   `booking-recovery-loop-deposit` product and prove a live booking can create
   its Dodo-hosted checkout and verify completion. Make integration status test
   the provider rather than treating a non-empty slug as configured.
3. Put demo revocation and API counters in one genuinely shared production
   store, or otherwise enforce them globally. Re-test reset using the old token
   across every replica and enforce 12 total concurrent writes per client.
4. Add explicit claim entries/tests for the unlisted auth and secret-boundary
   statements.

No product code was changed during this verification.
