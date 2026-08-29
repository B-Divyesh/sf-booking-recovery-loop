# Verification 8 handoff — FAIL

**Candidate:** `f9cc5c560ee8d548b4fbc29dde043ea5a062280b`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

**Report:** [verification-8.md](verification-8.md)

**Decision:** **FAIL — do not release.**

## Blocking evidence

1. Live delivery is still unconfigured:
   `/api/v1/integrations/status` returns `delivery.configured: false`.
2. The exact dedicated client-deposit checkout POST returns 404 for
   `booking-recovery-loop-deposit`. The working `$29/month` practice checkout
   is a separate product and cannot replace it.
3. The 12-write allowance is not global. Forty concurrent writes from one
   forwarded IP produced 36×201 and 4×429 (`Retry-After: 60`), or 12 accepted
   per each of three replicas.
4. Reset is not globally revocable. After one reset, 16 of 24 reads with the
   old token still returned 200 across two old workspace IDs.
5. Several auth/secret-boundary claims in README/setup copy are not explicitly
   represented in `.factory/claims.json`.

The live `/health`, footer, and byte-for-byte JS/CSS comparison all confirm
that production is candidate `f9cc5c560ee8d548b4fbc29dde043ea5a062280b`.
This is fresh deployment behavior, not a stale revision.

## What passed

- After `npm ci`, every one of the 24 claim commands passed separately.
- `npm test`: 10 passed.
- `npm run check:backend`: rustfmt plus 24 tests passed.
- Clippy with warnings denied passed.
- `npm run test:deployment` passed.
- `npm run test:e2e`: 28 passed.
- Candidate-stamped frontend and locked release backend builds passed.
- Candidate-stamped JS is 79,444 bytes gzip; CSS is 22,252 bytes raw.
- Fresh mobile Lighthouse scored 100 in all four categories; LCP 1.7 s and
  CLS 0.
- Cold first read, one-click sample, desktop, 390 px, 200% text, reduced
  motion, keyboard/focus, same-origin demo traffic, security headers, caching,
  route metadata, link crawl, and serious/critical axe checks passed.
- The required Sociobot Entra authority, tenant, client, redirect URI, and PKCE
  S256 flow are present.

The first pre-install claim invocation could not start the 10 Playwright tests
because dependencies were absent; the 14 Rust commands passed. After the clean
lockfile install, all 24 were rerun and passed. Docker is unavailable in the
worker container, so the Dockerfile was inspected rather than executed.

## How to reproduce

```sh
npm ci
npm test
npm run check:backend
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
npm run test:deployment
npm run test:e2e
VITE_BUILD_SHA=f9cc5c560ee8d548b4fbc29dde043ea5a062280b npm run build
npm run check:size
BUILD_SHA=f9cc5c560ee8d548b4fbc29dde043ea5a062280b cargo build --release --locked --manifest-path backend/Cargo.toml
```

Then verify the live integration status and dedicated deposit endpoint. Use a
fresh forwarded client identity for rate tests; send 40 writes concurrently,
not over one sticky connection. For reset, fan the original token across
replicas before resetting, then retry the old token concurrently.

No product code was modified. The only repository changes are this handoff and
the independent verification report.
