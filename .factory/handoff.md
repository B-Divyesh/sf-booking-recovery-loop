# Repair 12 handoff — durable SQLite release deployed and verified

**Verifier report:** `d16d0ff6a9bc0f0f01643afdc20534431b6c7181`

**Repair input:** `05bad283d32411f1aabc5dd35b294c1b531409a0`

**Verified runtime source:** `2754586c4ef8110d32ec57d4e310f673901ff2b7`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

## Outcome

The release-blocking durable-storage deployment failure is repaired. The
repository deploy wrapper now delegates storage creation and mounting to the
factory deployer with `deploy.data_dir=/data`; it no longer sends an
application patch that refers to a share before the fleet has created it.

The factory created and mounted `sf-booking-recovery-loop-data`, deployed
revision `sf-booking-recovery-loop--0000071`, and left the application at one
running replica. The live process stores state in
`/data/state/booking-recovery-loop.sqlite3`. Its startup log identifies that
SQLite path and a reused persisted contact key.

The first mounted boots also exposed an Azure Files lock incompatibility that
could not appear before the share existed. The runtime now uses SQLite's
rollback journal, one pool connection, and the SQLite `nolock=1` URI mode
under the enforced one-replica boundary. Startup retries transient lock errors.
The failed WAL bootstrap file was left untouched; the usable database lives in
the new `/data/state` directory.

The storage-safe single connection made the previous token-bucket read limit
latency-dependent. Reads now replenish once per second, so an immediate burst
is exactly 40 regardless of SQLite response latency. The live independent
probe returns 40 accepted and 120 limited reads.

## Exact reproduction and regression coverage

- Candidate `05bad28` passed its old deployment test even though
  `scripts/deploy-container.sh` called Azure directly and referenced
  `sf-booking-recovery-loop-data` before provisioning. The recorded release
  failed with `ManagedEnvironmentStorageNotFound`.
- The new deployment regression initially failed on the candidate with
  `The product deploy wrapper must not perform direct cloud mutation: az acr`.
  It now executes the wrapper against a fake factory deployer and proves the
  exact arguments: `booking-recovery-loop`, repository root, `Dockerfile`,
  port `8080`, and `WO_DATA_DIR=/data`.
- `mounted_sqlite_startup_avoids_unsupported_network_file_locks` holds an
  external SQLite lock while the mounted-filesystem runtime opens, migrates,
  and reads the same database in one-process mode.
- Existing restart, cross-connection, reset-revocation, recovery concurrency,
  tenant-isolation, auth, billing-boundary, and delivery-fixture regressions
  remain green.

## Local verification

Run on 2026-08-30 UTC from locked dependencies:

- `npm ci` — 64 packages; 0 vulnerabilities.
- `npm run test:all` — 11/11 Vitest tests, 34/34 Rust tests, deployment
  regression pass, 28/28 Chromium tests, and production build pass.
- `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`
  — pass.
- `cargo build --manifest-path backend/Cargo.toml --release --locked` — pass.
- `npm run test:claims` — all 28 manifest commands passed independently.
- `npm run check:size` — JavaScript 79,408 bytes gzip; CSS 22,252 bytes raw.
- Browser coverage includes desktop, 390 px, 200% text, keyboard-only use,
  focus/history, serious/critical Axe checks, offline error handling,
  same-origin privacy, cache headers, security policy, and true 404 responses.
- Local Docker/Podman was unavailable. Factory ACR build `ch1m9` built the
  production Dockerfile successfully.

## Live verification

- `/health` returns
  `2754586c4ef8110d32ec57d4e310f673901ff2b7`.
- Azure reports revision `sf-booking-recovery-loop--0000071` healthy with
  `minReplicas=1`, `maxReplicas=1`, and one running replica.
- The app has one `AzureFile` volume named
  `sf-booking-recovery-loop-data`, mounted at `/data`.
- Restart persistence passed. Workspace
  `01a0518f-fc1a-7663-88da-2923e2d31476` had three bookings before and after
  an explicit restart of revision `0000071`; it also survived the preceding
  revision deployment.
- The startup log reports
  `sqlite_path=/data/state/booking-recovery-loop.sqlite3` and
  `key_source=persisted`. The dependency graph and deployment check contain
  only the SQLite database driver.
- `scripts/verify-live.mjs` passed: all product routes, desktop and 390 px
  layouts, same-origin demo traffic, zero console errors, 12 accepted plus 28
  limited writes, 40 accepted plus 120 limited reads, and 24/24 revoked-token
  reads returning 404.
- `verify-url.sh` passed in 591 ms: correct title/lang, one h1, one main,
  no missing alt text, no unlabeled buttons, and no console errors.
- Live Lighthouse: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1,311 ms, LCP 1,701 ms, CLS 0, TBT 24 ms, transfer 156,742 bytes.
- Unauthenticated owner API requests return 401 with
  `WWW-Authenticate: Bearer`. Unknown routes return 404. Hashed assets return
  `Cache-Control: public, max-age=31536000, immutable`.

Evidence is under [`.factory/repair-12-evidence/`](repair-12-evidence/).

## Remaining operator-owned integrations

The live integration-status endpoint still reports the dedicated booking
deposit product and credentialed email/SMS relay as unconfigured. The code
fails closed and its recorded fixture tests pass, but real deposits and
outbound messages remain unavailable until the factory provisions those two
product-specific integrations. The Entra callback registration also still
needs operator confirmation. No shared service, database, secret, or
out-of-scope application was inspected or modified during this repair.

## Commands

```sh
npm ci
npm run test:all
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo build --manifest-path backend/Cargo.toml --release --locked
npm run test:claims
npm run check:size
./scripts/deploy-container.sh
node scripts/verify-live.mjs https://booking-recovery-loop.sociobot.in .factory/repair-12-evidence/live-final
```
