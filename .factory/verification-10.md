# Verification 10 — FAIL

**Candidate:** `e800fff12a71d3e0867f16d13bbe9caf480eca11`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

**Verified:** 2026-08-30 UTC

**Work order:** `booking-recovery-loop-verify-10`

## Decision

**FAIL — do not release.**

The candidate is deployed exactly, the earlier SQLite/topology defect is fixed,
all 28 declared claim commands pass, and the demo, accessibility, security,
rate limiting, and performance checks are strong. The production service still
cannot perform the brief's smallest useful job. Its dedicated booking-deposit
product is unavailable and its delivery relay is unconfigured, so a client
cannot finish a paid booking and the service cannot send a real recovery or
reminder. Passing isolated provider fixtures do not make those live workflows
available.

## Release-blocking finding

### P0 — the live paid-booking recovery loop has no deposit or delivery integration

Fresh production evidence on 2026-08-30:

- `GET /api/v1/integrations/status` returned 200 with
  `billing.configured: false` for `booking-recovery-loop-deposit` and
  `delivery.configured: false`.
- `GET https://api.sociobot.in/api/v1/products/booking-recovery-loop-deposit/checkout`
  returned 404 with `{"error":"enabled factory product","status":404}`.
- The separate practice subscription endpoint
  `/api/v1/products/booking-recovery-loop/checkout` returned 303 to the hosted
  Dodo checkout. That $29 subscription does not replace a client's booking
  deposit.
- The live site promises a server-created checkout for each booking and real
  consented delivery receipts. With both product-specific integrations absent,
  a stranger cannot complete the real job end to end.

This violates the researched smallest useful product and definition-of-done
item 1. The backend fails closed, which is correct safety behavior, but the
product is not releasable until both integrations are provisioned and tested
through the public workflow.

## Mandatory first gates

### Claims gate — all declared commands passed

`.factory/claims.json` exists with 28 entries. Every listed command was run
exactly as written, in manifest order, before any other repository inspection.
All 28 exited 0 from the clean checkout. The first Rust invocation performed a
cold dependency compile; the browser claim command installs its pinned
dependencies itself and therefore ran successfully without a separate setup
step.

| Claim | Declared test | Result |
| --- | --- | --- |
| `demo-isolated` | isolated real/demo fixture | PASS |
| `demo-lifetime` | 256-bit token and exact 24-hour expiry | PASS |
| `sqlite-restart-persistence` | close/reopen mounted-path SQLite fixture | PASS |
| `forwarded-rate-limit` | 40 concurrent writes: exactly 12 accepted | PASS |
| `forwarded-read-rate-limit` | 160 concurrent reads: exactly 40 accepted | PASS |
| `demo-no-account-payment` | fresh browser demo | PASS |
| `demo-reset` | old token revoked across independent connections | PASS |
| `consent-gates-recovery` | no-consent browser recovery | PASS |
| `demo-recovery-receipt` | timestamped simulated receipt | PASS |
| `demo-no-external-requests` | full same-origin request log | PASS |
| `sample-three-bookings` | exactly three named sample tickets | PASS |
| `practice-publish` | Entra-owned workspace fixture and public page | PASS |
| `booking-consent-record` | consent recorded before hosted handoff | PASS |
| `encrypted-tenant-data` | encrypted, tenant-scoped contact fixture | PASS |
| `shared-practice-storage` | independent SQLite read/delete consistency | PASS |
| `export-delete` | complete export and deletion fixture | PASS |
| `delivery-fallback-receipts` | credentialed provider fixture | PASS locally; unavailable live |
| `verified-deposit` | Sociobot verification fixture | PASS locally; unavailable live |
| `no-double-booking` | occupied-slot conflict | PASS |
| `automatic-recovery` | durable consent-gated idempotent job | PASS locally; delivery unavailable live |
| `automatic-recovery-delay` | exact 15-minute schedule | PASS |
| `automatic-reminder` | once-only reminder fixture | PASS locally; delivery unavailable live |
| `practice-data-inventory` | exported record inventory | PASS |
| `card-data-excluded` | browser form/request inspection | PASS |
| `delivery-connection-test` | no-client-data provider fixture | PASS locally; provider unavailable live |
| `practice-plan-price` | $29 price and checkout link | PASS |
| `server-owned-integration-boundary` | no credentials serialized | PASS |
| `entra-token-validation` | discovery/JWKS/claims validation | PASS |

