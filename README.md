# Booking Recovery Loop

Review stopped paid bookings with recorded consent and delivery evidence. It is
for solo coaches, tutors, and consultants.

Try the isolated sample at
`https://booking-recovery-loop.sociobot.in/?demo=1`.

## How booking recovery works

1. Sign in with Sociobot Entra External ID, then create a practice at `/start`.
2. Publish its `/b/<slug>` booking page and record the client’s consent.
3. Each booking receives a server-created Sociobot/Dodo hosted checkout.
4. Review recovery records and export or delete the practice data.

Recovery Loop Practice is $29/month for one practice. The product links to the
registered Sociobot/Dodo hosted checkout. Subscription entitlement state is
stored server-side; the browser never receives a billing secret.

When configured, live email/SMS delivery uses a server-owned provider adapter. It sends only
after channel consent, authenticates requests with a bearer credential, checks
callback bodies with HMAC-SHA256, stores durable receipts, and permits one SMS
fallback after an email bounce. The isolated demo remains simulated.

The demo has separate sample storage. It sends no real messages or payments.
Every product promise above is listed with its executable evidence in
[`.factory/claims.json`](.factory/claims.json).

## Run locally

Requirements: Node 22+, npm, and current stable Rust.

```sh
npm ci
npm run build
cargo run --manifest-path backend/Cargo.toml
```

Open `http://127.0.0.1:8080`.

With no storage environment variable, the service creates
`/data/state/booking-recovery-loop.sqlite3` in rollback-journal mode and `/data/state/contact.key` on
first boot. Set `BOOKING_DATA_DIR` only for local development on a host where
`/data` is unavailable. The production contract pins one replica and mounts a
durable volume at `/data`; it is recorded in `deploy/containerapp.m1.json`.

The API validates Entra discovery, issuer, JWKS/RS256 signature, audience,
tenant, expiry, and stable `oid` before it opens an owner workspace. Local
integration fixtures may use an isolated loopback delivery endpoint only with
`ALLOW_UNSAFE_TEST_DELIVERY_URLS=1`.

Configure live delivery with `DELIVERY_PROVIDER_URL`,
`DELIVERY_PROVIDER_TOKEN`, and `DELIVERY_CALLBACK_SECRET`. The approved relay
signs callbacks as `X-Provider-Signature: sha256=<HMAC-SHA256(raw body)>`.
Set `PUBLIC_BASE_URL` to the public origin. Booking checkout uses
`SOCIOBOT_BILLING_BASE_URL` (default `https://api.sociobot.in/api/v1`) and
`SOCIOBOT_BOOKING_PRODUCT_SLUG` (the dedicated booking-deposit product, not the
practice subscription). The browser receives no billing credential.

The service fails closed when either integration is absent: it does not accept
a booking without a dedicated deposit checkout, and it does not mark a message
sent without the complete delivery credential set.

## Verify

```sh
npm test
npm run check:backend
npm run test:deployment
npm run test:e2e
npm run build
npm run check:size
```

## Deploy

```sh
./scripts/deploy-container.sh
```

This factory-only command builds the image, verifies the existing `/data`
volume mount, pins the app to one replica, removes obsolete app-local database
settings, and applies `deploy/containerapp.m1.json`. It never creates or reads
an external database or secret store. The app remains runnable with only
`PORT`; use `/health` for a health check.

`npm run test:deployment` rejects forbidden external-resource identifiers and
proves the one-replica `/data` contract. The backend suite closes and reopens a
SQLite pool, then checks that durable state remains in the same mounted path.

## License

[MIT](LICENSE). Fraunces and Atkinson Hyperlegible Next use the SIL Open Font
License; their license texts are in `public/fonts/`.
