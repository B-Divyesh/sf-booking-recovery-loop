# Booking Recovery Loop

Review stopped paid bookings with recorded consent and delivery evidence. It is
for solo coaches, tutors, and consultants.

Try the isolated sample at
`https://booking-recovery-loop.sociobot.in/?demo=1`.

## How booking recovery works

1. Sign in with Sociobot Entra External ID, then create a practice at `/start`.
2. Publish its `/b/<slug>` booking page and record the client’s consent.
3. Review the resulting recovery records and export or delete the practice data.

Recovery Loop Practice is $29/month for one practice. The product links to the
registered Sociobot/Dodo hosted checkout. Subscription entitlement state is
stored server-side; the browser never receives a billing secret.

Live email/SMS delivery is deliberately unavailable in this deployment because
no credentialed provider adapter has been provisioned. The product does not
offer a fake Resend connection or claim that a recovery was sent. The isolated
demo continues to show a simulated receipt without contacting a provider.

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

For the multi-replica production path, set `DATABASE_URL` to the factory's
shared PostgreSQL connection and `CONTACT_ENCRYPTION_KEY` to the shared 32-byte
hex or base64url secret. Set `REQUIRE_SHARED_DATABASE=1` in production: it
refuses the replica-local SQLite fallback. With neither set, the service starts
in local SQLite mode for development and tests only. The deployment contract is
recorded in `deploy/containerapp.m1.json`.

The API validates Entra discovery, issuer, JWKS/RS256 signature, audience,
tenant, expiry, and stable `oid` before it opens an owner workspace. Local
integration fixtures may use an isolated loopback delivery endpoint only with
`ALLOW_UNSAFE_TEST_DELIVERY_URLS=1`.

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
docker build -f Dockerfile --build-arg BUILD_SHA=local -t booking-recovery-loop .
docker run --rm -p 8080:8080 booking-recovery-loop
```

Set `PORT` when you need a port other than 8080. Use `/health` for a health
check.

## License

[MIT](LICENSE). Fraunces and Atkinson Hyperlegible Next use the SIL Open Font
License; their license texts are in `public/fonts/`.