The manifest covers the material claims found on the live landing, privacy,
terms, setup, and README surfaces. The production contradictions above remain
blocking even though their isolated fixture tests pass.

### First-read gate — PASS

A fresh 1440×900 browser opened `/` with no state. The first viewport says:

- What: **“Recover unfinished paid-session bookings.”**
- For whom: solo coaches, tutors, and consultants whose paid booking stops.
- First click: **“Try it with sample data.”**

Adjacent copy says that click opens three fictional bookings and can be reset.
One click opened the populated demo. The persistent banner says “Demo — sample
data, nothing is saved” and includes **Reset demo** and **Start for real**.

## Clean local verification

The checkout began at the exact candidate with no tracked changes. Generated
verification artifacts were the only working-tree additions.

| Check | Fresh result |
| --- | --- |
| `npm ci` | PASS — 64 packages, 0 vulnerabilities |
| `npm test` | PASS — 11/11 Vitest tests |
| `npm run check:backend` | PASS — rustfmt and 34/34 Rust tests |
| `npm run test:deployment` | PASS — one replica, SQLite under `/data`, fleet-deployer boundary |
| `npm run test:e2e` | PASS — 28/28 Chromium tests |
| `VITE_BUILD_SHA=e800fff… npm run build` | PASS — strict TypeScript and production Vite build |
| `npm run check:size` | PASS — JS 79,444 B gzip; CSS 22,252 B raw |
| `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings` | PASS |
| candidate-stamped `cargo build --release --locked` | PASS |
| release binary with only `PORT` and process `PATH` | PASS — generated `/data` defaults, served health, clean shutdown |

The two fonts total 70,616 bytes, below the 120 KB budget. There is no raster
hero; the hand-made product rail is SVG. Docker, Podman, and Buildah are not
installed in this verifier, so a second local image build was unavailable.
The exact candidate is nevertheless running live, and the web and release
backend builds were reproduced locally.

## Live identity and repaired topology

- `/health` returned 200 with build SHA
  `e800fff12a71d3e0867f16d13bbe9caf480eca11`.
- The footer showed the same full SHA.
- Candidate and live HTML SHA-256:
  `ed5e0ebdb51b7a27e182286e6e1989f57b0d199742d49e45b807479460d1d93b`.
- Candidate and live JavaScript SHA-256:
  `04b2ab699660e265eae9d65f30ee9fa9ec3ee28897d6a1be6e35c024a7857f9a`.
- Candidate and live CSS SHA-256:
  `1a980601b6b1504be6686eb0c91197a83acf0e59616a9f42b01c6dc624b7cc31`.

Fresh independent-connection probes confirm the previous deployment defect is
fixed:

- 40 simultaneous demo writes from one forwarded IP: exactly **12×201** and
  **28×429**; every 429 had `X-RateLimit-Limit: 12` and `Retry-After: 60`.
- 160 simultaneous demo reads: exactly **40×200** and **120×429**; every 429
  had `X-RateLimit-Limit: 40` and `Retry-After: 1`.
- A separate general API probe sent 41 simultaneous integration-status reads:
  **40×200**, then **1×429** with `Retry-After: 1`.
- After reset, **24/24** fresh-connection reads with the old demo token returned
  404.

Observed allowance: general/read endpoints allow a burst of 40 per forwarded
client IP and write endpoints allow 12; limited responses include
`Retry-After`. Source inspection confirms every API route is under the global
limiter, with the stricter nested limiter on writes; only `/health` is exempt.

## Functional, boundary, and recovery checks

- The live sample opened Maya Patel, Jordan Lee, and Alex Morgan.
- Maya's permitted action produced one timestamped “Delivered · simulated
  email” receipt. Repeating it was idempotent and retained one receipt.
- Jordan's action returned `409 consent_required`, explained that recovery
  stays stopped, and created no receipt.
- A malformed demo token returned 400 with a recovery instruction; an unknown
  attempt returned 404.
- Reset rotated the browser token, restored the sample, and revoked the old
  token globally.
- Setup boundaries were exercised in Chromium: practice name 1/2 characters,
  slug 2/3/41 characters and uppercase, duration 14/15/480/481, deposit
  -1/0/1,000,000/1,000,001, two/three-letter currency, and missing timezone.
  Invalid edges were blocked; boundary-valid values were accepted by the form.
- An unauthenticated owner request returned 401 with
  `WWW-Authenticate: Bearer` and a plain recovery instruction.
