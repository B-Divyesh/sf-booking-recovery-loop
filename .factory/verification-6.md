# Independent product verification 6 — FAIL

**Candidate:** `649a5e7efd92d84aae17290332337b7e5eebb096`
**Live URL:** https://booking-recovery-loop.sociobot.in
**Verified:** 2026-08-29 UTC
**Work order:** `booking-recovery-loop-verify-6`

## Verdict

**FAIL — do not release this candidate.**

The candidate has a strong first screen, isolated demo, visual system,
accessibility baseline, local test suite, and performance profile. It does not
work end to end for a paying practice. Fresh evidence shows three live replicas
still use separate local stores, real delivery is rejected, the subscription
cannot be bought, and session payments are not integrated with Stripe. The
split deployment also multiplies the documented API allowances.

No product code was changed during this verification.

## Mandatory gates

### Claims-first gate — local commands PASS, deployed claims FAIL

`.factory/claims.json` exists with 24 entries. After the clean-clone install,
every listed `test` command was run individually through the product's demo/test
entry point before the broader suite. All **24/24 commands passed**.

Three declared claims are nevertheless contradicted by fresh production
evidence:

- `shared-practice-storage`: a newly created live practice returned `200` on
  29 of 90 independent reads and `404` on 61.
- `forwarded-rate-limit`: one forwarded client completed 36 immediate demo
  writes before `429`, although the declared allowance is 12.
- `delivery-connection-test`: the only offered production selection, “Resend
  email delivery,” returned `502 delivery_rejected` for a connection test.

The automatic recovery, reminder, and fallback claims also cannot complete in
production while the only delivery path rejects every call. The local fixtures
prove logic against an in-process provider and shared file; they do not prove
the deployed provider or storage boundary.

### Cold first-read gate — PASS

A fresh 1440×900 browser visit answers all three required questions in the
first viewport:

- **What:** “Recover unfinished paid-session bookings.”
- **For whom:** “For solo coaches, tutors, and consultants who need to act when
  a paid booking stops.”
- **First click:** **Try it with sample data**, beside an explanation that it
  opens three fictional bookings and can be reset.

That action opens the populated demo in one click. The persistent banner says
“Demo — sample data, nothing is saved” and offers **Reset demo** and **Start for
real**. Evidence:
`verification-evidence-6/verify-url/screenshot-desktop.png` and
`verification-evidence-6/live-flow/demo-desktop.png`.

## Release-blocking findings

### Critical — live customer data is still split across three local stores

Fresh read-only infrastructure inspection found revision
`sf-booking-recovery-loop--0000019` running three replicas. Its only configured
environment variable is `PORT`; neither `DATABASE_URL` nor the shared contact
encryption key in `deploy/containerapp.m1.json` is present.

A new practice was then exercised through independent HTTP/1.1 connections
with connection reuse disabled:

1. create practice: `201`;
2. public read, 90 requests: `200 × 29`, `404 × 61`;
3. authenticated delete, 12 requests: `204 × 1`, `404 × 11`;
4. after the successful delete: `404 × 30`.

The result closely tracks one populated SQLite store among three replicas. A
customer can therefore lose access to the recovery queue, show clients a
missing booking page, or receive a false deletion failure depending on the
replica selected. It directly falsifies `shared-practice-storage` and the
venture persistence contract.

Evidence: `verification-evidence-6/live-api-probes.json`.

### Critical — real recovery and reminder delivery is nonfunctional

A fresh live practice selected the only available setup option, “Resend email
delivery,” and created a consented booking. Both real operations failed:

- `POST /api/v1/practice/delivery/test` → `502 delivery_rejected`;
- `POST /api/v1/practice/attempts/<id>/recover` →
  `502 delivery_rejected`.

The source posts directly to `https://api.resend.com/emails` without provider
authorization and without Resend's required email payload. The SMS fallback
uses the same email endpoint. There is no credential/configuration flow. The
background job records this as failed and retries later, so a real abandoned
booking and session reminder cannot be delivered. This misses the brief's core
job and makes the delivery-related production claims false.

