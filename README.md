# Booking Recovery Loop

Booking Recovery Loop helps solo tutors, coaches, and consultants recover a
stopped paid booking without losing the consent record or delivery evidence.
M1 ships a public product page and a working, isolated recovery sandbox.

Try the deployed sample at
`https://booking-recovery-loop.sociobot.in/demo`. It uses three fictional
bookings, never sends a real message, and needs no account.

## What M1 includes

- A one-click demo of a consented abandoned-booking recovery.
- A server-enforced stop when the sample has no email consent.
- A simulated, timestamped delivery receipt from an in-process mailbox.
- Portable demo workspace tokens with 256 random bits and a 24-hour expiry.
- A reset control that replaces the current browser workspace and restores the seed.
- Plain-language privacy, terms, and product-native not-found routes.
- Per-IP API limits keyed from the first `X-Forwarded-For` hop.

The demo is not a production account. It does not call Entra, Sociobot
billing, Dodo, Stripe, a messaging provider, or an AI service. CIAM,
PostgreSQL customer tenancy, and the $29/month hosted subscription flow are
M2 scope in [.factory/plan.md](.factory/plan.md).

## Stack

- Vite 6 and strict TypeScript with semantic HTML and product-native CSS.
- Rust 2021, axum, sqlx, and SQLite for temporary M1 demo workspaces.
- A single non-root container serving both the API and built `dist/` assets.

The shared production customer store moves to PostgreSQL in M2, before real
practice data exists. The M1 container starts with only `PORT` and creates its
demo database in the working directory.

## Run locally

Requirements: Node 22+, npm, and Rust 1.98+.

```sh
npm ci
npm run build
cargo run --manifest-path backend/Cargo.toml
```

Open `http://127.0.0.1:8080`. The service creates
`booking-recovery-loop.db` locally. Optional configuration:

- `PORT` — HTTP port, default `8080`.
- `DATABASE_URL` — SQLite URL, default
  `sqlite://booking-recovery-loop.db`.
- `STATIC_DIR` — built web directory, default `dist` locally and `/app/dist`
  in the container.

## Verify

```sh
npm test
npm run check:backend
npm run test:e2e
npm run build
npm run check:size
```

The Playwright command builds and starts the complete service itself. It runs
one browser test per claim in [.factory/claims.json](.factory/claims.json),
plus axe and keyboard checks for every public route.

To build the production image:

```sh
docker build -f backend/Dockerfile --build-arg BUILD_SHA=local -t booking-recovery-loop .
docker run --rm -p 8080:8080 booking-recovery-loop
```

## Data and migrations

Migration `0001_demo_workspaces` creates demo workspaces, booking attempts,
outbound message records, and delivery events. The matching `.down.sql` file
removes them, and the backend test applies both directions. Each portable
token contains no personal data; server replicas store only its SHA-256 hash.

See [.factory/demo.md](.factory/demo.md) for the sandbox boundary and
[.factory/design.md](.factory/design.md) for the visual system and asset
provenance.

## Deployment

The factory deploys `backend/Dockerfile` as a container on port 8080. The
Dockerfile accepts `BUILD_SHA` and does not depend on `.git`. Do not add
secrets to this repository.

## License

[MIT](LICENSE). Fraunces and Atkinson Hyperlegible Next use the SIL Open Font
License; their license texts are in `public/fonts/`.
