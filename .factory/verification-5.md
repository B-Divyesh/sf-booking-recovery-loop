# Independent product verification 5 — FAIL

**Candidate:** `3e0256e1a0d72dcd315731554ad072122eca56b6`

**Live URL:** https://booking-recovery-loop.sociobot.in

**Verified:** 2026-08-29 UTC

**Work order:** `booking-recovery-loop-verify-5`

## Verdict

**FAIL — do not release this candidate.**

The frontend, isolated demo, build, accessibility, and performance are strong.
The deployed real-practice workflow is not reliable: one newly created practice
was visible on only one of three apparent live stores. The same public URL
returned `200` 10 times and `404` 20 times in 30 requests. Immediate owner
read and deletion also hit a different store. The paid subscription is not
available, DELETE is not rate limited, and the payment and messaging paths are
generic webhook adapters rather than usable Stripe and email/SMS integrations.

## Mandatory first-read gate — PASS

A cold 1440×900 visit showed, in the first viewport:

- **What:** “Recover unfinished paid-session bookings.”
- **For whom:** “For solo coaches, tutors, and consultants who need to act when
  a paid booking stops.”
- **First action:** **Try it with sample data**, beside “See three fictional
  bookings, then reset the sample at any time.”
- **Plain facts:** demo needs no account, sends no messages, and opens no
  payment.

The action enters the seeded demo in one click. `/demo` and `/?demo=1` show
three fictional bookings plus the persistent “Demo — sample data, nothing is
saved” banner, **Reset demo**, and **Start for real**.

Evidence: `verification-evidence-5/live-first-read-desktop.png`.

## Release-blocking findings

### Critical — live customer data is split across independent SQLite stores

A fresh real-practice API flow returned:

1. create practice: `201`;
2. create booking: `201`;
3. immediate owner read with the issued bearer key: `401`;
4. immediate delete with the same key: `404`;
5. next owner read with the same key: `200`, including the booking.

Thirty subsequent reads of that practice's public URL produced this stable
split: `200 × 10`, `404 × 20`. This is consistent with traffic reaching three
independent container-local databases. A user can therefore lose access,
receive a false deletion failure, or show clients a missing booking page based
only on which live instance answers.

The source explains the failure: the runtime defaults to
`sqlite://booking-recovery-loop.db` under `/data`, the encryption key is also
local under `/data/contact.key`, and `deploy/containerapp.m1.json` declares no
persistent shared volume. The venture plan explicitly required PostgreSQL
before real practice data, row-level tenant isolation, backups, and restore
evidence. Those requirements are not implemented.

Evidence: `verification-evidence-5/live-api-probes.json` and
`backend/src/main.rs`.

### Critical — the contracted paid recovery product is not end to end

- `GET https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout`
  returns `404 {"error":"enabled factory product","status":404}`. The live UI
  says “Checkout is not available yet.” A stranger cannot buy the promised
  $29/month product.
- Session payment is only a practice-entered static HTTPS URL. There is no
  Stripe Checkout-session creation, attempt/reference binding, or Stripe
  signature verification. The provider callback accepts a product-specific
  shared header token instead of a Stripe webhook signature.
- Email/SMS delivery is only a POST to an arbitrary URL entered by the owner.
  There is no supported provider, credential flow, or non-developer setup.
  A solo tutor must build and operate an adapter before any automatic recovery,
  reminder, receipt, or SMS fallback can work.

This is not the brief's smallest useful product: a branded paid-session page,
Stripe deposit, reminder delivery receipt, and automatic SMS/email fallback.
The prior builder handoff already identifies billing registration and a
first-party provider connection as missing; fresh live evidence confirms both.

### High — mandatory API rate limiting is incomplete and not service-wide

- Demo writes passed the documented check: requests 1–12 returned `201`, and
  request 13 returned `429` with `Retry-After: 60`.
- `DELETE /api/v1/practice` is not covered by the write limiter. Forty-five
  requests from one forwarded client all reached the handler (`404 × 45`),
  returned no `Retry-After`, and advertised `X-RateLimit-Whitelisted: true`.
- A concurrent burst of 200 GET requests from one forwarded client produced
  `404 × 140` and `429 × 60`, despite an advertised burst limit of 40. The
  limited responses had `Retry-After: 0`. Per-instance buckets are not a
  service-wide client allowance in the observed live topology.

The source configures the stricter governor for `Method::POST` only, even
though the same router contains the DELETE endpoint. The backend contract says
every non-health endpoint must return `429` with `Retry-After` once one client
passes its allowance. This candidate does not.

Evidence: `verification-evidence-5/live-api-probes.json` and
`backend/src/main.rs`.

### High — user-controlled delivery URLs create a server-side request risk

Unauthenticated practice creation accepts any URL whose scheme is HTTPS and
whose host parses. The backend later makes server-side POSTs to that URL for
connection tests, client messages, reminders, and fallbacks. There is no host
allowlist, private/link-local address rejection, DNS rebinding control, or
redirect restriction. Any visitor can obtain an owner key and use the service
as a request source toward HTTPS services reachable from the container.

This was established by source inspection; no internal address was probed.

### High — production identity does not meet the venture/auth contract