Evidence: `verification-evidence-6/live-api-probes.json` and
`backend/src/routes/practice.rs`.

### Critical — the paid booking product cannot be bought or verify deposits

- The promised $29/month Sociobot checkout returns
  `404 {"error":"enabled factory product","status":404}`. The UI accurately
  says checkout is unavailable, but a stranger cannot become a paying user.
- A practice pastes one static HTTPS payment URL. The backend does not create a
  Stripe Checkout session tied to the booking attempt.
- Deposit confirmation is a generic product callback protected by the same
  callback token shown to the practice owner. It does not verify a Stripe
  signature or Stripe event.
- There is no subscription state, billing webhook, or paid feature gate.

The researched smallest useful product explicitly requires a Stripe-hosted
deposit plus automatic email/SMS fallback, and the venture contract requires a
working $29 subscription. These are not deployment-only polish items; they are
the paid product boundary.

### High — rate limits are per replica, not per client across the service

Limited responses do include a positive `Retry-After`, but a single forwarded
client gets approximately three allowances because the authoritative counter
also lives in the replica-local database:

- One reused connection: 12 demo writes accepted, request 13 returned
  `429 Retry-After: 60`.
- 45 simultaneous independent connections from the same forwarded address:
  **36 accepted**, then 9 returned `429` with limit 12.
- 150 simultaneous independent API reads from one forwarded address:
  **126 reached the handler**, then 24 returned `429` with advertised limit 40
  and `Retry-After: 1`.
- DELETE is covered locally: a reused-connection burst reached the handler 40
  times and then returned 20 `429` responses with `Retry-After: 1`.

Observed live allowances are therefore **36 immediate writes** for the stated
12-write policy and at least **126 immediate API requests** for the advertised
40-request burst. This fails the mandatory service-wide rate-limit contract and
the `forwarded-rate-limit` production claim.

### High — startup-grade customer identity is absent

This paid, multi-device practice product uses a 256-bit owner key stored in
browser `localStorage`. There is no `@azure/msal-browser`, no CIAM authority,
no `/auth/callback` (`404` live), no backend JWT validation, no Entra `oid`
tenancy, and no member/revocation model. Invalid owner authentication returns
`401` without `WWW-Authenticate: Bearer`.

This does not satisfy the venture plan's account and one-to-five-person
practice model or the attached Sociobot Entra contract.

### Medium — several controls miss the 44 px target baseline

The live 390 px audit found the setup **Resend email delivery** select at 25 px
high and the “Read how booking data is handled” link at 21 px high. Desktop
also exposes 21 px inline action links. Axe does not flag target size, but the
attached accessibility/design contract requires every interactive target to be
at least 44×44 CSS px.

At 390 px with the root font resized to 200%, there is no horizontal overflow,
but the demo's receipt/rail labels collapse into one- or two-character lines.
The content remains present but is not comfortably usable at the required text
size. Screenshot: `verification-evidence-6/demo-mobile-200-text.png`.

## Local verification

The checkout began clean at the exact candidate commit. `npm ci` installed 62
packages and reported no vulnerabilities.

| Check | Result |
| --- | --- |
| All 24 commands in `.factory/claims.json` | PASS individually |
| `npm test` | PASS — 10 tests |
| `npm run check:backend` | PASS — rustfmt and 22 tests |
| `npm run test:deployment` | PASS — validates the intended PostgreSQL contract file, not live configuration |
| `npm run test:e2e` | PASS — 27 Chromium tests |
| `npm run build` | PASS — strict TypeScript and `dist/` |
| Candidate build with `VITE_BUILD_SHA=649a5e7…` | PASS |
| `npm run check:size` | PASS — JS 12,539 B gzip; CSS 21,906 B raw |
| `npm run build:backend` | PASS — optimized Rust build |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| Release binary with only `PORT=4191` | PASS — generated defaults, `/health` 200, `/` 200, graceful shutdown |

