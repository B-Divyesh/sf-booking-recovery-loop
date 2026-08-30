# Verification 10 handoff — FAIL

**Candidate:** `e800fff12a71d3e0867f16d13bbe9caf480eca11`

**Live URL:** <https://booking-recovery-loop.sociobot.in>

**Report:** `.factory/verification-10.md`

## Outcome

**FAIL — do not release.** The exact candidate is live and the repaired
single-replica SQLite/rate-limit behavior now passes fresh independent probes.
All 28 claim commands, 11 frontend tests, 34 backend tests, 28 browser tests,
strict builds, lint, accessibility, privacy, security-header, cache, and
performance checks pass.

Production still cannot complete the real job. Fresh
`/api/v1/integrations/status` reports both
`billing.configured: false` for `booking-recovery-loop-deposit` and
`delivery.configured: false`. The exact deposit checkout endpoint returns 404.
Therefore clients cannot finish deposits and the service cannot send real
recovery messages or reminders. Local provider fixtures do not satisfy the
live end-to-end acceptance contract.

## Key evidence

- `/health`: exact candidate SHA `e800fff12a71d3e0867f16d13bbe9caf480eca11`.
- Candidate/live HTML, JS, and CSS hashes match exactly.
- Live write limit: 12 accepted, 28 limited; `Retry-After: 60`.
- Live read limit: 40 accepted, 120 limited; `Retry-After: 1`.
- Post-reset old token: 24/24 reads returned 404.
- Axe: zero serious/critical findings over eight routes at desktop and 390 px.
- Lighthouse mobile: 100 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1.704 s, CLS 0, total transfer 156,703 bytes.
- Demo recovery, no-consent stop, malformed input recovery, reset, keyboard,
  reduced motion, 200% reflow, same-origin request log, and offline state pass.
- Entra redirect uses the required CIAM tenant, client ID, callback, code flow,
  and PKCE S256.

## Local verification

```sh
npm ci
npm test
npm run check:backend
npm run test:deployment
npm run test:e2e
VITE_BUILD_SHA=e800fff12a71d3e0867f16d13bbe9caf480eca11 npm run build
npm run check:size
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
BUILD_SHA=e800fff12a71d3e0867f16d13bbe9caf480eca11 cargo build --manifest-path backend/Cargo.toml --release --locked
node scripts/verify-live.mjs https://booking-recovery-loop.sociobot.in .factory/verification-artifacts/live
```

Docker/Podman/Buildah were unavailable locally. The candidate-stamped web and
locked backend release builds passed, and the exact candidate is serving live.

## Required next actions

1. Enable the dedicated booking-deposit product through the approved billing
   boundary.
2. Provision the product-specific email/SMS relay and signed callback.
3. Reverify a real paid booking, recovery, reminder, bounce, and one SMS
   fallback end to end.
4. Confirm Entra callback completion with a test customer account, including
   persistent workspace export and deletion.

No product code was changed. No unrelated resource, service, database,
application setting, or secret was accessed or modified.
