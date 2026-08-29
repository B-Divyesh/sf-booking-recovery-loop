# Repair 8 handoff

**Work order:** `booking-recovery-loop-repair-8`

**Verifier report:** `02c4cf37209d37e444824d1f290752c73ecabd5e`

**Failed candidate:** `cc5ce2c8289510bdd73e1133d5c1e99c5eab0cf9`
**Live URL:** <https://booking-recovery-loop.sociobot.in>

## Outcome

The two verifier defects are repaired in the product and covered at their
external boundaries. Owner-entered payment and delivery URLs are gone. A
booking now asks the Sociobot billing boundary for its own Dodo-hosted checkout,
stores the intent, and becomes paid only after Sociobot verifies the returned
license. A license hash is stored, never its plaintext value, and reuse across
bookings is rejected.

The delivery path now uses a server-owned HTTPS relay with bearer
authentication. Provider callbacks require HMAC-SHA256 over the exact body.
Authenticated event IDs and payload digests are durable and replay-safe. Email
bounces create at most one SMS fallback, and only when SMS consent and a phone
number were recorded. The scheduler continues to claim jobs durably and cannot
send when consent is missing or withdrawn.

The deployment remains a containerized web-with-backend product using Sociobot
Entra External ID and the shared PostgreSQL topology. Billing remains entirely
inside Sociobot/Dodo; Stripe and card fields were not added.

## Exact regression evidence

`configured_delivery_and_sociobot_checkout_fix_the_release_blocker_end_to_end`
uses separate HTTP processes for billing and delivery. It proves all of these
in one flow:

- the booking amount, currency, booking reference, and return URL reach the
  dedicated Sociobot checkout endpoint;
- the returned checkout and intent are stored for that booking;
- delivery receives the server bearer credential and an idempotency key;
- an unsigned receipt is rejected with 401;
- a correctly signed bounce is stored durably;
- replaying that callback does not duplicate the receipt or SMS;
- one permitted SMS fallback is sent;
- Sociobot verification changes the booking to paid;
- the same completion license cannot pay a second booking.

The browser harness runs a separate billing fixture process and follows the
real server checkout path to the allowlisted Dodo hostname. It also proves the
return license is removed from the address bar before server verification.

## Local verification

Run from a clean checkout:

```sh
npm ci
npm test
npm run check:backend
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
npm run test:deployment
npm run test:e2e
npm run build
npm run check:size
npm run build:backend
```

Results on 2026-08-29 UTC:

- clean `npm ci`: passed, 62 packages, zero vulnerabilities;
- Vitest: 10 passed;
- Rust API/integration: 24 passed;
- rustfmt and Clippy with warnings denied: passed;
- Playwright Chromium: 28 passed;
- every one of the 24 `.factory/claims.json` commands: passed separately;
- production frontend build: passed, JS 79,408 bytes gzip and CSS 22,252
  bytes raw;
- optimized Rust build: passed;
- release binary with no required integration configuration: started on
  `PORT`, returned 200 from `/health`, and logged graceful shutdown;
- desktop, keyboard, route focus, response policy, offline error state,
  serious/critical axe checks, and 390 px at 200% text: covered by the passing
  Playwright suite.

Migration `0007_owned_checkout_and_provider_callbacks` adds payment sessions
and authenticated provider-callback receipts. Its down migration removes only
those two tables. Existing legacy URL columns remain empty for safe migration
compatibility and are never populated from owner input.

## Deployment configuration

Non-secret configuration:

- `PUBLIC_BASE_URL=https://booking-recovery-loop.sociobot.in`
- `SOCIOBOT_BILLING_BASE_URL=https://api.sociobot.in/api/v1`
- `SOCIOBOT_BOOKING_PRODUCT_SLUG=booking-recovery-loop-deposit`
- `REQUIRE_SHARED_DATABASE=1`

Secret-backed configuration:

- `DATABASE_URL` — shared PostgreSQL runtime URL;
- `CONTACT_ENCRYPTION_KEY` — shared 32-byte contact encryption key;
- `DELIVERY_PROVIDER_TOKEN` — approved relay bearer credential;
- `DELIVERY_CALLBACK_SECRET` — shared provider callback HMAC secret.

`DELIVERY_PROVIDER_URL` must name the approved relay's HTTPS send endpoint.
No secret value belongs in source control or browser code.

## Needs operator action before release

The worker's Azure inventory contains no email/SMS delivery service, relay URL,
SMS credential, or delivery callback secret. Gmail OAuth client credentials
exist, but there is no refresh token and they do not provide SMS. Therefore no
honest live delivery configuration can be derived from the approved secrets.
Provision the approved dual-channel relay and add the three delivery settings
above. Until then `/api/v1/integrations/status` reports delivery unconfigured
and real sends fail closed as `delivery_not_connected`.

The approved billing APIs currently return 404 for
`booking-recovery-loop-deposit` in both production and pilot. The existing
`booking-recovery-loop` product is the $29 monthly practice subscription and
must not be reused for client deposits. Register the dedicated variable-amount
deposit product in Sociobot/Dodo with the production return origin, then retain
the configured product slug above. Until registration, booking creation rolls
back safely with `checkout_rejected` and does not occupy the chosen slot.

Confirm the Entra redirect URI
`https://booking-recovery-loop.sociobot.in/auth/callback` remains registered on
client `25c704f4-465a-47af-80ab-2c489466b697`.

These are external service-provisioning blockers, not repository TODOs. The
implementation and deterministic deployed-path fixtures are complete, but the
venture must not be declared releasable until live status and a controlled
email-bounce-to-SMS booking pass against the provisioned services.

## Live deployment evidence

The repaired image was built by ACR from commit
`56fcedd4cb0125f577a03a3204de843114e32b65` and deployed as Container App
revision `sf-booking-recovery-loop--0000031`. `/health` returned that complete
build SHA. The revision retained secret references for shared PostgreSQL and
contact encryption, plus `REQUIRE_SHARED_DATABASE=1`.

`node scripts/verify-live.mjs` passed against the public origin after the final
image rollout. It observed 200 responses for every real route, a real 404 for
an unknown route, three isolated demo bookings, token rotation on reset,
same-origin-only demo traffic, no console errors, no 390 px overflow, and
exactly 12 accepted writes followed by 429 with `Retry-After: 60`.

The live Entra redirect used the required Sociobot CIAM origin, tenant
`35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
`25c704f4-465a-47af-80ab-2c489466b697`, production callback URI, authorization
code flow, and PKCE S256. Unauthenticated practice access returned 401 with
`WWW-Authenticate: Bearer`; an unsigned provider callback also returned 401.

Mobile Lighthouse on the final image scored 100 performance, 100
accessibility, 100 best practices, and 100 SEO. LCP was 1,666 ms, CLS was 0,
TBT was 0 ms, and transfer size was 156,733 bytes. The server now gzip-compresses
the hashed application bundle and retains its immutable cache policy.

The live integration audit intentionally remains red: delivery reports
`configured: false`, and the dedicated deposit product returns 404 from the
approved billing API. Those results are the concrete operator-action evidence
described above; no substitute provider or wrong billing product was enabled.
