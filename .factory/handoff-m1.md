# M1 handoff — Booking Recovery Loop

**Work order:** `venture-booking-recovery-loop-m1`

**Date:** 2026-08-28

**Milestone status:** built, deployed, and verified; independent review and
polish remain the gate before M2.

## What shipped

- `/` now explains the booking-recovery job in plain words and opens the
  sample in one click. It follows the required landing-page order and states
  that production accounts and checkout are not open yet.
- `/demo` and `/?demo=1` create or resume an isolated North Star Coaching
  workspace with three fictional bookings. The banner remains visible and
  supports reset and exit.
- The complete M1 job works: select Maya Patel, inspect recorded email consent,
  run one accelerated sample follow-up, and read its timestamped simulated
  delivery receipt.
- Jordan Lee’s missing consent is enforced by the server with
  `409 consent_required`; no message or delivery event is written.
- `/privacy`, `/terms`, `/404`, and the standalone `404.html` are complete,
  titled routes with one h1, main landmarks, working navigation, and consistent
  twilight styling.
- The original appointment rail, social card, touch icon, Fraunces, and
  Atkinson Hyperlegible Next assets are self-hosted. Provenance and OFL texts
  are recorded in `.factory/design.md` and `public/fonts/`.

## Backend and data evidence

- Rust/axum serves the built Vite client and `/api/v1/demo/*` from one
  non-root container.
- SQLx persists demo workspaces, booking attempts, outbound messages, and
  delivery events in SQLite. The up migration is tracked and rerunnable; the
  down migration is exercised in the test suite.
- Portable workspace tokens contain 256 bits from the operating-system CSPRNG,
  an issue time, and a fixed fictional-state marker. They contain no personal
  data; each replica stores only a SHA-256 hash. Existing rows require the
  hash, `is_demo = 1`, and an unexpired 24-hour timestamp.
- A valid token can recreate only the fixed sample on another container
  replica. Recovery rotates its marker so the simulated receipt survives a
  replica change. The API suite proves this across three independent SQLite
  databases; no token path can select a production workspace.
- The API suite inserts a non-demo practice fixture and proves its token cannot
  pass through a demo route. It also proves expired workspaces are rejected.
- All product API routes use `tower_governor`. The limiter reads the first
  `X-Forwarded-For` hop, allows 20 requests per second with burst 40, and applies
  a stricter write burst of 12. The test proves `429` and `Retry-After`.
- Demo writes require an idempotency key. Recovery is also bounded by a unique
  attempt/channel row, preventing a duplicate simulated email.
- Security headers are present in both the container and static configuration.
  CSP allows only same-origin connections and assets.

The venture plan originally named PostgreSQL for shared customer data. M1 has
no customer or account data, and the runtime contract provides only `PORT`.
The plan now records SQLite as the temporary M1 demo store. M2 still introduces
PostgreSQL before any real practice data is accepted.

## Claims and automated evidence

All five entries in `.factory/claims.json` have exactly one Playwright test:

| Claim | Evidence |
| --- | --- |
| `demo-isolated` | token namespace, API request headers, and non-demo absence checked |
| `demo-reset` | changed token and restored original state checked |
| `consent-gates-recovery` | server rejection and receipt absence checked after reload |
| `demo-recovery-receipt` | delivered label, timestamp, and outcome checked |
| `demo-no-external-requests` | every request URL captured and asserted same-origin |

Local verification from the committed tree:

```text
npm test                                      2 files, 9 tests passed
cargo test --manifest-path backend/Cargo.toml 7 tests passed
npm run test:e2e                             11 tests passed
npm run build                                passed; dist/ produced
npm run check:size                           JS 8,352 B gzip; CSS 18,736 B raw
Lighthouse mobile /                          performance 100; accessibility 100
                                                best practices 100; SEO 100
Lighthouse LCP / CLS / total                 1.7 s / 0 / 118 KiB
```

Playwright also runs axe on landing, demo, privacy, terms, and the unknown-path
404, plus a keyboard-only skip/select/recover flow. There were no page console
errors. The landing copy audit is in `.factory/copy-audit.md`.

## Deployment

Deployment uses `/opt/fleet/lib/deploy-container.sh` with:

- slug `booking-recovery-loop`;
- repository `/work/repo`;
- Dockerfile `backend/Dockerfile`;
- container port `8080`;
- build arg `BUILD_SHA` supplied by the factory deployer.

Runtime variables are optional: `PORT`, `DATABASE_URL`, and `STATIC_DIR`.
There are no secrets in M1. The image defaults to port 8080, `/app/dist`, and
a generated local SQLite file under the non-root `/data` working directory.

Initial visual verification ran against commit
`b0ea43e6dadf17bc368f521792caacba81cfb134`. The final deployment includes the
portable-token replica correction in the commit containing this handoff; its
exact source commit is returned by `/health`. Cold verification against
`https://booking-recovery-loop.sociobot.in` found:

```text
GET /                                      200
GET /privacy, /terms, /404                 200 each
GET /health                                status ok; matching build SHA
verify-url load                            586 ms
title / lang / h1 / main / image alts      pass
browser console errors                     0
live mobile demo recovery                  pass
live demo request origins                  booking-recovery-loop.sociobot.in only
```

Screenshots, fetched HTML, and the verification JSON are in
`.factory/evidence/m1-live/`.

## Scope decision and M2 brief

The work-order preamble asked for CIAM and billing while also requiring exact
M1 plan scope. The plan explicitly assigns both to M2 and keeps M1 public,
account-free, and unable to charge. Pulling them into M1 would violate the
approved routes, claims, and demo-only boundary, so they were not added early.

M2 must now:

1. confirm the shared Entra callback
   `https://booking-recovery-loop.sociobot.in/auth/callback`;
2. add `@azure/msal-browser` PKCE and full API JWT validation from discovery;
3. add PostgreSQL practice/member migrations, tenant-scoped repositories, and
   row-level security;
4. add onboarding for practice, timezone, and service persistence;
5. wire the registered `$29/month` Recovery Loop Practice plan to Sociobot’s
   hosted Dodo-backed gateway with verified, idempotent billing events;
6. preserve every M1 demo claim without invoking auth or billing in demo mode.

Known M1 boundary: SQLite stores only ephemeral fictional demo records on each
container filesystem. Portable tokens keep that fixed sample coherent across
replicas. This is not the shared or backed-up customer store and must not be
reused for production practice data.
