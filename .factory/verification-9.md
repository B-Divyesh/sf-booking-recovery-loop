# Verification 9 — FAIL

**Candidate:** `06c4b50fbc1f5b3eaae13b38bf2f11789e8d7d07`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

**Verified:** 2026-08-29 UTC

**Work order:** `booking-recovery-loop-verify-9`

## Decision

**FAIL — do not release.**

The candidate is deployed, polished, accessible, fast, and locally well tested.
It still cannot complete the brief's real paid-booking recovery job because the
dedicated booking-deposit product and delivery provider are unavailable.
Fresh multi-connection production probes also show that the claimed shared
database/rate-limit repair is not active: the three replicas still expose
separate demo workspaces and separate request allowances. This directly
falsifies two mandatory claims.

## Release-blocking findings

### P0 — production cannot take a client deposit or deliver recovery messages

Fresh production responses:

- `GET /api/v1/integrations/status` returned 200 with
  `billing.configured: false` for `booking-recovery-loop-deposit` and
  `delivery.configured: false`.
- Both GET and POST to the exact Sociobot endpoint
  `/api/v1/products/booking-recovery-loop-deposit/checkout` returned 404 with
  `{"error":"enabled factory product","status":404}`.
- The separate `$29/month` practice-subscription checkout returned 303 to
  `checkout.dodopayments.com`. It is not a client deposit.

The backend correctly fails closed, but that means a real customer cannot
complete a new paid booking. It also cannot send the automatic recovery,
reminder, delivery receipt, or email-bounce-to-SMS fallback required by the
researched smallest useful product. Local credentialed-provider fixtures do
not make the production workflow usable.

### P1 — the shared 12-write and 40-read allowances multiply by three

The repository's live script passed when it reused one pooled browser request
context: 12 writes succeeded and the 13th returned 429. That connection was
sticky to one replica and did not test the deployed topology.

A fresh independent probe opened one request context per request so ingress
could distribute traffic across replicas:

| One forwarded client | Observed | Required |
| --- | --- | --- |
| 40 concurrent writes in 373 ms | **36×201**, 4×429 | 12 accepted, 28 limited |
| 160 concurrent reads in 704 ms | **120×200**, 40×429 | 40 accepted, 120 limited |

Every observed 429 did include `Retry-After` (`60` for writes, `1` for reads)
and the advertised limit (`12` or `40`). The effective allowance is nevertheless
one full burst per each of three replicas. This falsifies claim
`forwarded-rate-limit` and violates the backend-wide rate-limit contract.

Raw summarized evidence is in
[`verification-9-evidence/topology-probes.json`](verification-9-evidence/topology-probes.json).

### P1 — Reset demo leaves the old token usable on other replicas

The same separate-connection test exposed three different workspace IDs for
one portable demo token. After `POST /api/v1/demo/reset` returned 200 with a new
token, 36 requests using the old token produced:

- **24×200**, exposing two still-live old workspace IDs;
- 12×404, from the replica that handled reset.

The live privacy page promises that reset makes the current workspace
inaccessible. Claim `demo-reset` requires every old-token read to return 404.
Production does neither, so this is both a privacy defect and a false deployed
claim. The passing supplied script again stayed on one pooled connection and
therefore did not exercise multiple replicas.

### Release gate — the prescribed pre-install claim invocation is not runnable

`.factory/claims.json` exists with 26 entries. As required, every exact command
was invoked before any setup action. All 17 Rust commands passed. All nine
Playwright commands failed before collecting a test because the clean clone had
no installed `@playwright/test` package (`ERR_MODULE_NOT_FOUND`). After the
required `npm ci`, all nine were rerun individually and passed. All 26 claim
tests therefore pass in an installed checkout, but the literal first clean-clone
claim gate recorded failures. The work order says any failing claim invocation
is release-blocking; the substantive production claim failures above are
independent of this setup issue.

## First-read gate — PASS

A fresh 1440×1000 browser context opened `/` without prior state. The first
screen plainly answers all three questions:

- What: **“Recover unfinished paid-session bookings.”**
- For whom: solo coaches, tutors, and consultants whose paid booking stops.
- First click: **“Try it with sample data.”**

Adjacent copy says it opens three fictional bookings and can be reset. One
click opened the populated demo without an account or payment. The persistent
banner says “Demo — sample data, nothing is saved” and provides **Reset demo**
and **Start for real**. Evidence is in the live desktop/mobile screenshots under
`.factory/verification-9-evidence/live/`.

