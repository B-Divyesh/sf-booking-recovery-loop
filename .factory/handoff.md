# Repair 11 handoff — code repaired; factory provisioning still blocks release

**Verifier report:** `d16d0ff6a9bc0f0f01643afdc20534431b6c7181`

**Candidate reproduced:** `52bc3a0b397b94dedf859edc3df0169ba5e6768d`

**Repaired runtime source:** `da417e3579af`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

## Outcome

The mandatory storage remediation ran first. Runtime state now uses only a
WAL-enabled SQLite file and generated contact key under `/data`. The deployment
contract pins one replica and `deploy.data_dir` remains `/data`. The Rust
dependency graph is SQLite-only. A repository regression check reconstructs
and rejects every controller-supplied prohibited resource identifier, legacy
connection setting, server-database URL, and non-SQLite driver name without
retaining those strings in source.

The rate-limit, reset, and clean-clone claim findings are repaired with exact
regression coverage. The running app has also been safely pinned to one
replica, and independent live probes now return the required counts.

The repaired image cannot be released yet. The factory-managed Container Apps
environment storage `sf-booking-recovery-loop-data` does not exist. The app
patch was rejected before any revision change. The separate deposit product
and credentialed delivery relay also remain unprovisioned. The product still
fails closed for deposits and real messages, so this handoff does **not** claim
release readiness.

## Reproduction and repair

- The candidate contains 42 prohibited storage/config references outside lock
  files. Its deployment contract selects an external server database and three
  replicas. This was reproduced from the candidate tree before edits.
- The API now starts with only `PORT`, creates
  `/data/booking-recovery-loop.sqlite3` plus `/data/contact.key`, enables WAL,
  runs migrations, and logs whether configuration was supplied or generated
  without logging secret material.
- A local SQLite-only SQLx facade removes unused server-database drivers from
  `Cargo.lock` and the container build graph.
- One process-wide limiter uses the first forwarded client IP. Writes permit
  exactly 12 immediate requests; reads permit exactly 40. Every rejected
  request returns 429, `Retry-After`, and the advertised limit.
- Demo reset revokes the old token for every subsequent connection. Demo and
  practice data are read from the same durable SQLite file.
- `.factory/claims.json` adds restart persistence and independent read-limit
  claims. `npm run test:claims` invokes every one of the 28 listed commands.
  Browser claim commands install locked dependencies themselves, including
  from an archived checkout with no `node_modules`.
- The deploy wrapper touches only `sf-booking-recovery-loop`, preserves only
  approved delivery references, mounts the expected factory volume, enforces
  one replica, validates the result, and removes obsolete app-local secret
  names without reading values.

## Verification evidence

All checks below passed on 2026-08-30 UTC.

- `npm ci` — 64 packages installed; 0 vulnerabilities.
- `npm test` — 11/11 tests.
- `npm run check:backend` — 33/33 tests, including independent request
  handles, reset revocation, cross-pool isolation, migrations, and two SQLite
  restart tests.
- `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`
  — pass.
- `cargo build --manifest-path backend/Cargo.toml --release --locked` — pass.
- `npm run test:deployment` — pass, including the prohibited-reference scan,
  `/data` contract, WAL, and exactly one replica.
- `npm run build && npm run check:size` — pass; JavaScript 79,408 bytes gzip,
  CSS 22,252 bytes raw.
- `npm run test:e2e` — 28/28 Chromium checks. This covers desktop, 390 px,
  200% text, keyboard, focus/history, serious/critical Axe checks, privacy,
  offline error handling, response policy, cache behavior, and claims.
- `npm run test:claims` — every one of 28 manifest commands passed
  independently.
- Fresh `git archive` checkout with no dependencies:
  `npm run test:claim:e2e -- --grep @claim:demo-no-account-payment` — pass
  after the script's own clean install.
- Only-`PORT` release process restart — the first process created workspace
  `01a05138-f672-7be3-9897-37279004b5c7`; after graceful stop and a fresh
  process, the same token returned that ID and all three sample attempts from
  `/data/booking-recovery-loop.sqlite3`. The persisted key was reused.
- Standalone Axe 4.10.3 — 0 violations on `/`, `/demo`, `/privacy`, `/terms`.
- `verify-url.sh` — 563 ms load, zero console errors, correct title and `lang`,
  one h1, one main landmark, zero missing alt attributes.
- Local Lighthouse: Performance 99, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1,414 ms, LCP 1,727 ms, CLS 0, TBT 0 ms, transfer 160,474 bytes.

Local reports and screenshots are under
[`repair-11-evidence`](repair-11-evidence/).

## Live state and deployment evidence

- Running source remains candidate
  `52bc3a0b397b94dedf859edc3df0169ba5e6768d`, revision
  `sf-booking-recovery-loop--0000063`. The repaired image was not activated.
- The named app now has `minReplicas: 1`, `maxReplicas: 1`. Its only app
  setting name is `PORT`; it reports no app secret names. No protected service,
  setting, or secret was inspected or changed.
- Independent live request contexts now prove 12 created plus 28 limited
  writes, 40 loaded plus 120 limited reads, and 24/24 old-token reads returning
  404. Every limiter response includes the required headers. See
  [`live-check.json`](repair-11-evidence/live-current/live-check.json).
- Live routes, 390 px layout, same-origin demo, subscription checkout, and
  console checks pass. `/health` correctly identifies the still-running
  candidate.
- Live integration status reports both dedicated billing and delivery as
  unconfigured. The dedicated deposit checkout returns 404.
- ACR build `ch1hb` built and pushed runtime source `da417e3579af` as image
  digest `sha256:542ce310a5dc4c546f70557f98472ed939a3ea75da20efd756ca05f6848478fd`.
  Its log fetched and compiled only the SQLite SQLx driver. The app-only patch returned
  `ManagedEnvironmentStorageNotFound` for
  `sf-booking-recovery-loop-data`. The request failed before changing the
  revision. The repair deliberately did not create or inspect shared storage,
  DNS, certificates, registries, vaults, databases, or other applications.

## Needs operator action

1. Let the factory provision and attach the work order's durable
   `sf-booking-recovery-loop-data` environment storage at `/data`. Do not point
   this app at any legacy data service.
2. Register and activate the variable-amount
   `booking-recovery-loop-deposit` product in the Sociobot billing service.
3. Provision the approved credentialed email/SMS relay URL, bearer token, and
   callback secret on this app only.
4. Run `./scripts/deploy-container.sh`, then verify `/health` reports the repair
   commit and repeat the process-restart persistence probe against the mounted
   `/data` file.
5. Complete one real Entra owner → public booking → hosted deposit → verified
   callback → recovery/reminder/receipt → signed bounce → one SMS fallback
   flow. These external effects cannot be honestly proven while the two
   integrations report unconfigured.

## Commands

```sh
npm ci
npm test
npm run check:backend
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
npm run test:deployment
npm run build
npm run check:size
npm run test:e2e
npm run test:claims
./scripts/deploy-container.sh
```
