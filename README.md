# Booking Recovery Loop

Booking Recovery Loop is being built for small tutoring, coaching, and
consulting practices that need a paid appointment to make it from booking
intent to a completed session. The product will focus on consented recovery and
visible reminder delivery proof, not replacing a calendar or CRM.

This commit is the venture-planning and engineering foundation. It does **not**
yet provide booking, accounts, payments, or outbound messages. The executable
roadmap is in [.factory/plan.md](.factory/plan.md); the product’s visual system
is in [.factory/design.md](.factory/design.md).

## What is here

- A strict TypeScript/Vite public-shell foundation with route titles, semantic
  landmarks, focus management, tokens, a product-native 404 page, and static
  deployment headers.
- A health-only Rust/axum service that starts without secrets on `PORT` (8080
  by default), emits structured JSON logs, and has a multi-stage Dockerfile.
- Unit-test, build, and GitHub Actions quality gates.
- The M1 demo and claim contracts, ready for the next builder to implement.

## Develop

Requirements: Node 22+, npm, and Rust 1.98+ for the API checks.

```sh
npm install
npm run dev
```

Open the local Vite URL shown in the terminal. The `/demo`, `/privacy`, and
`/terms` routes are foundation placeholders until M1. The finished product will
ship at `https://booking-recovery-loop.sociobot.in` through the factory.

## Verify

```sh
npm test
npm run build       # writes dist/
npm run check:backend
npm run build:backend
```

`npm run test:e2e` is reserved for the M1 Playwright claim suite. Its planned
tests are defined in `.factory/claims.json`; do not present those claims to
customers before M1 implements and verifies them.

## API foundation

```sh
cargo run --manifest-path backend/Cargo.toml
curl http://127.0.0.1:8080/health
```

For the later PostgreSQL-backed milestones, start the local database with
`docker compose up postgres`. The foundation service does not connect to it yet.

## Deployment and privacy

The factory owns deployment. `backend/Dockerfile` is a non-root, multi-stage
container and accepts factory build identity through `BUILD_SHA`. Runtime
configuration is intentionally minimal at this stage: only optional `PORT` is
read. Do not commit secrets.

No analytics, external runtime scripts, webfonts, customer data collection,
payment collection, or messaging is present in this foundation. M1 will replace
the policy placeholders before it stores any data; M2 will add Sociobot Entra
CIAM and Sociobot/Dodo subscription billing; M3 will add Stripe-hosted session
deposits as specified in the venture plan.

## License

[MIT](LICENSE)