## Claims results

After `npm ci`, every manifest command passed in isolation:

| Claim | Result |
| --- | --- |
| `demo-isolated` | PASS — 1 Rust test |
| `demo-lifetime` | PASS — 1 Rust test |
| `forwarded-rate-limit` | PASS locally; **FAIL live multi-replica** |
| `demo-no-account-payment` | PASS — 1 Playwright test |
| `demo-reset` | PASS locally; **FAIL live multi-replica** |
| `consent-gates-recovery` | PASS — 1 Playwright test |
| `demo-recovery-receipt` | PASS — 1 Playwright test |
| `demo-no-external-requests` | PASS — 1 Playwright test |
| `sample-three-bookings` | PASS — 1 Playwright test |
| `practice-publish` | PASS — 1 Playwright fixture test |
| `booking-consent-record` | PASS — 1 Playwright fixture test |
| `encrypted-tenant-data` | PASS — 1 Rust test |
| `shared-practice-storage` | PASS — 1 Rust test |
| `export-delete` | PASS — 1 Rust test |
| `delivery-fallback-receipts` | PASS — 1 credentialed fixture test; unavailable live |
| `verified-deposit` | PASS — 1 billing fixture test; unavailable live |
| `no-double-booking` | PASS — 1 Rust test |
| `automatic-recovery` | PASS — 1 Rust test |
| `automatic-recovery-delay` | PASS — 1 Rust test |
| `automatic-reminder` | PASS — 1 Rust test |
| `practice-data-inventory` | PASS — 1 Rust test |
| `card-data-excluded` | PASS — 1 Playwright test |
| `delivery-connection-test` | PASS — 1 Rust test |
| `practice-plan-price` | PASS — 1 Playwright test |
| `server-owned-integration-boundary` | PASS — 1 Rust test |
| `entra-token-validation` | PASS — 1 Rust test |

The manifest now covers the material public claims found on the landing,
privacy, terms, setup, and README surfaces. The two live contradictions above
are the blocking claim findings.

## Clean local verification

The checkout began clean and exactly at the candidate. `npm ci` installed 64
packages and reported zero vulnerabilities.

| Check | Result |
| --- | --- |
| `npm test` | PASS — 10 tests |
| `npm run check:backend` | PASS — rustfmt and 29 Rust tests |
| `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings` | PASS |
| `npm run test:deployment` | PASS |
| `npm run test:e2e` | PASS — 28 Chromium tests |
| `VITE_BUILD_SHA=06c4… npm run build` | PASS — exact production frontend build |
| `npm run check:size` | PASS — JS 79,446 B gzip; CSS 22,252 B raw |
| locked candidate-stamped Rust release build | PASS |
| runtime with only `PORT` plus process `PATH` | PASS — generated-default DB, persisted key, health and static app served |

The no-config release binary returned the candidate SHA from `/health`, logged
configuration sources without secret values, and shut down cleanly. Docker,
Podman, and Buildah are unavailable in this verifier, so the image could not be
rebuilt. Inspection confirms the required multi-stage, floating `rust:1-slim`,
non-root, build-argument, and port contracts.

## Live deployment identity — PASS

- `/health` returned 200 and build SHA
  `06c4b50fbc1f5b3eaae13b38bf2f11789e8d7d07`.
- The footer showed the same full SHA.
- Candidate-stamped local and live HTML SHA-256 both equal
  `cedf3f59b29a9269beefaee0a6ed9ded49ca8b587ee3fbe6ff99a71aa54e8683`.
- Local and live JavaScript SHA-256 both equal
  `cad4565df196807304ad23c4de0b4e8881a03533fd532cce3faff45eaf59d8f6`.
- Local and live CSS SHA-256 both equal
  `1a980601b6b1504be6686eb0c91197a83acf0e59616a9f42b01c6dc624b7cc31`.

The live deployment matches this candidate. The result is not a stale-release
or deployment-only failure.

## Functional and recovery-path evidence

- The live one-click demo opened exactly Maya Patel, Jordan Lee, and Alex
  Morgan.
- Maya's permitted follow-up produced a timestamped “Delivered · simulated
  email” receipt and recovered outcome.
- Jordan's missing consent stopped recovery, showed the reason, produced no
  receipt, and remained stopped after reload.