The product stores multi-device customer contact data and is intended to sell a
subscription, so the attached auth contract calls for Sociobot Entra External
ID. There is no MSAL client, `/auth/callback`, discovery/JWKS validation,
membership model, or `oid` tenancy. A bearer owner key is stored in
`localStorage`, shown on screen, and must be manually shared. This is not a
one-to-five-person practice account and does not provide revocation or member
access control.

### High — an unlisted and incompletely tested live claim remains

The app says: “Recovery and reminders run automatically when their due time
arrives.” No `.factory/claims.json` entry asserts that a due reminder is sent.
`automatic-reminder` proves only that a paid event inserts one queued row. The
test does not run the scheduler for that reminder or observe provider delivery.
Under the claims contract, this unlisted/under-tested promise blocks release.

### Medium — production privacy disclosure omits provider transfer details

The demo request log is correctly same-origin. In production, however, the
backend sends decrypted client contact information to the owner-entered
delivery URL. `/privacy` lists stored data but does not plainly identify this
third-party transfer, the fields sent, or the third party's retention/control
boundary.

## Claims gate

`.factory/claims.json` exists and contains 23 well-formed entries.

The literal pre-install first pass ran every listed command before other repo
inspection. All 13 Rust commands passed; all 10 Playwright commands failed
because the clean clone had not yet installed `@playwright/test`. After the
documented `npm ci` prerequisite, every manifest command was rerun individually
and all **23/23 passed**. The substantive verdict does not depend on the
pre-install dependency failure.

The passing claim suite does not detect the live split-store defect because its
real-practice tests use one local SQLite process. It also does not establish a
usable billing, Stripe, or messaging integration.

## Local verification

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 62 packages, 0 reported vulnerabilities |
| `npm test` | PASS — 10 tests |
| `npm run check:backend` | PASS — rustfmt and 19 tests |
| `npm run test:deployment` | PASS — asserts config says one replica |
| `npm run test:e2e` | PASS — 27 Chromium tests |
| `npm run build` | PASS — strict TypeScript and `dist/` |
| `npm run check:size` | PASS — JS 12,457 B gzip; CSS 21,906 B raw |
| `npm run build:backend` | PASS — optimized Rust build |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| runtime with only `PORT=4191` | PASS — generated defaults; `/health` and `/` 200 |

No Docker, Podman, or Buildah executable is installed in this worker, so the
container image itself could not be rebuilt. Both stages' exact repository
build commands passed, and the Dockerfile was inspected for its build args,
non-root runtime, `/data` working directory, and port contract.

## Live deployment and UX verification

- `/health` returns candidate SHA
  `3e0256e1a0d72dcd315731554ad072122eca56b6`.
- A candidate build using that SHA matches live `index.html`, JS, and CSS
  byte-for-byte. See `verification-evidence-5/deployment-match.json`.
- The repository's complete `scripts/verify-live.mjs` passed once, including
  demo reset, a same-connection practice flow, export/delete, mobile layout,
  and 12/13 write limiting. The later independent cross-request probe exposes
  why that single-connection flow is insufficient.
- Factory `verify-url.sh` passed: HTTPS 200, title, `lang=en`, one h1, main,
  no missing image alternatives, no unnamed buttons, and no console errors.
- Desktop and 390 px live axe scans across `/`, `/demo`, `/start`, `/privacy`,
  and `/terms` found zero serious/critical violations and zero overflow.
- Keyboard-only demo recovery passed. The skip link receives first focus,
  Enter focuses `<main>`, Space selects Maya, and Enter runs recovery. Focus is
  a visible 3 px `#9CCBFF` outline.
- With `prefers-reduced-motion: reduce`, no element reported a material
  animation or transition.
- The complete live demo flow contacted only
  `https://booking-recovery-loop.sociobot.in` and stored only the
  `demo:workspace-token` key. No analytics, payment, messaging, sign-in, AI, or
  font-CDN request occurred.
- Normal and invalid API checks behaved correctly for short names, invalid
  slugs, duration/deposit boundaries, non-HTTPS payment URLs, missing consent,
  invalid consented email, and duplicate slots. Eight concurrent demo recovery
  requests produced eight 200 responses and no 5xx.
- Security headers include CSP with `frame-ancestors 'none'`, `nosniff`, DENY
  framing, strict-origin referrer policy, and a restrictive permissions policy.
- Hashed JS/CSS/fonts use one-year immutable caching. HTML uses `no-cache`.
- Font payload is 70,616 bytes. Social image is 46,268 bytes. Lighthouse
  transferred 138,789 bytes.
- Fresh mobile Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 1.22 s, LCP 1.52 s, CLS 0, TBT 0 ms.

This product is not a library, CLI, or PWA. Consumer-package and service-worker
checks are not applicable.

## Required release work

1. Move real practice/contact/job data and encryption-key management to a
   shared durable store with tested backup/restore; prove consistency across
   live instances.
2. Register and wire the $29/month Sociobot subscription and the required
   Sociobot Entra CIAM account flow.
3. Implement a real Stripe-hosted deposit integration with signed provider
   events, plus a supported email/SMS delivery path that a solo practice can
   configure without writing an adapter.
4. Apply a shared/client-correct limiter to every non-health endpoint,
   including DELETE, and return a meaningful positive `Retry-After`.
5. Constrain outbound webhook targets and redirects against SSRF.
6. Add a claim that executes a due reminder through delivery, or remove the
   automatic-reminder copy.
