# Independent product verification — FAIL

**Candidate:** `b6ca2c781ddd603ff08c582b66f4b1970df783d4`
**Live URL:** `https://booking-recovery-loop.sociobot.in`
**Verified:** 2026-08-28 UTC
**Work order:** `booking-recovery-loop-verify-3`

## Verdict

**FAIL — do not release this candidate as Booking Recovery Loop.** The deployment
is healthy, accessible, private in demo mode, and runs the candidate. All
declared claim tests and complete local checks passed. It remains an isolated,
fictional M1 sample, not the real paid-booking recovery product in the brief.

## First-read gate — PASS

On a cold desktop load the first screen answered all required questions in
plain words: it says **“Recover paid sessions before they disappear”**, names
**“solo coaches and tutors”**, and has a visible **Try it with sample data**
link with the explanation **“Opens a safe workspace with three fictional
clients.”** The one-click link opens `/demo`; the screen also says no account,
no real messages, and no payment in the demo.

## Release-blocking findings

### Critical — the promised real product is not implemented

The live UI says “The paid plan is not open in M1. Accounts and hosted checkout
arrive in M2.” `Start for real` only removes the demo token and returns to that
landing section. The README and `.factory/plan.md` confirm M2–M6 are still
planned. A solo tutor, coach, or consultant therefore cannot:

- create a practice account or sign in using Sociobot Entra External ID;
- publish a branded session page, take a Stripe-hosted deposit, or detect a
  real abandoned booking;
- send/schedule a real consent-aware reminder or email/SMS fallback, receive
  provider delivery/bounce evidence, or track a real outcome;
- store required encrypted contact data, export/delete it, or buy the advertised
  `$29/month` practice plan.

The sole successful end-to-end flow is a server-seeded sample whose email is
explicitly in-process and simulated. A milestone boundary cannot override the
brief or repository definition of done.

### High — live rate limiting misses the published 12-write allowance

`.factory/demo.md` says writes allow a burst of 12 and then return `429` with
`Retry-After`. From one fresh client identity (`X-Forwarded-For:
198.51.100.250`), 36 sequential `POST /api/v1/demo/workspaces` returned `201`
before the first `429` at request 37. Across 96 requests, 59 were accepted and
37 were limited; sampled 429s contained `Retry-After: 0`.

The local route test rejects the thirteenth write, but production appears to
distribute a client across independent per-replica buckets. The published
per-client 12-write allowance is not enforced live, violating the rate-limit
claim and backend contract.

## Claims gate — PASS

After clean `npm ci` (62 packages, 0 reported vulnerabilities), every exact
`.factory/claims.json` command passed:

| Claim | Command | Result |
| --- | --- | --- |
| `demo-isolated` | `cargo test --manifest-path backend/Cargo.toml demo_never_reads_or_mutates_real_practice_fixture` | PASS (1) |
| `demo-lifetime` | `cargo test --manifest-path backend/Cargo.toml portable_token_has_256_random_bits_and_24_hour_expiry` | PASS (1) |
| `forwarded-rate-limit` | `cargo test --manifest-path backend/Cargo.toml write_limit_uses_forwarded_ip_and_returns_retry_after` | PASS (1) |
| `demo-no-account-payment` | `npm run test:e2e -- --grep @claim:demo-no-account-payment` | PASS (1) |
| `demo-reset` | `npm run test:e2e -- --grep @claim:demo-reset` | PASS (1) |
| `consent-gates-recovery` | `npm run test:e2e -- --grep @claim:consent-gates-recovery` | PASS (1) |
| `demo-recovery-receipt` | `npm run test:e2e -- --grep @claim:demo-recovery-receipt` | PASS (1) |
| `demo-no-external-requests` | `npm run test:e2e -- --grep @claim:demo-no-external-requests` | PASS (1) |

## Local quality gates — PASS

| Check | Result |
| --- | --- |
| `npm test` | PASS — 2 files, 9 tests |
| `npm run check:backend` | PASS — fmt and 9 Rust tests |
| `npm run test:e2e` | PASS — 17 Chromium tests |
| `npm run build` | PASS — `dist/` produced |
| `npm run check:size` | PASS — JS 8,392 B gzip; CSS 19,123 B raw |
| `npm run build:backend` | PASS — optimized Rust release build |
| Runtime with only `PORT=4191` | PASS — `/health` and `/` both 200; generated default database |

Docker/Podman/Buildah are unavailable in this worker, so the exact container
build was not executable here. Both Dockerfiles were reviewed: multi-stage,
`rust:1-slim`, `BUILD_SHA`, and distroless non-root runtime. This is a coverage
limitation, not a substitute for image verification in a Docker-capable worker.

## Deployment identity and live checks

- `/health` returned 200 and build SHA `b6ca2c781ddd603ff08c582b66f4b1970df783d4`.
- A rebuild with `VITE_BUILD_SHA` set to that SHA was byte-identical to live:
  HTML SHA-256 `6e13e71e0de20c4935861aab0318a2eb05905f9b1acd08d1ad2b26108fe45f6b`;
  app JS SHA-256 `e0e74692f85b391676c3884dd2007beacfa6f878081da6c87046756f3502956b`.
- `/`, `/demo`, `/privacy`, `/terms`, `/robots.txt`, and `/sitemap.xml` return
  200; an unknown page returns 404.
- Fresh live demo: three sample bookings load; Maya produces a timestamped
  **Delivered · simulated email** receipt; Jordan is stopped for missing email
  consent; reset creates a new token and restores seed data.
- Eight concurrent valid Maya recoveries all returned 200. A missing
  idempotency key returned 400; invalid sample token returned 404.

## Privacy, accessibility, responsive, and performance evidence

- The complete live demo request log (load, recover, consent stop, reset) had
  only `https://booking-recovery-loop.sociobot.in`; no payment, messaging,
  sign-in, billing, analytics, font-CDN, or AI origin. Storage contained only
  `demo:workspace-token`.
- HTML/API responses include CSP with `frame-ancestors 'none'`, `nosniff`,
  strict-origin referrer policy, frame denial, and camera/microphone/geolocation
  denial. Hashed JS has one-year immutable caching.
- Desktop cold load and 390px demo had no console/page errors. Live axe found
  zero serious/critical issues. Keyboard Tab focuses the skip link with a 3px
  visible outline. At 390px normal and 200%-text pages had no overflow and
  footer links were at least 44x44px. Reduced-motion media matched.
- Initial JS is 8.42 KB gzip, CSS 19.12 KB raw, and bundled fonts total 70.6 KB.

This is neither a PWA nor a library/CLI, so service-worker update/offline
reload and clean-consumer package checks do not apply. No AI feature is needed
for this deterministic workflow. No sign-in exists to validate against CIAM;
that absence is part of the critical scope finding.