- Reset rotated the browser token and restored the visible seed on the serving
  replica. Cross-replica revocation fails as documented above.
- Setup form boundaries behaved correctly: practice name 1/2 characters,
  slug 2/3 characters, duration 14/15/480/481 minutes, deposit
  -1/0/1,000,000/1,000,001 minor units, and two/three-letter currency input.
- An unauthenticated owner request returned 401 with
  `WWW-Authenticate: Bearer`; the production-only `X-Test-Oid` header did not
  bypass authentication.
- The real deposit/delivery path could not be completed because both production
  integrations are unavailable.

## Sign-in boundary — PASS as far as unauthenticated verification permits

Clicking **Sign in** made discovery and authorization requests only through
`sociobotcustomers.ciamlogin.com`. The authorization request used:

- tenant `35c6fe40-0ec0-46b6-98c6-213ad4de6650`;
- client `25c704f4-465a-47af-80ab-2c489466b697`;
- redirect `https://booking-recovery-loop.sociobot.in/auth/callback`;
- authorization code flow with PKCE `S256`;
- scopes `openid profile email offline_access`.

No test customer credentials were supplied, so token return and a real
multi-device owner session could not be completed. The production test-identity
header is correctly disabled.

## Accessibility, mobile, and visual review — PASS

Fresh Playwright axe scans on `/`, `/demo`, `/privacy`, `/terms`, `/start`,
`/app`, `/app/settings/data`, and a real 404 found zero serious or critical
issues at both 1440px and 390px. Every route had `lang="en"`, one `h1`, one
`main`, and no image missing alt text.

At 390×844 there was no horizontal overflow and no visible interactive target
below 44×44 CSS pixels. The same held with root text enlarged to 200%. Keyboard
focus began on the skip link with a visible 3px sky-blue outline and 4px offset;
Enter moved focus to main, and the demo recovery was keyboard-operable. Reduced
motion reduced transitions/animations to 0.01 ms with no movement transform.
Valid routes and demo actions emitted no console/page errors. The intentional
404 navigation produced only Chromium's expected failed-resource 404 message.

The implemented twilight appointment-rail visual system matches
`.factory/design.md`, uses self-hosted fonts and original hand-made artwork,
and is distinct rather than a generic template.

## Privacy, headers, caching, links, and performance

The complete fresh live demo and recovery flow requested only the product
origin: HTML, two self-hosted fonts, hashed JS/CSS, workspace creation, and
sample recovery. It contacted no payment, messaging, sign-in, billing, AI,
analytics, or font-CDN origin. Local storage contained only
`demo:workspace-token`; session storage contained the MSAL version marker; no
cookie was set by the product origin.

HTML/API responses include CSP with header-only `frame-ancestors 'none'`,
`nosniff`, `DENY`, strict-origin referrer policy, permissions policy, request
IDs, and gzip compression. HTTP redirects to HTTPS. HTML and 404s are
`no-cache`; hashed JS/CSS and fonts are
`public, max-age=31536000, immutable`. All expected internal links returned
200; the deliberate missing route returned 404; the practice subscription link
returned the expected 303. Robots, sitemap, favicon, touch icon, and social
image are live.

Fresh mobile Lighthouse evidence:

- Performance 100, Accessibility 100, Best Practices 100, SEO 100;
- FCP 1,211 ms, LCP 1,663 ms, TBT 0 ms, CLS 0;
- total transfer 156,711 bytes; long-cache audit 1.0.

This product is not a PWA, library, or CLI. Service-worker update/offline reload
and clean-consumer package checks do not apply; the explicit offline error state
does pass locally.

## Required remediation

1. Register and verify the dedicated variable-amount
   `booking-recovery-loop-deposit` product, then complete a real client booking
   and verified deposit through Sociobot/Dodo.
2. Provision the approved credentialed email/SMS relay and prove live recovery,
   reminder, delivery receipt, signed bounce, and single SMS fallback.
3. Make every live replica use one genuinely shared production database and
   shared encryption configuration. Re-test with separate TCP connections, not
   one pooled/sticky browser context.
4. Enforce 12 writes/minute and 40 reads/second globally by first forwarded
   client IP; verify exact counts and `Retry-After` across all replicas.
5. Revoke an old demo token globally so every post-reset read returns 404.
6. Make the first prescribed claim run executable from the stated clean-clone
   state, or make dependency installation an explicit prerequisite to that
   gate.

No product code was changed during this verification.
