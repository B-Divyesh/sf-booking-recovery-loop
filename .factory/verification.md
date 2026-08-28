# Independent product verification — FAIL

**Candidate:** `d03d83db200435a8582ea5fac676139abfb139cb`

**Live URL:** `https://booking-recovery-loop.sociobot.in`

**Verified:** 2026-08-28 UTC

**Work order:** `booking-recovery-loop-verify-1`

## Verdict

**FAIL — do not release this candidate as the product described in the brief.**

The deployed M1 sample is polished, accessible at ordinary zoom, isolated from
external services, and correctly identifies itself as the candidate commit.
It is nevertheless only a fictional demo. A real practice cannot create an
account, publish a paid-session page, collect a deposit, recover a real booking,
send a reminder or fallback, inspect a real delivery receipt, export/delete its
data, or buy the stated plan. The repository and live page explicitly defer
those requirements to M2. That misses the acceptance contract's smallest useful
product and the definition-of-done requirement to complete the real job rather
than only demonstrate it.

The candidate also has a reproducible recovery-write concurrency failure, a
cold claims-gate timeout, claims that are not registered in `claims.json`,
200%-text reflow and touch-target problems, missing production cache headers,
and an incorrect HTTP status for unknown routes.

## First-read gate

Result: **PASS**.

Cold first screen at `/`:

- What it does: “Recover paid sessions before they disappear.”
- For whom: “For solo coaches and tutors who need to see why a paid booking
  stopped and what can happen next.”
- What to click first: visible **Try it with sample data** link.
- Adjacent expectation: “Opens a safe workspace with three fictional clients.”
- One click opens `/demo`; no account or setup is required.

The first screen also presents three short facts: no account, no real messages,
and no payment in the demo. Evidence:
`verification-artifacts/live-cold-desktop.png`.

## Release-blocking findings

### Critical — the real job-to-be-done does not exist

Only an isolated, fixed-data demo is implemented. **Start for real** removes the
demo token and links to a landing-page section that says the paid plan is not
open. There is no usable production path for the brief's customer.

Missing from the candidate:

- practice onboarding and Sociobot Entra customer sign-in;
- branded public session/booking page;
- hosted deposit collection through the required billing boundary;
- real abandoned-booking detection;
- scheduled reminders and bounce detection;
- automatic consent-aware email/SMS fallback;
- real delivery receipts and recovery outcome tracking;
- encrypted customer/contact persistence, export, and deletion;
- the advertised `$29/month` subscription.

This is stated directly in the UI and README as M2 scope. A milestone boundary
does not override the supplied product acceptance contract.

### High — valid concurrent recovery requests return HTTP 500

Eight concurrent recoveries for the same consented sample booking, each with a
valid unique idempotency key:

- live deployment: 3 returned `200`, 5 returned `500 demo_unavailable`;
- one local candidate process with a fresh SQLite database: 1 returned `200`,
  7 returned `500 demo_unavailable`.

The server logs each failure as `500 Internal Server Error`. This is below the
rate-limit burst and is therefore not expected throttling. Concurrent retries
must resolve idempotently or with a specific conflict, not an internal error.

### High — the first exact claim command fails from the clean environment

The required first action was to run each command in `.factory/claims.json`.
The first exact command,
`npm run test:e2e -- --grep @claim:demo-isolated`, failed because Playwright's
120-second `webServer` timeout elapsed while the clean Rust build was still
compiling. It passed in 8.1 seconds when rerun after compilation. The other four
exact commands passed. A later warm full E2E run passed all 11 tests.

The observed behavior is good, but the clean-clone claim gate itself is not
reliable. The contract explicitly makes any failing claim command blocking.

### High — claims inventory is incomplete and one claim test is insufficient

Material claims outside `.factory/claims.json` include:

- README: a 256-random-bit token and 24-hour expiry;
- README/demo docs: per-IP limiting based on the first forwarded hop;
- privacy page: server storage for up to 24 hours and removal behavior;
- landing/demo copy: no account and no payment are needed;
- terms: abusive automation returns a retry time.

