# Booking Recovery Loop

Booking Recovery Loop helps solo coaches, tutors, and consultants act when a
paid booking stops. A practice can publish one session page, record channel
consent, open hosted payment, and review provider receipts.

Try the isolated sample at
`https://booking-recovery-loop.sociobot.in/?demo=1`. It opens three fictional
bookings without an account or payment. Demo actions stay on the product origin
and send no real message.

## Product workflow

1. Create a private practice workspace at `/start`.
2. Publish the generated `/b/<slug>` session page.
3. A client records email or SMS consent before hosted payment opens.
   A time already held by another active booking is rejected.
4. The payment provider confirms a deposit through the authenticated callback.
5. A connected delivery service returns accepted, delivered, bounced, or failed receipts.
6. One permitted SMS fallback can follow an email bounce.
7. Export or delete the complete practice from `/app/settings/data`.

Client contact fields are encrypted before database storage. Owner tokens scope
every private read and write to one practice. The demo uses a separate token,
schema path, and fictional seed. Each statement above has one named test in
[.factory/claims.json](.factory/claims.json).

## Run locally

Requirements: Node 22+, npm, and current stable Rust.

```sh
npm ci
npm run build
cargo run --manifest-path backend/Cargo.toml
```

Open `http://127.0.0.1:8080`. Optional settings are `PORT`, `DATABASE_URL`,
`STATIC_DIR`, and `CONTACT_KEY_FILE`. With no settings, the container creates
its SQLite database and encryption key under its writable data directory.

## Verify

```sh
npm test
npm run check:backend
npm run test:deployment
npm run test:e2e
npm run build
npm run check:size
```

Playwright starts the complete Rust and Vite build. It checks each browser
claim plus keyboard, route, mobile, offline, privacy, and axe coverage.

Build the production image with:

```sh
docker build -f Dockerfile --build-arg BUILD_SHA=local -t booking-recovery-loop .
docker run --rm -p 8080:8080 booking-recovery-loop
```

The factory deploys that non-root container on `PORT`. `/health` reports the
build SHA. No secret is stored in this repository.

## License

[MIT](LICENSE). Fraunces and Atkinson Hyperlegible Next use the SIL Open Font
License; their texts are in `public/fonts/`.
