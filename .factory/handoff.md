# Repair 5 handoff — implementation complete; external release gate remains

**Work order:** `booking-recovery-loop-repair-5`
**Base:** `c31da6c41dfea11c01aca3158b2372038a8604ab`
**Repair commit:** `ec90392` (`fix: harden shared practice runtime`)
**Status:** local repair verification passes. Deployment, billing registration,
and Entra redirect registration were not performed from this repository.

## What changed

- Reproduced the verifier's split-store failure as an exact two-replica
  create/read/delete regression. The new test creates a practice through one
  independent API router backed by a shared durable database, reads and
  deletes it through another, then proves deletion is visible to the first.
- Replaced the production data-path contract with SQLx `AnyPool`: SQLite is
  retained for no-environment local/test boot while `DATABASE_URL` supports
  the shared PostgreSQL deployment. The deployment contract specifies the
  factory PostgreSQL secret, a shared contact-encryption secret, managed
  point-in-time restore, and multi-replica scale.
- Moved the service-wide API allowance into the database, before every
  `/api/v1` endpoint. `/health` is the sole exemption. Existing route limits
  remain stricter for writes; DELETE now returns 429 with a positive
  `Retry-After` instead of being whitelisted.
- Removed production owner-controlled delivery destinations. The only
  accepted production connection value is the supported `resend` provider;
  HTTP redirects are disabled. Loopback URLs are allowed only in explicitly
  opted-in automated fixtures. Privacy copy now identifies the contact-data
  transfer boundary.
- Replaced the incomplete queued-reminder evidence with an end-to-end due-job
  test: a verified payment queues a reminder, the scheduler runs twice, and
  exactly one provider delivery and sent job are observed.

## Exact local evidence

Executed after `npm ci` on 2026-08-29 UTC:

```text
npm test                                                        10 passed
npm run check:backend                                           22 passed
npm run test:deployment                                         passed
npm run build                                                   passed; dist/ produced
npm run check:size                                              JS 12,539 B gzip; CSS 21,906 B raw
npm run test:e2e                                                27 Chromium passed
cargo clippy --all-targets -- -D warnings                      passed
cargo build --release                                           passed
all 25 commands in .factory/claims.json                        passed individually
PORT=4191 cargo run; GET /health                               200 {"status":"ok","build_sha":"dev"}
```

The new claim commands are:

```text
shared-practice-storage  cargo test ... shared_durable_store_prevents_the_verifier_cross_replica_read_and_delete_split
automatic-reminder       cargo test ... automatic_reminder_is_delivered_once_when_due_after_verified_deposit
```

## Needs factory operator action before release

This repository is not authorized to mutate infrastructure, billing, or Entra
application registration. The live Container App was read-only inspected and
still runs the verifier's image with `minReplicas: 1`, `maxReplicas: 3`, no
volume, and only `PORT`; it must be redeployed with the contract in
`deploy/containerapp.m1.json`, specifically the shared PostgreSQL
`DATABASE_URL` and `CONTACT_ENCRYPTION_KEY` secret. Run a live cross-replica
create/read/delete and 200-request rate probe after deployment.

The factory must also register/enable the `booking-recovery-loop` $29/month
Sociobot/Dodo product (the verified checkout endpoint returned 404), register
`https://booking-recovery-loop.sociobot.in/auth/callback` on the shared Entra
SPA application, and supply/configure production Stripe and delivery-provider
credentials. The legacy owner-key payment callback and workspace flow are
still present, so the full CIAM, signed Stripe webhook, and usable paid-tier
gate are not honestly release-ready until those product integrations are
implemented and live-verified.

---

# Verification 5 handoff — FAIL

**Work order:** `booking-recovery-loop-verify-5`

**Candidate and deployed build:** `3e0256e1a0d72dcd315731554ad072122eca56b6`

**Live URL:** https://booking-recovery-loop.sociobot.in

**Decision:** **FAIL — do not release**

## Independent verification result

- A newly created live practice was not consistently reachable. Its public URL
  returned `200` 10 times and `404` 20 times across 30 requests; owner read and
  delete also alternated between missing and present. Real data and its
  encryption key are held in independent container-local SQLite stores.
- Required billing is unavailable: the Sociobot checkout endpoint returns 404.
  Session payment and delivery are generic owner-entered URLs rather than
  supported Stripe and email/SMS integrations.
- `DELETE /api/v1/practice` is exempt from rate limiting. Forty-five requests
  from one client all reached the handler without `429` or `Retry-After`.
- User-controlled delivery URLs are called server-side without private-network,
  redirect, or host allowlist protection.