These have unit/integration coverage in places, but the claims contract requires
each user-facing claim to be registered with exactly one demo-entry test.

Additionally, `@claim:demo-isolated` says its sandbox inserts a real-practice
fixture and proves it cannot be read or changed. The tagged Playwright test does
not insert such a fixture or attempt a mutation; it only checks the demo token,
request headers, and absence of the words “Private Practice.” A separate Rust
test covers read isolation, but it is not the declared claim command and does
not cover mutation through the demo flow.

## Other findings

### Medium — 200% text enlargement breaks reflow

At a 390px viewport, normal text size has no horizontal overflow
(`scrollWidth === clientWidth === 390`). With text enlarged to 200%, document
width becomes 649px and only 12 of 13 visible controls remain inside the
viewport. The unbroken build SHA in the footer expands the layout. Evidence:
`verification-artifacts/live-mobile-200-text.png`.

### Medium — footer links miss the 44px touch-target baseline

At 390px, the visible **Privacy** and **Terms** footer links measure about
43x18px and 39x18px. At desktop they are about 43x22px and 39x22px. Other
primary controls meet the target size.

### Medium — static assets are not cacheable as required

Hashed JavaScript, CSS, and font responses have no `Cache-Control` header.
Lighthouse's long-cache audit scored 0.5 and identified four resources with a
zero cache lifetime (about 116 KB total avoidable repeat transfer). Immutable
hashed assets should receive a long-lived immutable policy.

### Low — unknown routes return HTTP 200

`/missing-page` renders a good product-native not-found view and title, but the
HTTP response is `200`, not `404`. The backend SPA fallback serves `index.html`
without changing the status.

### Low — handled consent rejection appears as a console error

The no-consent path correctly returns `409 consent_required`, preserves the
empty receipt state, and explains recovery. Chromium nevertheless records
`Failed to load resource: the server responded with a status of 409` as a
console error during this expected user flow. Cold page loads have no console
or page errors.

### Low — Docker base violates the supplied runtime contract

`backend/Dockerfile` uses `FROM rust:1.98-slim-bookworm`. The backend contract
requires the floating stable major (`rust:1-slim` or `rust:1-alpine`) and
explicitly forbids pinning a Rust minor. Docker/Podman was unavailable in this
container, so the image itself could not be rebuilt here; both constituent
release builds passed.

## Claims gate results

| Claim | Exact isolated command | Result |
| --- | --- | --- |
| `demo-isolated` | `npm run test:e2e -- --grep @claim:demo-isolated` | **FAIL cold:** server timeout at 120s; **PASS warm:** 1/1 |
| `demo-reset` | `npm run test:e2e -- --grep @claim:demo-reset` | PASS, 1/1 |
| `consent-gates-recovery` | `npm run test:e2e -- --grep @claim:consent-gates-recovery` | PASS, 1/1 |
| `demo-recovery-receipt` | `npm run test:e2e -- --grep @claim:demo-recovery-receipt` | PASS, 1/1 |
| `demo-no-external-requests` | `npm run test:e2e -- --grep @claim:demo-no-external-requests` | PASS, 1/1 |

The warm full run passed all five claim tests plus six route/accessibility tests.

## Local build and automated checks

Started from a clean checkout at the candidate SHA. `npm ci` installed 62
packages and reported zero vulnerabilities.

| Check | Result |
| --- | --- |
| `npm test` | PASS — 2 files, 9 tests |
| `npm run check:backend` | PASS — fmt plus 7 Rust tests |
| `npm run test:e2e` | PASS warm — 11/11 Chromium tests |
| `npm run build` | PASS — TypeScript and Vite; `dist/` produced |
| `npm run check:size` | PASS — JS 8,352 B gzip; CSS 18,736 B raw |
| `npm run build:backend` | PASS — optimized release build |
| runtime with only `PORT` | PASS — generated default SQLite DB, served `/health` and `/` |

There is no repository lint command beyond TypeScript checking and Rust fmt.
The exact container build could not run because this verification environment
has no Docker, Podman, or Buildah executable.

## Live deployment identity and route evidence