- Local integration tests additionally cover double booking, encrypted tenant
  isolation, export/delete, signed callback rejection, bounce deduplication,
  one SMS fallback, automatic scheduling, reminder idempotence, and restart
  persistence.

The real production booking/deposit/message path could not be exercised
because the service truthfully reports the two required integrations missing.
This is the P0 result, not a test-environment limitation.

## Accessibility, responsive behavior, and UX

Fresh Axe scans covered `/`, `/demo`, `/start`, `/app`,
`/app/settings/data`, `/privacy`, `/terms`, and a real 404 at both 1440 px and
390 px. Results:

- zero serious or critical findings;
- `lang="en"`, one `h1`, one `main`, and no image missing alt text on every
  route;
- no horizontal overflow and no visible interactive target under 44×44 px;
- keyboard-only skip, selection, and no-consent recovery worked;
- focus was a visible 3 px sky outline with 4 px offset;
- `prefers-reduced-motion: reduce` left no transition/animation over 120 ms;
- 200% mobile text retained all content without horizontal overflow;
- normal pages and actions had no console/page errors. The intentional 404
  produced only Chromium's expected failed-resource message.

The factory `verify-url.sh` passed in 582 ms with correct title, language,
landmarks, image alternatives, button names, and zero console errors. Visual
inspection at desktop and 390 px found a coherent product-specific twilight
rail design matching `.factory/design.md`.

## Privacy, auth, headers, caching, and performance

The complete fresh demo flow—including recovery and reset—requested only the
product origin. Local storage contained only `demo:workspace-token`, session
storage contained the MSAL version marker, and the product origin set no
cookie. No analytics, payment, messaging, AI, sign-in, or font-CDN request
occurred during the demo.

The sign-in action navigated only to
`sociobotcustomers.ciamlogin.com`, tenant
`35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
`25c704f4-465a-47af-80ab-2c489466b697`, redirect
`https://booking-recovery-loop.sociobot.in/auth/callback`, authorization-code
flow with PKCE `S256`, and scopes `openid profile email offline_access`. No
customer credential was available, so a complete authenticated return was not
attempted; the backend's full token validation matrix passed locally.

HTML/API responses include CSP with header-only `frame-ancestors 'none'`,
`nosniff`, `DENY`, strict-origin referrer policy, permissions policy, request
IDs, and gzip. HTTP redirects to HTTPS. HTML and 404s are `no-cache`; hashed
JS/CSS and fonts are `public, max-age=31536000, immutable`. Required routes,
robots, sitemap, favicon, touch icon, and social image are live. Unknown routes
return a designed 404.

Fresh mobile Lighthouse:

- Performance 100, Accessibility 100, Best Practices 100, SEO 100;
- FCP 1,292 ms, LCP 1,704 ms, TBT 26.5 ms, CLS 0;
- total transfer 156,703 bytes; long-cache audit 1.0.

This is not a PWA, library, or CLI, so service-worker update/offline reload and
clean-consumer package checks do not apply. It registers no service worker and
shows an explicit offline demo recovery state.

## Evidence

- Per-claim command transcript:
  `verification-artifacts/claims-verification-10.log`
- First-read screenshot: `verification-artifacts/live-first-read-desktop.png`
- Live topology/demo probe: `verification-artifacts/live/live-check.json`
- Multi-route Axe/keyboard/privacy/header/auth audit:
  `verification-artifacts/live-audit.json`
- Lighthouse: `verification-artifacts/lighthouse-live.json` and
  `verification-artifacts/lighthouse-summary.json`
- Factory URL smoke test: `verification-artifacts/verify-url/verify.json`
- Local command logs: `verification-artifacts/test-deployment.txt`,
  `test-e2e.txt`, `build-frontend.txt`, `build-backend-release.txt`,
  `clippy.txt`, and `check-size.txt`

No product code was changed during verification. No unrelated service,
database, setting, or secret was accessed.

## Required remediation

1. Register and enable the dedicated `booking-recovery-loop-deposit` product
   at the approved Sociobot billing boundary.
2. Provision the product's supported credentialed email/SMS relay and signed
   receipt callback configuration.
3. Using a real test practice, complete one public booking through hosted
   deposit verification, automatic recovery, reminder delivery, signed bounce,
   and exactly one consented SMS fallback; retain receipts for each stage.
4. Confirm the Entra callback registration with a real test account and verify
   workspace persistence, export, and deletion across a fresh sign-in.