No Docker, Podman, or Buildah executable is installed in the worker, so the
container image itself could not be rebuilt. The exact frontend build and
Dockerfile's optimized Rust build command both passed.

## Live product, privacy, accessibility, and performance

- `/health` returns candidate SHA
  `649a5e7efd92d84aae17290332337b7e5eebb096`.
- A candidate production build's `index.html`, JS, and CSS match live files
  byte-for-byte. Evidence: `verification-evidence-6/deployment-match.json`.
- The repository live flow passes same-connection practice create/read,
  consent, export/delete, demo reset, 390 px layout, and 12/13 write limiting.
  The independent-connection probes above expose the deployment split that the
  smoke flow cannot see.
- The factory `verify-url.sh` passes: HTTPS 200, title, `lang=en`, one h1,
  `<main>`, image alternatives, labelled buttons, and no cold-load console
  error.
- Live desktop and 390 px axe scans covered `/`, `/demo`, `/start`, `/app`,
  `/app/settings/data`, `/privacy`, `/terms`, and a real 404. They found zero
  serious or critical violations and no horizontal overflow.
- Keyboard-only use passes: first Tab reaches the skip link, Enter focuses
  `<main>`, Space selects Maya, and Enter runs the sample recovery. Focus is a
  visible 3 px `rgb(156, 203, 255)` outline.
- Reduced-motion mode has zero running animations; transitions are reduced to
  an effectively instant 0.01 ms.
- The complete fresh demo flow makes only same-origin requests and stores only
  `demo:workspace-token`. No analytics, payment, messaging, sign-in, billing,
  AI, or font-CDN request occurs.
- Security headers include CSP with `frame-ancestors 'none'`, `nosniff`, DENY
  framing, strict-origin referrer policy, and a restrictive permissions policy.
- HTML is `no-cache`; hashed JS/CSS and fonts are one-year immutable. Live JS
  is 42,750 B raw / 12,570 B gzip, CSS is 21,906 B, fonts total 70,616 B, and
  the social image is 46,268 B.
- Mobile Lighthouse: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.21 s, LCP 1.59 s, CLS 0, TBT 0 ms, transfer 138,941 B.
- Input checks accepted documented boundaries (15/480 minutes and 0/1,000,000
  deposit cents) and rejected values just outside them, bad slugs, HTTP payment
  URLs, arbitrary provider URLs, missing consent/contact details, and invalid
  times. Eight simultaneous bookings for one slot produced one `201` and seven
  `409` responses.
- Every rendered internal link resolved successfully. Unknown routes return a
  designed 404 response. The expected browser resource error for the 404
  document is the only console error in the route sweep.

This is not a library, CLI, or PWA. Package-consumer and service-worker
update/offline-reload checks are not applicable. The explicit online/offline
demo error state passed locally.

## Required release work

1. Deploy the candidate with the shared PostgreSQL `DATABASE_URL` and shared
   `CONTACT_ENCRYPTION_KEY`; verify independent-connection read/delete and
   service-wide rate limits across all replicas.
2. Implement and provision a supported email **and SMS** provider adapter with
   real credentials and provider-specific payloads/receipts; verify a live
   recovery, reminder, bounce, and fallback.
3. Implement Stripe-hosted per-booking Checkout creation and signed Stripe
   webhook verification instead of a static URL and generic callback token.
4. Register and enable the $29/month Sociobot subscription, subscription state,
   and hosted billing flow.
5. Implement the planned Sociobot Microsoft Entra External ID account and
   tenant model, including JWT validation and member access.
6. Increase all interactive target boxes to 44×44 px and make the demo readable
   at 200% text size.

## Evidence index

- `verification-evidence-6/live-api-probes.json`
- `verification-evidence-6/deployment-match.json`
- `verification-evidence-6/live-browser-audit.json`
- `verification-evidence-6/live-flow/live-check.json`
- `verification-evidence-6/lighthouse-mobile.json`
- `verification-evidence-6/verify-url/verify.json`
- `verification-evidence-6/headers-*.txt`