- `/health` returned `200` with
  `{"status":"ok","build_sha":"d03d83db200435a8582ea5fac676139abfb139cb"}`.
- Rebuilding with `VITE_BUILD_SHA` set to that SHA produced byte-identical live
  HTML and JS:
  - HTML SHA-256: `2b1505a55a6036e019eb0bda2b31f831ecbffb63e251100c6bf56ae0116f0f3e`
  - JS SHA-256: `e2ab4642d48c0d84ef56eab297ab14b87e8edfd8ff65215d6f434e645339a764`
- `/`, `/demo`, `/privacy`, `/terms`, `/404`, `/robots.txt`, and
  `/sitemap.xml` returned `200`.
- Every internal link found on the five rendered routes returned `200`.
- Each rendered route has `lang="en"`, exactly one `h1`, one `main`, and its
  route-specific title.

The earlier deployment-only concern is not present in this fresh check: the
deployment is reachable and matches the candidate.

## Live end-to-end and boundary evidence

- Fresh `/demo` created three realistic sample bookings and showed the
  persistent demo safety banner.
- Maya Patel: recovery returned `200`, changed state to recovered, and showed
  a timestamped “Delivered · simulated email” receipt.
- Jordan Lee: recovery returned `409 consent_required`; no receipt was added,
  including after reload.
- Alex Morgan: already-complete text was shown, an existing simulated receipt
  remained visible, and no recovery action was offered.
- Reset replaced the token and restored the original states.
- Missing/short idempotency key returned `400 idempotency_key_required`.
- Invalid token and unknown attempt returned `404 demo_not_found`.
- Desktop and 390px flows completed with no ordinary-size horizontal overflow.
- Keyboard-only use reached the skip link, moved focus to `main`, selected a
  ticket with Space, and ran recovery with Enter. Keyboard focus rendered a
  3px `#9CCBFF` outline with 4px offset.
- Reduced-motion media emulation matched, with transitions/animations reduced
  to 0.01ms and movement transforms removed.

## Privacy, headers, and rate limit

The complete live demo flow issued requests only to
`https://booking-recovery-loop.sociobot.in`. It contacted no payment,
messaging, sign-in, billing, analytics, font-CDN, or AI origin. The browser
stored only `demo:workspace-token` in local storage.

HTML and API responses included CSP, `X-Content-Type-Options: nosniff`,
`Referrer-Policy: strict-origin-when-cross-origin`, `X-Frame-Options: DENY`,
and a camera/microphone/geolocation-denying permissions policy.

A 24-request concurrent live POST burst from one fixed first
`X-Forwarded-For` address produced its first `429` on request 12 and included
`Retry-After: 0`. Because requests reached multiple replicas, 17 were accepted
and 7 were rejected during the full burst. Thus the documented per-process
write burst of 12 is enforced and the required header exists, although the
allowance is not globally coordinated across replicas.

## Accessibility and performance

- Live desktop and 390px demo: zero axe serious/critical violations.
- Local axe coverage: landing, demo, privacy, terms, and not-found all passed.
- Normal viewport: no horizontal overflow; visible focus and skip link work.
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.2s, LCP 1.6s, CLS 0, TBT 30ms, total 116 KiB.
- Bundle budgets pass: initial JS 8.39 KB gzip, CSS 18.74 KB raw, fonts 70.62 KB.
- Failures at 200% text and footer touch targets are described above.

Lighthouse raw evidence is in
`verification-artifacts/lighthouse-live.json`; desktop, mobile, and 200%-text
screenshots are beside it. This product is neither a PWA nor a library/CLI, so
offline/service-worker and clean-consumer package checks do not apply.

## Required next actions

1. Implement the actual paid booking/recovery product through the remaining
   venture milestones before seeking product acceptance.
2. Serialize or safely arbitrate recovery writes; add a claim-level concurrent
   retry test that never returns 500 and proves exactly-once effect.
3. Make every claims command reliable from a cold clone and register all
   material user-facing claims with adequate tests.
4. Fix 200% text reflow, footer touch targets, asset caching, unknown-route
   status, and the Rust base image tag.