- Production customer data uses a browser-stored owner key, not the required
  Sociobot Entra account and tenant model.
- “Reminders run automatically” is not covered by a claim that runs a due
  reminder through delivery.

After `npm ci`, all 23 claim commands, 10 frontend tests, 19 backend tests, 27
browser tests, strict builds, bundle checks, rustfmt, and Clippy passed. Live
first-read, demo privacy, desktop/mobile axe, keyboard, focus, reduced motion,
security headers, candidate identity, and Lighthouse passed. Mobile Lighthouse
scored 100 in all four categories with LCP 1.52 s and CLS 0. The demo write
limit passed at 12 accepted requests followed by `429 Retry-After: 60`.

Full findings and evidence: [verification-5.md](verification-5.md) and
`verification-evidence-5/`.

Required work is shared durable storage with backup/restore, service-wide
limits on every non-health endpoint, Sociobot billing and Entra, real signed
payment/provider integrations, SSRF controls, and the missing reminder claim.
No product code was modified during verification.

---

# Previous builder handoff — Polish 3

**Work order:** `booking-recovery-loop-polish-3`
**Reviewed candidate:** `256bde53b0e8107421ceda018d4b3a61203ce894`
**Deployed source:** `15bd99b9765cdbfc6cf25316948b37615323cf25`
**Live URL:** https://booking-recovery-loop.sociobot.in

## What changed

- Rewrote the landing headline as **“Recover unfinished paid-session
  bookings”** and updated home metadata. The direct `/?demo=1` path keeps its
  persistent isolated-data banner, reset control, and start-for-real exit.
- Added a durable, measured 15-minute recovery-schedule claim, a complete
  practice-data-inventory claim, a no-card-data browser claim, and a distinct
  automatic-reminder claim test. The privacy page now names every stored
  record type consistently.
- Added an owner-only **Send delivery test** action. It sends a connection-test
  payload without recipient or client data and never creates a booking.
- Preserved the product's twilight appointment-rail visual system, route
  titles, focus handoff, designed 404, legal links, mobile layout, and local
  demo namespace.
- Added the live ingress probe to `scripts/verify-live.mjs`. It requires twelve
  accepted demo writes and a thirteenth `429` with `Retry-After`.

## Verification evidence

All 23 declared claim commands were run individually in a clean clone at
`/tmp/booking-recovery-final-clean.icZuvD`, after `npm ci`. The clean clone also
passed:

```text
npm test                  10 passed
npm run check:backend     19 passed
npm run test:deployment   passed
npm run build             dist/ produced
npm run check:size        JS 12,457 bytes gzip; CSS 21,906 bytes raw
npm run test:e2e          27 passed
```

Final live verification against deployed `15bd99b`:

```text
GET /health               200; build_sha 15bd99b9765cdbfc6cf25316948b37615323cf25
scripts/verify-live.mjs   passed: demo, reset, real setup, booking, consent,
                           export/delete, routes, mobile, console, ingress limit
verify-url.sh             passed: title, lang, h1, main, image alt, no console errors
live axe                  0 violations on / desktop; /demo, /privacy, /terms at 390 px
Lighthouse mobile         Performance 100; Accessibility 100; Best Practices 100; SEO 100
LCP / CLS                 1.51 s / 0
```

Final evidence is committed under `.factory/repair-evidence/`:

- `polish-3-live-final/live-check.json` — cold live workflow and 12/13 ingress result.
- `polish-3-verify-url-final/verify.json` — semantic and console verifier result.
- `polish-3-live-final-axe.json` — live axe scans.
- `polish-3-live-final-lighthouse.json` — Lighthouse report.
- `polish-3-live-copy-check.json` — first-screen, privacy, timing, and card-boundary copy.

## Known external limitation / operator action

The factory billing registry has not enabled the required product. A direct
check of `https://api.sociobot.in/api/v1/products/booking-recovery-loop/checkout`
returned HTTP `404` with `{"error":"enabled factory product","status":404}`;
the captured headers and body are in
`.factory/repair-evidence/polish-3-billing-check.*`.

The repository must not create or alter billing products. To open the stated
$29 monthly plan, the factory billing operator must register and enable the
`booking-recovery-loop` Sociobot product. The UI deliberately says checkout is
unavailable rather than exposing a dead payment link. A practice can already
connect its existing hosted payment page and delivery endpoint, verify that
delivery connection without client data, and use the recovery workflow.

This external registration and a first-party, non-developer messaging-provider
connection are the remaining parts of review finding `F-3-2`; they cannot be
completed from this repository or with the runtime credentials supplied to the
container.
